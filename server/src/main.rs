use axum::{http::StatusCode, response::IntoResponse, routing::{get, post, put}, Json, Router};
use axum::http::Method;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

mod db;
mod handlers;
mod wechat;

use db::{init_pool, DatabaseConfig, DbPool};
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
struct Pagination { page: u32, limit: u32, pages: u32 }

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self { code: 0, message: "ok".into(), data: Some(data), pagination: None, total: None }
    }

    fn msg(code: i32, msg: &str) -> Self {
        Self { code, message: msg.into(), data: None, pagination: None, total: None }
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
        .route("/wechat/jssdkConfig", post(handlers::api_wechat_jssdk))
        .route("/wechat/getUserInfo", post(handlers::api_wechat_userinfo))
        .route("/wechat/bindPhone", post(handlers::api_wechat_bind_phone))
        .route(
            "/wechat/decryptPhoneNumber",
            post(handlers::api_wechat_decrypt_phone),
        )
        // 学员课程相关
        .route(
            "/api/student/courses",
            get(handlers::api_student_courses_by_phone),
        )
        // 管理端路由（最小占位）
        .route("/admin/dashboard", get(api_admin_dashboard))
        .route("/admin/users", get(api_admin_users))
        .route("/admin/users/:id", put(api_admin_update_user).delete(api_admin_delete_user))
        .route("/admin/courses", get(api_admin_courses).post(api_admin_create_course))
        .route("/admin/courses/:id", put(api_admin_update_course).delete(api_admin_delete_course))
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


async fn api_admin_dashboard() -> impl IntoResponse {
    #[derive(Serialize)]
    struct Stats { users: u64, courses: u64, revenue: f64 }
    (StatusCode::OK, Json(ApiResponse::ok(Stats { users: 1200, courses: 86, revenue: 128000.0 })))
}

async fn api_admin_users() -> impl IntoResponse {
    #[derive(Serialize)]
    struct User { _id: String, name: String, phone: String, memberLevel: String, role: String, createdAt: String }
    let list = vec![User { _id: "u1".into(), name: "张三".into(), phone: "13800138000".into(), memberLevel: "普通会员".into(), role: "user".into(), createdAt: "2025-01-01".into() }];
    (StatusCode::OK, Json(ApiResponse { code: 0, message: "ok".into(), data: Some(list), pagination: Some(Pagination { page: 1, limit: 10, pages: 1 }), total: Some(1) }))
}

async fn api_admin_update_user() -> impl IntoResponse { (StatusCode::OK, Json(ApiResponse::<()>::msg(0, "updated"))) }
async fn api_admin_delete_user() -> impl IntoResponse { (StatusCode::OK, Json(ApiResponse::<()>::msg(0, "deleted"))) }

async fn api_admin_courses() -> impl IntoResponse {
    #[derive(Serialize)]
    struct Course { _id: String, name: String, price: f64, description: String, image: String, isFeatured: bool, r#type: String, duration: u32, totalSessions: u32 }
    let list = vec![Course {_id:"c1".into(), name:"篮球基础".into(), price:199.0, description:"入门训练".into(), image:"https://img.yzcdn.cn/vant/cat.jpeg".into(), isFeatured:true, r#type:"篮球".into(), duration:60, totalSessions:10 }];
    (StatusCode::OK, Json(ApiResponse { code: 0, message: "ok".into(), data: Some(list), pagination: Some(Pagination { page: 1, limit: 10, pages: 1 }), total: Some(1) }))
}

async fn api_admin_create_course() -> impl IntoResponse { (StatusCode::OK, Json(ApiResponse::<()>::msg(0, "created"))) }
async fn api_admin_update_course() -> impl IntoResponse { (StatusCode::OK, Json(ApiResponse::<()>::msg(0, "updated"))) }
async fn api_admin_delete_course() -> impl IntoResponse { (StatusCode::OK, Json(ApiResponse::<()>::msg(0, "deleted"))) }
