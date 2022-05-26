use crate::prelude::*;
use actix_web::http::header::ContentType;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/echo")
            .route("", web::get().to(verification_message))
            .route("", web::post().to(receive_message)),
    )
    .service(get_config)
    .service(login)
    .service(get_userinfo)
    .service(menu_create)
    .service(make_qrcode);
}

/// 验证来自微信服务器的消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Basic_Information/Access_Overview.html#%E7%AC%AC%E4%BA%8C%E6%AD%A5%EF%BC%9A%E9%AA%8C%E8%AF%81%E6%B6%88%E6%81%AF%E7%9A%84%E7%A1%AE%E6%9D%A5%E8%87%AA%E5%BE%AE%E4%BF%A1%E6%9C%8D%E5%8A%A1%E5%99%A8
async fn verification_message(
    web::Query(query): web::Query<ser::wx::ValidationMessage>,
) -> HttpResult {
    let is_ok = wx::validation_message(&query);
    if is_ok {
        return Ok(HttpResponse::Ok().body(query.echostr));
    } else {
        return Ok(HttpResponse::Ok().body("false"));
    }
}

/// 接收微信转发来的用户消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html
async fn receive_message(body: web::Bytes) -> HttpResult {
    log::info!("{:#?}", &body);
    let msg: ser::wx::WxMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
    let msg_type: &str = msg.msg_type.as_str();

    match msg_type {
        "text" => {
            let msg: ser::wx::TextMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        "image" => {
            let msg: ser::wx::ImageMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        "voice" => {
            let msg: ser::wx::VoiceMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        "video" => {
            let msg: ser::wx::VideoMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        "shortvideo" => {
            let msg: ser::wx::ShortVideoMsg =
                serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        "location" => {
            let msg: ser::wx::LocationMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        "link" => {
            let msg: ser::wx::LinkMsg = serde_xml_rs::from_reader(body.as_ref()).map_err(ej)?;
            log::info!("{:?}", &msg);
        }
        _ => (),
    }

    // https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Passive_user_reply_message.html#%E5%9B%9E%E5%A4%8D%E6%96%87%E6%9C%AC%E6%B6%88%E6%81%AF
    Ok(HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(format!(
            r#"
            <xml>
                <ToUserName><![CDATA[{to}]]></ToUserName>
                <FromUserName><![CDATA[{from}]]></FromUserName>
                <CreateTime>{time}</CreateTime>
                <MsgType><![CDATA[text]]></MsgType>
                <Content><![CDATA[{content}]]></Content>
            </xml>"#,
            to = msg.from_user_name,
            from = msg.to_user_name,
            time = timestamp!(),
            content = "你好!!!"
        )))
}

/// 使用code登录
///
/// https://developers.weixin.qq.com/doc/offiaccount/OA_Web_Apps/Wechat_webpage_authorization.html
#[http_get("/login")]
async fn login(
    web::Query(query): web::Query<ser::wx::LoginQuery>,
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let userid: u64 = wx::get_oauth_access_token(&redis_pool, &client, &query.code)
        .await
        .map_err(ej)?;
    res_ok!(userid)
}

#[derive(Debug, Deserialize)]
struct GetUserinfo {
    user_id: String,
}

/// 获取用户信息
#[http_get("/get_userinfo")]
async fn get_userinfo(
    web::Query(query): web::Query<GetUserinfo>,
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let userinfo: ser::wx::Userinfo = wx::get_userinfo(&redis_pool, &client, &query.user_id)
        .await
        .map_err(ej)?;
    res_ok!(userinfo)
}

/// 获取jssdk的config参数
#[http_get("/get_config")]
async fn get_config(
    web::Query(query): web::Query<ser::wx::GetJsSdkConfig>,
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let res = wx::get_jssdk_config(&redis_pool, &client, &query.url)
        .await
        .map_err(ej)?;
    res_ok!(res)
}

/// 生成带参数的二维码
///
/// https://developers.weixin.qq.com/doc/offiaccount/Account_Management/Generating_a_Parametric_QR_Code.html
#[http_post("/make_qrcode")]
async fn make_qrcode(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let _ = wx::make_qrcode(&redis_pool, &client).await.map_err(ej)?;

    res_ok!()
}

/// 自定义菜单
///
/// https://developers.weixin.qq.com/doc/offiaccount/Custom_Menus/Creating_Custom-Defined_Menu.html
#[http_post("/menu_create")]
async fn menu_create(
    redis_pool: web::Data<RedisPool>,
    client: web::Data<awc::Client>,
) -> HttpResult {
    let errcode = wx::menu_create(&redis_pool, &client).await.map_err(ej)?;
    res_ok!(errcode)
}
