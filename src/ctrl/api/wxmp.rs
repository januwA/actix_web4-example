use crate::prelude::*;
use actix_web::http::header::ContentType;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(create_qrcode)
        .service(create_unlimited_mpcode)
        .service(get_phone_number)
        .service(get_urlscheme)
        .service(get_urllink)
        .service(img_ai_crop)
        .service(create_mpcode);
}

/// 通过 wx.login 接口获得临时登录凭证 code 后传到开发者服务器调用此接口完成登录流程
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/login/auth.code2Session.html
#[http_post("/login")]
async fn login(
    form: HttpBody<wxmp::MpLogin>,
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let form = form.into_inner();
    let userid: u64 = wxmp::login(&redis_pool, &client, &form.code)
        .await
        .map_err(ej)?;
    res_ok!(userid)
}

#[http_get("/create_qrcode")]
async fn create_qrcode(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let res = wxmp::create_qrcode(&redis_pool, &client)
        .await
        .map_err(ej)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(res))
}

#[http_get("/create_mpcode")]
async fn create_mpcode(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let res = wxmp::create_mpcode(&redis_pool, &client)
        .await
        .map_err(ej)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(res))
}

#[http_get("/create_unlimited_mpcode")]
async fn create_unlimited_mpcode(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let res = wxmp::create_unlimited_mpcode(&redis_pool, &client)
        .await
        .map_err(ej)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(res))
}

#[http_get("/get_urlscheme")]
async fn get_urlscheme(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let res = wxmp::get_urlscheme(&redis_pool, &client)
        .await
        .map_err(ej)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(res))
}

#[http_get("/get_urllink")]
async fn get_urllink(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let res = wxmp::get_urllink(&redis_pool, &client).await.map_err(ej)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(res))
}

#[derive(Debug, Deserialize)]
pub struct GetPhoneForm {
    pub code: String,
}

#[http_post("/get_phone_number")]
async fn get_phone_number(
    form: HttpBody<GetPhoneForm>,
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let form = form.into_inner();
    let phone = wxmp::get_phone_number(&redis_pool, &client, &form.code)
        .await
        .map_err(ej)?;

    res_ok!(phone)
}

#[derive(Debug, Deserialize)]
pub struct GetImgAiCrop {
    pub img_url: String,
}

#[http_post("/img_ai_crop")]
async fn img_ai_crop(
    form: HttpBody<GetImgAiCrop>,
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let form = form.into_inner();
    let res = wxmp::img_ai_crop(&redis_pool, &client, &form.img_url)
        .await
        .map_err(ej)?;

    res_ok!(res)
}
