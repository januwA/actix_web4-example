use crate::prelude::*;
use actix_web::{error::ResponseError, http::StatusCode, Error as ActixError};
pub use diesel::result::{DatabaseErrorKind, Error as DbError};
use serde_json::to_string_pretty;
use std::fmt::{Display, Formatter, Result as FmtResult};
use validator::{ValidationErrors, ValidationErrorsKind};

/// http json error
pub fn ej<E: Display>(e: E) -> OkErr {
    OkErr::new::<E>(e)
}

/// error to string
pub fn es<E: Display>(e: E) -> String {
    e.to_string()
}

/// anyhow
pub fn eany<E: Display>(e: E) -> anyhow::Error {
    anyhow!(e.to_string())
}

pub fn db_unique_err(e: DbError, err_msg: &'static str) -> anyhow::Error {
    match e {
        DbError::DatabaseError(kind, _) => match kind {
            DatabaseErrorKind::UniqueViolation => anyhow!(err_msg),
            _ => anyhow!(e),
        },
        _ => anyhow!(e),
    }
}

pub fn db_not_found_err(e: DbError, err_msg: &'static str) -> anyhow::Error {
    match e {
        DbError::NotFound => anyhow!(err_msg),
        _ => anyhow!(e),
    }
}

#[derive(Debug, Serialize)]
pub struct OkErr {
    /// 1普通错误，2需要登陆
    pub error_code: char,

    /// 错误消息
    pub error_message: String,

    /// 0无声,1警告消息,2错误消息,4通知,9页面
    pub show_type: i8,
}

impl Display for OkErr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for OkErr {
    fn error_response(&self) -> HttpResponse {
        // https://pro.ant.design/zh-CN/docs/request/
        // https://github.com/actix/examples/blob/master/json/json-error/src/main.rs
        HttpResponse::Ok().status(StatusCode::BAD_GATEWAY).json(json!({
            "success": false,
            "data": null,
            "errorCode": self.error_code,
            "errorMessage": self.error_message,
            "showType": self.show_type,
        }))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl OkErr {
    fn new<E: Display>(e: E) -> Self {
        Self {
            error_code: '1',
            error_message: e.to_string(),
            show_type: 2,
        }
    }

    /// 0无声,1警告消息,2错误消息,4通知,9页面
    pub fn show_type(&mut self, show_type: i8) -> &mut Self {
        self.show_type = show_type;
        self
    }

    pub fn error_code(&mut self, error_code: char) -> &mut Self {
        self.error_code = error_code;
        self
    }

    /// 需要登陆
    pub fn auth(&mut self) -> &mut Self {
        self.error_code('2')
    }

    pub fn build(&self) -> Self {
        Self {
            error_code: self.error_code,
            error_message: self.error_message.clone(),
            show_type: self.show_type,
        }
    }

    pub fn actix(&self) -> ActixError {
        ActixError::from(self.build())
    }

    /// 针对请求参数验证失败的错误
    pub fn ev(e: ValidationErrors) -> Self {
        // { phone: [...ValidationError], other: [] }

        let err_messages = e
            .into_errors()
            .iter()
            .filter_map(|(k, v)| {
                if let ValidationErrorsKind::Field(errors) = v {
                    let first_err = match errors.first() {
                        Some(x) => x,
                        _ => {
                            return Some(format!("{} 参数错误", k));
                        }
                    };

                    let first_err_message: String = match &first_err.message {
                        Some(m) => m.to_string(),
                        _ => format!("{} 参数错误", k),
                    };
                    Some(first_err_message)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        Self {
            error_code: '1',
            error_message: err_messages.join(","),
            show_type: 2,
        }
    }
}
