use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct WxApiRes {
    pub errcode: i64,
    pub errmsg: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidationMessage {
    /// 微信加密签名，signature结合了开发者填写的token参数和请求中的timestamp参数、nonce参数。
    pub signature: String,

    /// 时间戳(秒)
    pub timestamp: String,

    /// 随机数
    pub nonce: String,

    /// 随机字符串
    pub echostr: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccessTokenResult {
    /// 获取到的凭证
    pub access_token: Option<String>,

    /// 凭证有效时间，单位：秒
    pub expires_in: Option<usize>,

    /// 返回码
    pub errcode: Option<i64>,

    /// 说明
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OauthAccessTokenResult {
    /// 网页授权接口调用凭证,注意：此access_token与基础支持的access_token不同
    pub access_token: Option<String>,

    /// access_token接口调用凭证超时时间，单位（秒）
    pub expires_in: Option<usize>,

    /// 用户刷新access_token
    pub refresh_token: Option<String>,

    /// 用户唯一标识，请注意，在未关注公众号时，用户访问公众号的网页，也会产生一个用户和公众号唯一的OpenID
    pub openid: Option<String>,

    /// 用户授权的作用域，使用逗号（,）分隔
    pub scope: Option<String>,

    /// 错误时微信会返回JSON数据包如下（示例为Code无效错误）:
    /// 返回码
    pub errcode: Option<i64>,

    /// 说明
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsapiTicketResult {
    /// ticket 值
    pub ticket: Option<String>,

    /// 凭证有效时间，单位：秒
    pub expires_in: Option<usize>,

    /// 返回码
    pub errcode: Option<i64>,

    /// 说明
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetJsSdkConfig {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct JsSdkConfigResult {
    /// 必填，公众号的唯一标识
    #[serde(rename(serialize = "appId"))]
    pub app_id: String,

    /// 必填，生成签名的时间戳
    pub timestamp: i64,

    /// 必填，生成签名的随机串
    #[serde(rename(serialize = "nonceStr"))]
    pub nonce_str: String,

    /// 必填，签名
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct WxMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,
}

/// 文本消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E6%96%87%E6%9C%AC%E6%B6%88%E6%81%AF
#[derive(Debug, Serialize, Deserialize)]
pub struct TextMsg {
    #[serde(rename = "ToUserName")]
    pub to_user_name: String,

    #[serde(rename = "FromUserName")]
    pub from_user_name: String,

    #[serde(rename = "CreateTime")]
    pub create_time: i64,

    #[serde(rename = "MsgType")]
    pub msg_type: String,

    #[serde(rename = "Content")]
    pub content: String,

    #[serde(skip_serializing)]
    #[serde(rename = "MsgId")]
    pub msg_id: u64,
}

/// 图片消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E5%9B%BE%E7%89%87%E6%B6%88%E6%81%AF
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "CreateTime"))]
    pub create_time: i64,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,

    #[serde(rename(deserialize = "PicUrl"))]
    pub pic_url: String,

    #[serde(rename(deserialize = "MediaId"))]
    pub media_id: String,

    #[serde(rename(deserialize = "MsgId"))]
    pub msg_id: u64,
}

/// 语音消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E8%AF%AD%E9%9F%B3%E6%B6%88%E6%81%AF
///
/// 请注意，开通语音识别后，用户每次发送语音给公众号时，微信会在推送的语音消息XML数据包中，增加一个Recognition字段
///
#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "CreateTime"))]
    pub create_time: i64,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,

    #[serde(rename(deserialize = "MediaId"))]
    pub media_id: String,

    #[serde(rename(deserialize = "Format"))]
    pub format: String,

    #[serde(rename(deserialize = "Recognition"))]
    pub recognition: Option<String>,

    #[serde(rename(deserialize = "MsgId"))]
    pub msg_id: u64,
}

/// 视频消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E8%A7%86%E9%A2%91%E6%B6%88%E6%81%AF
///
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "CreateTime"))]
    pub create_time: i64,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,

    #[serde(rename(deserialize = "MediaId"))]
    pub media_id: String,

    #[serde(rename(deserialize = "ThumbMediaId"))]
    pub thumb_media_id: String,

    #[serde(rename(deserialize = "MsgId"))]
    pub msg_id: u64,
}

/// 小视频消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E5%B0%8F%E8%A7%86%E9%A2%91%E6%B6%88%E6%81%AF
///
#[derive(Debug, Serialize, Deserialize)]
pub struct ShortVideoMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "CreateTime"))]
    create_time: i64,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,

    #[serde(rename(deserialize = "MediaId"))]
    pub media_id: String,

    #[serde(rename(deserialize = "ThumbMediaId"))]
    pub thumb_media_id: String,

    #[serde(rename(deserialize = "MsgId"))]
    pub msg_id: u64,
}

/// 地理位置消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E5%9C%B0%E7%90%86%E4%BD%8D%E7%BD%AE%E6%B6%88%E6%81%AF
///
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "CreateTime"))]
    create_time: i64,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,

    #[serde(rename(deserialize = "Location_X"))]
    pub location_x: f64,

    #[serde(rename(deserialize = "Location_Y"))]
    pub location_y: f64,

    #[serde(rename(deserialize = "Scale"))]
    pub scale: i32,

    #[serde(rename(deserialize = "Label"))]
    pub label: String,

    #[serde(rename(deserialize = "MsgId"))]
    pub msg_id: u64,
}

/// 链接消息
///
/// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Receiving_standard_messages.html#%E9%93%BE%E6%8E%A5%E6%B6%88%E6%81%AF
///
#[derive(Debug, Serialize, Deserialize)]
pub struct LinkMsg {
    #[serde(rename(deserialize = "ToUserName"))]
    pub to_user_name: String,

    #[serde(rename(deserialize = "FromUserName"))]
    pub from_user_name: String,

    #[serde(rename(deserialize = "CreateTime"))]
    create_time: i64,

    #[serde(rename(deserialize = "MsgType"))]
    pub msg_type: String,

    #[serde(rename(deserialize = "Title"))]
    pub title: String,

    #[serde(rename(deserialize = "Description"))]
    pub description: String,

    #[serde(rename(deserialize = "Url"))]
    pub url: String,

    #[serde(rename(deserialize = "MsgId"))]
    pub msg_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Userinfo {
    /// 用户的唯一标识
    pub openid: Option<String>,

    /// 用户昵称
    pub nickname: Option<String>,

    /// 用户的性别，值为1时是男性，值为2时是女性，值为0时是未知
    pub sex: Option<u8>,

    /// 用户个人资料填写的省份
    pub province: Option<String>,

    /// 普通用户个人资料填写的城市
    pub city: Option<String>,

    /// 国家，如中国为CN
    pub country: Option<String>,

    /// 用户头像，最后一个数值代表正方形头像大小（有0、46、64、96、132数值可选，0代表640*640正方形头像），用户没有头像时该项为空。若用户更换头像，原有头像URL将失效。
    pub headimgurl: Option<String>,

    /// 用户特权信息，json 数组，如微信沃卡用户为（chinaunicom）
    pub privilege: Option<Vec<String>>,

    /// 只有在用户将公众号绑定到微信开放平台帐号后，才会出现该字段。
    pub unionid: Option<String>,

    pub errcode: Option<i64>,

    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub code: String,
}
