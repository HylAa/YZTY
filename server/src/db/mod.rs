use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::time::Duration;

pub mod wechat_user;

pub type DbPool = Pool<MySql>;

/// 数据库连接基础配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}
impl DatabaseConfig {
    /// 从环境变量加载数据库配置，提供默认值便于本地快速启动
    pub fn from_env() -> Self {
        let host = std::env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = std::env::var("MYSQL_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3306);
        let username = std::env::var("MYSQL_USER").unwrap_or_else(|_| "root".into());
        let password = std::env::var("MYSQL_PASSWORD").unwrap_or_else(|_| "123456".into());
        let database = std::env::var("MYSQL_DATABASE").unwrap_or_else(|_| "yzty".into());
        let max_connections = std::env::var("MYSQL_POOL_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Self {
            host,
            port,
            username,
            password,
            database,
            max_connections,
        }
    }

    /// 构造数据库连接字符串
    pub fn to_url(&self) -> String {
        format!(
            "mysql://{user}:{password}@{host}:{port}/{database}",
            user = self.username,
            password = self.password,
            host = self.host,
            port = self.port,
            database = self.database,
        )
    }
}
/// 根据配置初始化 MySQL 连接池
pub async fn init_pool(config: &DatabaseConfig) -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config.to_url());

    MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect(&database_url)
        .await
}

