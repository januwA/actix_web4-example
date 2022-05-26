use crate::prelude::*;

pub mod admin;
pub mod auth;
pub mod upload;
pub mod user;

#[cfg(feature = "wx")]
pub mod wx;

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePwd {
    /// 当前密码
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub pwd: String,

    /// 新密码
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub new_pwd: String,

    /// 确认新密码
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub new_pwd2: String,
}
