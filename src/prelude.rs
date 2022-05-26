pub use std::{collections::HashMap, sync::Arc};

pub use actix_web::{get as http_get, post as http_post, web, HttpRequest, HttpResponse};
pub use anyhow::{anyhow, bail, Result as R};
pub use bigdecimal::BigDecimal;
pub use chrono::prelude::*;
pub use diesel::{
    dsl,
    prelude::*,
    result::{DatabaseErrorKind, Error::DatabaseError},
};
pub use rand::prelude::*;
pub use redis::AsyncCommands;
pub use regex::Regex;
pub use serde::{Deserialize, Serialize};
pub use serde_json::json;
pub use url::Url;
pub use validator::Validate;

use diesel::{r2d2, sql_types};

// 创建 sql 裸函数声明
sql_function! {
    // 获取mysql表最后插入的pk
 fn last_insert_id () -> sql_types::Unsigned<sql_types::BigInt>
}

sql_function! {
 fn current_timestamp () -> sql_types::Datetime
}

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<MysqlConnection>>;
pub type RedisPool = deadpool_redis::Pool;
pub type HttpResult = actix_web::Result<HttpResponse>;
pub type HttpBody<T> = web::Either<web::Json<T>, web::Form<T>>;

pub use crate::utils::{
    error::{db_not_found_err, db_unique_err, eany, ej, es, OkErr},
    jwt::Claims,
    valid as vf,
};

pub use itertools::izip;

/// 数据库主键的类型
pub type PK = u64;

pub use crate::models;
pub use crate::schema;
pub use crate::ser;
pub use crate::serv;

#[cfg(feature = "wx")]
pub use crate::modules::wx;

#[cfg(feature = "wxmp")]
pub use crate::modules::wxmp;

#[cfg(feature = "wxpay")]
pub use crate::modules::wxpay;

#[cfg(feature = "alipay")]
pub use crate::modules::alipay;
