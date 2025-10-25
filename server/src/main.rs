use axum::http::Method;
use axum::{
    routing::{get, post},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

mod db;
mod handlers;
mod wechat;

use db::{
    admin::{self, NewAdminUser},
    init_pool, venue, DatabaseConfig, DbPool,
};
use handlers::{
    api_admin_login, api_admin_update_venue_status, api_auth_me, api_get_venue_availability,
    api_get_venue_overview, api_student_courses_by_phone, api_swim_courses_by_phone,
    api_wechat_bind_phone, api_wechat_decrypt_phone, api_wechat_jssdk, api_wechat_userinfo,
    hash_password,
};
use wechat::{WechatClient, WechatConfig};

#[derive(Clone)]
pub struct AppState {
    pub wechat_client: Arc<WechatClient>,
    pub db_pool: DbPool,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<Pagination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u64>,
}

#[derive(Serialize, Clone)]
struct Pagination {
    page: u32,
    limit: u32,
    pages: u32,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".into(),
            data: Some(data),
            pagination: None,
            total: None,
        }
    }

    fn msg(code: i32, msg: &str) -> Self {
        Self {
            code,
            message: msg.into(),
            data: None,
            pagination: None,
            total: None,
        }
    }
}

#[tokio::main]
async fn main() {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 初始化微信配置
    let wechat_config = match WechatConfig::from_env() {
        Ok(config) => {
            if let Err(e) = config.validate() {
                eprintln!("Warning: WeChat config validation failed: {}", e);
            }
            Arc::new(config)
        }
        Err(e) => {
            eprintln!("Warning: Failed to load WeChat config: {}", e);
            eprintln!("WeChat features will use mock data");
            // 使用默认配置以避免崩溃
            Arc::new(WechatConfig {
                app_id: "wx_mock_appid".to_string(),
                app_secret: "mock_secret_32_characters_______".to_string(),
            })
        }
    };

    let wechat_client = Arc::new(WechatClient::new(wechat_config));

    let db_config = DatabaseConfig::from_env();
    let db_pool = match init_pool(&db_config).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("无法连接到 MySQL 数据库: {}", e);
            eprintln!("请确认数据库已启动且账号密码配置正确");
            std::process::exit(1);
        }
    };

    if let Err(err) = admin::ensure_admin_tables(&db_pool).await {
        eprintln!("初始化管理员表失败: {}", err);
        std::process::exit(1);
    }
    if let Err(err) = venue::ensure_venue_tables(&db_pool).await {
        eprintln!("初始化场地状态表失败: {}", err);
        std::process::exit(1);
    }
    if let Err(err) = ensure_default_admin(&db_pool).await {
        eprintln!("创建默认管理员失败: {}", err);
        std::process::exit(1);
    }

    let state = Arc::new(AppState {
        wechat_client,
        db_pool: db_pool.clone(),
    });

    // 注意：当允许携带凭据时，CORS 不能与通配符头/源一起使用。
    // 这里不启用 credentials，且显式列出允许的请求头，避免运行时 panic。
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        // 健康检查
        .route("/health", get(|| async { "ok" }))
        // 微信相关路由（与前端 wxUtils 对齐）
        .route("/wechat/jssdkConfig", post(api_wechat_jssdk))
        .route("/wechat/getUserInfo", post(api_wechat_userinfo))
        .route("/wechat/bindPhone", post(api_wechat_bind_phone))
        .route("/wechat/decryptPhoneNumber", post(api_wechat_decrypt_phone))
        // 学员课程相关
        .route("/api/student/courses", get(api_student_courses_by_phone))
        .route("/api/swim/courses", get(api_swim_courses_by_phone))
        // 管理员认证
        .route("/api/auth/login", post(api_admin_login))
        .route("/api/auth/me", get(api_auth_me))
        // 场地占用
        .route("/api/venues/overview", get(api_get_venue_overview))
        .route("/api/venues/availability", get(api_get_venue_availability))
        .route(
            "/api/admin/venues/status",
            post(api_admin_update_venue_status),
        )
        .with_state(state)
        .layer(cors);

    // 读取环境变量 PORT，默认 5000
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8018);
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    println!("listening on http://{} (PORT={})", addr, port);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("端口 {} 绑定失败: {}\n可能已有进程占用该端口。", port, e);
            eprintln!(
                "macOS 可用命令排查:\n  lsof -iTCP:{} -sTCP:LISTEN -n -P\n  kill -9 <PID>",
                port
            );
            std::process::exit(1);
        }
    };
    axum::serve(listener, app).await.unwrap();
}

async fn ensure_default_admin(pool: &DbPool) -> Result<(), String> {
    let count = admin::count_admin_users(pool)
        .await
        .map_err(|err| err.to_string())?;
    if count > 0 {
        return Ok(());
    }

    let username = std::env::var("ADMIN_DEFAULT_USERNAME").unwrap_or_else(|_| {
        eprintln!("警告: 未设置 ADMIN_DEFAULT_USERNAME，使用默认账号 admin");
        "admin".to_string()
    });
    let password = std::env::var("ADMIN_DEFAULT_PASSWORD").unwrap_or_else(|_| {
        eprintln!(
            "警告: 未设置 ADMIN_DEFAULT_PASSWORD，使用默认密码 admin123。请尽快在环境变量中修改。"
        );
        "admin123".to_string()
    });

    let password_hash = hash_password(&password).map_err(|err| err.to_string())?;

    admin::insert_admin_user(
        pool,
        &NewAdminUser {
            username: &username,
            password_hash: &password_hash,
            display_name: Some("系统管理员"),
            role: "admin",
        },
    )
    .await
    .map_err(|err| err.to_string())?;

    println!(
        "默认管理员已创建，账号: {} / 密码: {} (请尽快修改密码)",
        username, password
    );

    Ok(())
}
