use crate::prelude::*;

#[derive(Debug, Deserialize, Validate)]
pub struct SendPhoneCaptcha {
    #[validate(custom(function = "vf::phone", message = "手机号格式错误"))]
    pub phone: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct LoginAccount {
    pub username: Option<String>,

    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub password: Option<String>,

    #[validate(custom = "vf::phone")]
    pub mobile: Option<String>,

    #[validate(length(equal = 4, message = "验证码长度为4位!"))]
    pub captcha: Option<String>,

    /// account 账号+密码, mobile 手机号+验证码
    pub r#type: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct PhoneUserRegister {
    #[serde(flatten)]
    pub user: models::user::NewPhoneUser,

    #[validate(length(equal = 4, message = "验证码长度为4位!"))]
    pub captcha: String,
}
