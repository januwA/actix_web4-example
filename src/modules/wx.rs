use crate::prelude::*;
use ser::wx::*;

pub static APP_ID: &str = env!("WX_APP_ID");
pub static APPSECRET: &str = env!("WX_APPSECRET");
pub static TOKEN: &str = env!("WX_TOKEN");
pub static ENCODING_AES_KEY: &str = env!("WX_ENCODING_AES_KEY");

/// 验证消息的确来自微信服务器
///
/// https://developers.weixin.qq.com/doc/offiaccount/Basic_Information/Access_Overview.html
pub fn validation_message(query: &ValidationMessage) -> bool {
    // 排序
    let mut tmp_arr: Vec<&str> = vec![TOKEN, &query.timestamp, &query.nonce];
    tmp_arr.sort();

    // 拼接字符串
    let tmp_str = tmp_arr.join("");

    let sign = sha1!(tmp_str.as_ref());

    // 检查是否一致
    sign == query.signature
}

/// 获取 Access token
///
/// https://developers.weixin.qq.com/doc/offiaccount/Basic_Information/Get_access_token.html
///
pub async fn get_access_token(redis_pool: &RedisPool, client: &awc::Client) -> R<String> {
    let mut con = redis_pool.get().await?;
    let key = rk_wx_access_token!();
    let access_token: Option<String> = con.get(key).await?;
    Ok(match access_token {
        Some(access_token) => access_token,
        _ => {
            let body : web::Bytes = client.get(format!(
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

/// 获取 jsapi_ticket
///
/// https://developers.weixin.qq.com/doc/offiaccount/OA_Web_Apps/JS-SDK.html#62
pub async fn get_jsapi_ticket(
    redis_pool: &RedisPool,
    client: &awc::Client,
    access_token: &str,
) -> R<String> {
    let mut con = redis_pool.get().await?;
    let key = rk_wx_jsapi_ticket!();
    let jsapi_ticket: Option<String> = con.get(key).await?;
    Ok(match jsapi_ticket {
        Some(ticket) => ticket,
        _ => {
            let body: web::Bytes = client.get(format!(
                "https://api.weixin.qq.com/cgi-bin/ticket/getticket?access_token={access_token}&type=jsapi"
            ))
            .send()
            .await.map_err(eany)?
            .body()
            .await.map_err(eany)?;

            let res: JsapiTicketResult = serde_json::from_slice(&body)?;

            if let Some(errcode) = res.errcode {
                bail!(format!("获取ticket失败:{}", errcode));
            }

            let ticket = res.ticket.unwrap();
            let expires_in = res.expires_in.unwrap() - 60 * 5; // 提前五分钟刷新

            // 存缓存
            con.set_ex(key, &ticket, expires_in).await?;

            ticket
        }
    })
}

/// 获取 OAuth Access token
///
/// 通过code换取的是一个特殊的网页授权access_token,
/// 与基础支持中的access_token（该access_token用于调用其他接口）不同
///
/// https://developers.weixin.qq.com/doc/offiaccount/OA_Web_Apps/Wechat_webpage_authorization.html
pub async fn get_oauth_access_token(
    redis_pool: &RedisPool,
    client: &awc::Client,
    code: &str,
) -> R<PK> {
    let mut con = redis_pool.get().await?;

    // 通过redis的递增来获取id
    // TODO: 从数据库中创建新用户，然后获取id
    let user_id: PK = con.incr(concat!(env!("APP_NAME"), ":wx:userid"), 1).await?;

    // 每个用户的key都不同
    let key = &format!("{}:wx:oauth:{}", env!("APP_NAME"), user_id);

    let access_token: Option<String> = con.hget(key, "access_token").await?;

    if access_token.is_none() {
        let body : web::Bytes = client.get(&format!(
                  "https://api.weixin.qq.com/sns/oauth2/access_token?appid={APP_ID}&secret={APPSECRET}&code={code}&grant_type=authorization_code"))
                  .send()
                  .await.map_err(eany)?
                  .body()
                  .await
                  .map_err(eany)?;
        let res: OauthAccessTokenResult = serde_json::from_slice(&body)?;

        if let Some(errcode) = res.errcode {
            bail!(format!("获取access_token失败:{}", errcode));
        }

        let access_token = res.access_token.unwrap();
        let refresh_token = res.refresh_token.unwrap();
        let openid = res.openid.unwrap();
        let scope = res.scope.unwrap();
        let expires_in = res.expires_in.unwrap();

        // 存缓存
        redis::pipe()
            .hset_multiple(
                key,
                &[
                    ("access_token", &access_token),
                    ("refresh_token", &refresh_token),
                    ("openid", &openid),
                    ("scope", &scope),
                ],
            )
            .expire(key, expires_in) // access_token 的过期时间， refresh_token 有效期为30天
            .query_async(&mut con)
            .await?;
    };

    return Ok(user_id);
}

pub async fn get_userinfo(
    redis_pool: &RedisPool,
    client: &awc::Client,
    user_id: &str,
) -> R<Userinfo> {
    let mut con = redis_pool.get().await?;
    let key = &rk_wx_userinfo!(user_id);

    if !con.exists(key).await? {
        bail!("用户不存在");
    }

    let (access_token, openid): (String, String) =
        con.hget(key, &["access_token", "openid"]).await?;

    let body : web::Bytes = client.get(&format!(
                  "https://api.weixin.qq.com/sns/userinfo?access_token={access_token}&openid={openid}&lang=zh_CN"))
                  .send()
                  .await.map_err(eany)?
                  .body()
                  .await.map_err(eany)?;

    let res: Userinfo = serde_json::from_slice(&body)?;

    if let Some(errcode) = res.errcode {
        bail!(format!("获取用户信息失败:{}", errcode));
    }

    return Ok(res);
}

/// 获取 jssdk 的配置信息
///
/// https://developers.weixin.qq.com/doc/offiaccount/OA_Web_Apps/JS-SDK.html#1
///
/// JS-SDK使用权限签名算法:
/// https://developers.weixin.qq.com/doc/offiaccount/OA_Web_Apps/JS-SDK.html#62
pub async fn get_jssdk_config(
    redis_pool: &RedisPool,
    client: &awc::Client,
    url: &str,
) -> R<JsSdkConfigResult> {
    let access_token = get_access_token(redis_pool, client).await?;
    let jsapi_ticket = get_jsapi_ticket(redis_pool, client, &access_token).await?;
    let nonce_str = make_random_string!(16);
    let timestamp: i64 = timestamp!();

    let tmp_str =
        format!("jsapi_ticket={jsapi_ticket}&noncestr={nonce_str}&timestamp={timestamp}&url={url}");

    let sign: String = sha1!(tmp_str.as_ref());

    return Ok(JsSdkConfigResult {
        app_id: APP_ID.into(),
        timestamp,
        nonce_str,
        signature: sign,
    });
}

/// 生成带参数的二维码
///
/// https://developers.weixin.qq.com/doc/offiaccount/Account_Management/Generating_a_Parametric_QR_Code.html
pub async fn make_qrcode(redis_pool: &RedisPool, client: &awc::Client) -> R<()> {
    let access_token = wx::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        // 该二维码有效时间，以秒为单位。 最大不超过2592000（即30天），此字段如果不填，则默认有效期为60秒。
        "expire_seconds": 604800,

        // 二维码类型，QR_SCENE为临时的整型参数值，QR_STR_SCENE为临时的字符串参数值，QR_LIMIT_SCENE为永久的整型参数值，QR_LIMIT_STR_SCENE为永久的字符串参数值
        "action_name": "QR_SCENE",

        // 二维码详细信息
        "action_info": r#"{"scene": {"scene_id": 1}}"#,

        // 场景值ID，临时二维码时为32位非0整型，永久二维码时最大值为100000（目前参数只支持1--100000）
        "scene_id": 1u32,

        // 场景值ID（字符串形式的ID），字符串类型，长度限制为1到64
        "scene_str": "1",
    });

    // TODO: 解析返回值
    let _body: web::Bytes = client
        .post(&format!(
            "https://api.weixin.qq.com/cgi-bin/qrcode/create?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?
        .body()
        .await
        .map_err(eany)?;

    Ok(())
}

/// 创建菜单
///
/// https://developers.weixin.qq.com/doc/offiaccount/Custom_Menus/Creating_Custom-Defined_Menu.html
pub async fn menu_create(redis_pool: &RedisPool, client: &awc::Client) -> R<i64> {
    let access_token = wx::get_access_token(&redis_pool, &client).await?;

    let request = json!({
        "button":[
        {
            // 点击这个菜单，回向我们的接口推送类型为event的消息
            // <MsgType><![CDATA[event]]></MsgType>
            // <Event><![CDATA[CLICK]]></Event>
            // <EventKey><![CDATA[V1001_TODAY_MUSIC]]></EventKey>
             "type":"click",
             "name":"今日歌曲",
             "key":"V1001_TODAY_MUSIC"
         },
         {
              "name":"菜单",
              "sub_button":[
              {
                  "type":"view",
                  "name":"搜索",
                  "url":"http://www.soso.com/"
               },
            //    {
            //         "type":"miniprogram",
            //         "name":"wxa",
            //         "url":"http://mp.weixin.qq.com",
            //         "appid":"wx286b93c14bbf93aa",
            //         "pagepath":"pages/lunar/index"
            //     },
               {
                  "type":"click",
                  "name":"赞一下我们",
                  "key":"V1001_GOOD"
               }]
          }]
    });

    let body: web::Bytes = client
        .post(&format!(
            "https://api.weixin.qq.com/cgi-bin/menu/create?access_token={access_token}"
        ))
        .send_json(&request)
        .await
        .map_err(eany)?
        .body()
        .await
        .map_err(eany)?;

    let res: WxApiRes = serde_json::from_slice(&body)?;
    Ok(res.errcode)
}
