use openssl::base64;
use openssl::sha;

use crate::prelude::*;

/// 商户号
const MCHID: &str = "";

/// 商户API证书序列号
const SERIAL_NO: &str = "";

/// 计算签名值
///
/// https://pay.weixin.qq.com/wiki/doc/apiv3/wechatpay/wechatpay4_0.shtml#part-2
pub async fn calculate_signature(_redis_pool: &RedisPool, _client: &awc::Client) -> R<i32> {
    // 随机串
    let nonce_str: String = make_random_string!(8);

    // 时间戳(s)
    let timestamp = timestamp!();

    let request_str = format!(
        r#"{method}\n{path}\n{timestamp}\n{nonce_str}\n{body}\n"#,
        method = "GET",
        path = "/v3/certificates",
        timestamp = timestamp,
        nonce_str = &nonce_str,
        body = "",
    );

    log::info!("{}", request_str);

    // Sha256 加密
    let mut hasher = sha::Sha256::new();

    hasher.update(request_str.as_ref());
    hasher.update(b""); // 支付密钥
    let hex = hex::encode(hasher.finish());

    let signature = base64::encode_block(hex.as_ref());

    log::info!("{}", signature);

    let authorization = format!(
        r#"WECHATPAY2-SHA256-RSA2048 mchid="{mchid}",serial_no="{serial_no}",nonce_str="{nonce_str}",timestamp="{timestamp}",signature="{signature}""#,
        mchid = MCHID,
        serial_no = SERIAL_NO,
        nonce_str = &nonce_str,
        timestamp = timestamp,
        signature = signature
    );

    log::info!("{}", authorization);

    // let body = client
    //     .post(&format!("https://api.mch.weixin.qq.com/v3/certificates"))
    //     .send()
    //     .await
    //     .map_err(es)?
    //     .body()
    //     .await
    //     .map_err(es)?;

    // log::info!("{}", std::str::from_utf8(&body).unwrap());

    Ok(1)
}
