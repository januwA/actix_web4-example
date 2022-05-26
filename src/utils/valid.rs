use lazy_static::lazy_static;
use regex::Regex;
use validator::ValidationError;

/// 手机号验证，允许空字符串
pub fn phone(data: &str) -> Result<(), ValidationError> {
    if data.is_empty() {
        return Ok(());
    }

    lazy_static! {
        static ref PHONE_EXP: Regex = Regex::new(
            r"^((13[0-9])|(14[0-9])|(15[0-9])|(16[0-9])|(17[0-9])|(18[0-9])|(19[0-9]))\d{8}$"
        )
        .unwrap();
    }

    if !PHONE_EXP.is_match(data) {
        return Err(ValidationError::new("phone"));
    }

    Ok(())
}

/// 邮箱验证，允许空字符串
pub fn email(data: &str) -> Result<(), ValidationError> {
    if data.is_empty() {
        return Ok(());
    }

    match validator::validate_email(data) {
        true => Ok(()),
        false => Err(ValidationError::new("email")),
    }
}

pub fn default_false() -> bool {
    false
}

pub fn default_true() -> bool {
    true
}
