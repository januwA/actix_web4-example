use crate::prelude::{DbPool, RedisPool};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};

/// 获取数据库连接池
pub fn mysql_pool() -> DbPool {
    let mysql_manager = ConnectionManager::<MysqlConnection>::new(env!("DATABASE_URL"));
    Pool::builder()
        .build(mysql_manager)
        .expect("mysql 池创建失败")
}

/// 获取redis连接池
pub fn redis_pool() -> RedisPool {
    let cfg = deadpool_redis::Config::from_url(env!("REDIS_URL"));
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("redis 池创建失败")
}

#[cfg(feature = "dev")]
pub fn env() {
    dotenv::from_filename(".env").ok();
}

#[cfg(not(feature = "dev"))]
pub fn env() {
    dotenv::from_filename("prod.env").ok();
}
