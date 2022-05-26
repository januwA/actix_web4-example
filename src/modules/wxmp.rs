use crate::prelude::*;
use actix_web::web::Bytes;

pub static APP_ID: &str = env!("WXMP_APP_ID");
pub static APPSECRET: &str = env!("WXMP_APPSECRET");

#[derive(Debug, Deserialize)]
pub struct MpLogin {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct WxmpApiErrorResult {
    pub errcode: i64,
    pub errmsg: String,
}

#[derive(Debug, Deserialize)]
pub struct Code2SessionResult {
    /// 用户唯一标识
    pub openid: Option<String>,

    /// 会话密钥
    pub session_key: Option<String>,

    /// 错误码
    pub errcode: Option<i64>,

    /// 错误信息
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccessTokenResult {
    /// 获取到的凭证
    pub access_token: Option<String>,

    /// 凭证有效时间，单位：秒。目前是7200秒之内的值。
    pub expires_in: Option<usize>,

    /// 返回码
    pub errcode: Option<i64>,

    /// 说明
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetUserPhoneNumberResult {
    /// 返回码
    pub errcode: i64,

    /// 说明
    pub errmsg: String,

    pub phone_info: PhoneInfo,
}

#[derive(Debug, Deserialize)]
pub struct PhoneInfo {
    #[serde(rename(deserialize = "phoneNumber"))]
    pub phone_number: String,

    #[serde(rename(deserialize = "purePhoneNumber"))]
    pub pure_phone_number: String,

    #[serde(rename(deserialize = "countryCode"))]
    pub country_code: String,

    pub watermark: Watermark,
}

#[derive(Debug, Deserialize)]
pub struct Watermark {
    pub timestamp: i64,
    pub appid: String,
}

/// 通过 wx.login 接口获得临时登录凭证 code 后传到开发者服务器调用此接口完成登录流程
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/login/auth.code2Session.html
pub async fn login(redis_pool: &RedisPool, client: &awc::Client, code: &str) -> R<PK> {
    let body : Bytes = client.get(format!(
        "https://api.weixin.qq.com/sns/jscodesession?appid={appid}&secret={secret}&js_code={js_code}&grant_type=authorization_code",
        appid = APP_ID,
        secret = APPSECRET,
        js_code = code,

    ))
    .send()
    .await.map_err(eany)?
    .body()
    .await.map_err(eany)?;

    let mut con = redis_pool.get().await?;

    // TODO: 模拟的userid
    let user_id: u64 = con.incr("wxmp:userid", 1).await?;
    let key = &format!("{}:wxmp:session:{}", env!("APP_NAME"), user_id);

    let res: Code2SessionResult = serde_json::from_slice(&body)?;

    if let Some(errcode) = res.errcode {
        bail!(format!("获取session_key失败: {}", errcode));
    }

    let openid = res.openid.unwrap();
    let session_key = res.session_key.unwrap();

    // 没有过期时间，需要前端使用 wx.checkSession 检查是否过期，过期再次调用login接口就行
    // https://developers.weixin.qq.com/miniprogram/dev/api/open-api/login/wx.checkSession.html
    con.hset_multiple(key, &[("openid", &openid), ("session_key", &session_key)])
        .await?;

    Ok(user_id)
}

/// 获取小程序全局唯一后台接口调用凭据（access_token）
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/access-token/auth.getAccessToken.html
///
/// 和微信公众号的一样
pub async fn get_access_token(redis_pool: &RedisPool, client: &awc::Client) -> R<String> {
    let mut con = redis_pool.get().await?;
    let key = rk_wxmp_access_token!();

    let access_token: Option<String> = con.get(key).await?;
    Ok(match access_token {
        Some(access_token) => access_token,
        _ => {
            let body : Bytes = client.get(format!(
                "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={APP_ID}&secret={APPSECRET}"
            ))
            .send()
            .await.map_err(eany)?
            .body()
            .await.map_err(eany)?;

            let res: AccessTokenResult = serde_json::from_slice(&body)?;

            if let Some(errcode) = res.errcode {
                bail!(format!("获取access_token失败: {}", errcode));
            }

            let access_token = res.access_token.unwrap(); // access_token的存储至少要保留512个字符空间
            let expires_in = res.expires_in.unwrap() - 60 * 5; // access_token的有效期目前为2个小时，提前五分钟刷新

            // 存缓存
            con.set_ex(key, &access_token, expires_in).await?;

            access_token
        }
    })
}

/// 获取小程序二维码，适用于需要的码数量较少的业务场景。通过该接口生成的小程序码，永久有效，有数量限制
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/qr-code/wxacode.createQRCode.html
pub async fn create_qrcode(redis_pool: &RedisPool, client: &awc::Client) -> R<Bytes> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "path": "pages/index/index",
        "width": 430,
    });

