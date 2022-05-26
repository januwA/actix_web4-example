use crate::{
    models::user::{NewAccountUser, User},
    prelude::*,
};
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(account_register)
        .service(phone_register)
        .service(login)
        .service(admin_login)
        .service(get_phone_captcha)
        .service(cmd);
}

/// 执行特殊命令的接口
#[http_get("/cmd/{cmd}")]
pub async fn cmd(c: web::Path<String>) -> HttpResult {
    return match c.into_inner().as_str() {
        // 创建加密后的密码
        "pwd" => {
            res_ok!(pwd_encode!("123456"))
        }

        _ => Err(actix_web::error::ErrorNotFound("")),
    };
}

/// AccountUser 注册
#[http_post("/account_register")]
pub async fn account_register(
    db_pool: web::Data<DbPool>,
    data: HttpBody<NewAccountUser>,
) -> HttpResult {
    let mut data = data.into_inner();
    data.validate().map_err(OkErr::ev)?;

    data.password = pwd_encode!(&data.password);

    User::create_account_user(db_pool, data).await.map_err(ej)?;
    res_ok!()
}

/// PhoneUser 注册
#[http_post("/phone_register")]
pub async fn phone_register(
    db_pool: web::Data<DbPool>,
    redis_pool: web::Data<RedisPool>,
    data: HttpBody<ser::auth::PhoneUserRegister>,
) -> HttpResult {
    let data = data.into_inner();
    data.validate().map_err(OkErr::ev)?;
    data.user.validate().map_err(OkErr::ev)?;

    serv::auth::verify_phone_code(&redis_pool, &data.user.phone, &data.captcha)
        .await
        .map_err(ej)?;

    User::create_phone_user(db_pool, data.user)
        .await
        .map_err(ej)?;
    res_ok!()
}

/**
* 账号或手机号登陆

username: "admin"
password: "123"
type: "account"


mobile: "15281414664"
captcha: "123"
type: "mobile"
*/
#[http_post("login")]
pub async fn login(
    data: HttpBody<ser::auth::LoginAccount>,
    db_pool: web::Data<DbPool>,
    redis_pool: web::Data<RedisPool>,
) -> HttpResult {
    let data = data.into_inner();
    data.validate().map_err(OkErr::ev)?;

    let access_token: String = match data.r#type.as_str() {
        "account" => {
            let username = data.username.unwrap_or_default();
            let password = data.password.unwrap_or_default();
            serv::auth::account_login(username, password, db_pool)
                .await
                .map_err(ej)?
        }
        "mobile" => {
            let mobile = data.mobile.unwrap_or_default();
            let captcha = data.captcha.unwrap_or_default();
            serv::auth::phone_login(mobile, captcha, db_pool, redis_pool)
                .await
                .map_err(ej)?
        }
        _ => return res_err!("调用接口错误"),
    };

    res_ok!(json!({ "access_token": access_token }))
}

#[http_post("admin_login")]
pub async fn admin_login(
    db_pool: web::Data<DbPool>,
    data: HttpBody<ser::admin::Login>,
) -> HttpResult {
    let data = data.into_inner();
    let access_token = models::admin::Admin::login(db_pool, data.username, data.password)
        .await
        .map_err(ej)?;
    res_ok!(json!({ "access_token": access_token }))
}

/// 使用手机号发送验证码
#[http_get("get_phone_captcha")]
pub async fn get_phone_captcha(
    data: web::Query<ser::auth::SendPhoneCaptcha>,
    redis_pool: web::Data<RedisPool>,
) -> HttpResult {
    let data = data.into_inner();
    data.validate().map_err(OkErr::ev)?;

    serv::auth::send_phone_captcha(redis_pool, data.phone)
        .await
        .map_err(ej)?;
    res_ok!()
}
