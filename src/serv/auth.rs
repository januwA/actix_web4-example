use crate::prelude::*;
use actix_web::web;
use models::user::User;

/// 验证手机验证码
pub async fn verify_phone_code(redis_pool: &RedisPool, phone: &str, code: &str) -> R<()> {
    // redis key
    let key = rk_phone_vc!(phone);

    let mut con = redis_pool.get().await?;

    // 获取发送存的code
    let cache_code: Option<String> = con.get(&key).await?;

    // 没有或则code不对返回错误
    if cache_code.is_none() || cache_code.unwrap_or_default().as_str() != code {
        bail!("无效验证码");
    }

    // 成功则删除缓存
    con.del(&key).await?;

    Ok(())
}

/// 手机号+验证码登录
pub async fn phone_login(
    phone: String,
    code: String,
    db_pool: web::Data<DbPool>,
    redis_pool: web::Data<RedisPool>,
) -> R<String> {
    verify_phone_code(&redis_pool, &phone, &code).await?;

    let c_db_pool = db_pool.clone();
    let user: User = User::obj_with_phone(c_db_pool, phone).await?;

    if !user.is_active {
        bail!("账号未激活");
    }

    User::update_last_login(db_pool, user.id).await?;

    Ok(Claims::new(user.id, user.user_type).jwt()?)
}

/// 账号+密码登录
pub async fn account_login(
    username: String,
    password: String,
    db_pool: web::Data<DbPool>,
) -> R<String> {
    let c_db_pool = db_pool.clone();
    let user: User = User::obj_with_username(c_db_pool, username).await?;

    if !pwd_decode!(&user.password.unwrap_or_default(), password.as_ref()) {
        bail!("账号或密码错误");
    }

    if !user.is_active {
        bail!("账号未激活");
    }

    User::update_last_login(db_pool, user.id).await?;
    Ok(Claims::new(user.id, user.user_type).jwt()?)
}

/// 发送手机验证码
pub async fn send_phone_captcha(redis_pool: web::Data<RedisPool>, phone: String) -> R<()> {
    let mut con = redis_pool.get().await?;
    let key = rk_phone_vc!(phone);

    // 已存在验证码不能再次发送
    if con.exists(&key).await? {
        bail!("验证码已发送，请稍后再试");
    }

    let code = make_phone_code!(4);
    con.set_ex(&key, &code, 60_usize).await?;
    Ok(())
}