    let mut req = client
        .post(&format!(
            "https://api.weixin.qq.com/cgi-bin/wxaapp/createwxaqrcode?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?;

    // "content-type": Value { inner: ["image/jpeg"] }
    log::info!("{:?}", req.headers());

    let body = req.body().await.map_err(eany)?;
    let content_type = req.headers().get("content-type").unwrap().to_str().unwrap();

    if content_type != "image" {
        let res: WxmpApiErrorResult = serde_json::from_slice(&body)?;
        // 获取二维码失败
        bail!(format!("获取小程序二维码失败: {}", res.errcode));
    }

    Ok(body)
}

/// 获取小程序码，适用于需要的码数量较少的业务场景。通过该接口生成的小程序码，永久有效，有数量限制
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/qr-code/wxacode.get.html
pub async fn create_mpcode(redis_pool: &RedisPool, client: &awc::Client) -> R<Bytes> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "path": "pages/index/index",
        "env_version": "release",
        "width": 430,
    });

    let mut req = client
        .post(&format!(
            "https://api.weixin.qq.com/wxa/getwxacode?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?;

    // "content-type": Value { inner: ["image/jpeg"] }
    log::info!("{:?}", &req.headers());

    let body = req.body().await.map_err(eany)?;
    let content_type: String = req
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .into();

    log::info!("{:?}", content_type);

    if !content_type.starts_with("image") {
        let res: WxmpApiErrorResult = serde_json::from_slice(&body)?;
        // 获取二维码失败
        bail!(format!("获取小程序码失败: {}", res.errcode));
    }

    Ok(body)
}

/// 获取小程序码，适用于需要的码数量极多的业务场景。通过该接口生成的小程序码，永久有效，数量暂无限制
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/qr-code/wxacode.getUnlimited.html
pub async fn create_unlimited_mpcode(redis_pool: &RedisPool, client: &awc::Client) -> R<Bytes> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "scene": "1",
        "path": "pages/index/index",
        "env_version": "release",
        "width": 430,
    });

    let mut req = client
        .post(&format!(
            "https://api.weixin.qq.com/wxa/getwxacodeunlimit?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?;

    // "content-type": Value { inner: ["image/jpeg"] }
    log::info!("{:?}", &req.headers());

    let body = req.body().await.map_err(eany)?;
    let content_type: String = req
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .into();

    log::info!("{:?}", content_type);

    if !content_type.starts_with("image") {
        let res: WxmpApiErrorResult = serde_json::from_slice(&body)?;
        // 获取二维码失败
        bail!(format!("获取小程序码失败: {}", res.errcode));
    }

    Ok(body)
}

/// code换取用户手机号。 每个code只能使用一次，code的有效期为5min
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/phonenumber/phonenumber.getPhoneNumber.html
pub async fn get_phone_number(
    redis_pool: &RedisPool,
    client: &awc::Client,
    code: &str,
) -> R<String> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "code": code,
    });

    let body: Bytes = client
        .post(&format!(
            "https://api.weixin.qq.com/wxa/business/getuserphonenumber?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?
        .body()
        .await
        .map_err(eany)?;

    // {"errcode":0,"errmsg":"ok","phone_info":{"phoneNumber":"xxx","purePhoneNumber":"xxx","countryCode":"86","watermark":{"timestamp":1651754399,"appid":"wx3d8295b2f9b0e732"}}}
    // log::info!("{}", std::str::from_utf8(&body).unwrap());

    let res: GetUserPhoneNumberResult = serde_json::from_slice(&body)?;

    if res.errcode != 0 {
        bail!(format!("获取失败: {}", res.errcode));
    }

    Ok(res.phone_info.phone_number)
}

/// 获取小程序 scheme 码，适用于短信、邮件、外部网页、微信内等拉起小程序的业务场景。目前仅针对国内非个人主体的小程序开放
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/url-scheme/urlscheme.generate.html
pub async fn get_urlscheme(redis_pool: &RedisPool, client: &awc::Client) -> R<Bytes> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "jump_wxa": {
            "path": "pages/index/index"
        },
        "expire_type": 1,
        "expire_interval": 1,
    });

    let mut req = client
        .post(&format!(
            "https://api.weixin.qq.com/wxa/generatescheme?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?;

    // "content-type": Value { inner: ["image/jpeg"] }
    log::info!("{:?}", &req.headers());

    let body = req.body().await.map_err(eany)?;
    let content_type: String = req
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .into();

    log::info!("{:?}", content_type);

    if !content_type.starts_with("image") {
        let res: WxmpApiErrorResult = serde_json::from_slice(&body)?;
        bail!(format!("获取小程序 scheme 码失败: {}", res.errcode));
    }

    Ok(body)
}

/// 获取小程序 URL Link，适用于短信、邮件、网页、微信内等拉起小程序的业务场景。目前仅针对国内非个人主体的小程序开放
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/url-link/urllink.generate.html
pub async fn get_urllink(redis_pool: &RedisPool, client: &awc::Client) -> R<Bytes> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "path": "pages/index/index",
        "env_version": "develop",
        "expire_type": 1,
        "expire_interval": 1,
    });

    let mut req = client
        .post(&format!(
            "https://api.weixin.qq.com/wxa/generate_urllink?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?;

    // "content-type": Value { inner: ["image/jpeg"] }
    log::info!("{:?}", &req.headers());

    let body = req.body().await.map_err(eany)?;
    let content_type: String = req
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .into();

    log::info!("{:?}", content_type);

    if !content_type.starts_with("image") {
        let res: WxmpApiErrorResult = serde_json::from_slice(&body)?;
        bail!(format!("获取小程序 URL Link 失败: {}", res.errcode));
    }

    Ok(body)
}

/// 提供基于小程序的图片智能裁剪能力
///
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/img/img.aiCrop.html
pub async fn img_ai_crop(redis_pool: &RedisPool, client: &awc::Client, img_url: &str) -> R<i32> {
    let access_token = wxmp::get_access_token(&redis_pool, &client).await?;

    let body = client
        .post(&format!(
            "https://api.weixin.qq.com/cv/img/aicrop?img_url={img_url}&access_token={access_token}"
        ))
        .send()
        .await
        .map_err(eany)?
        .body()
        .await
        .map_err(eany)?;

    log::info!("{}", std::str::from_utf8(&body).unwrap());

    Ok(1)
}
