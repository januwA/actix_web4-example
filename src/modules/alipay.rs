extern crate openssl;
use crate::prelude::*;

use openssl::{
    base64, hash::MessageDigest, nid::Nid, pkey::PKey, rsa::Rsa, sign::Signer, x509::X509,
};

/// 支付宝商户id
const MERCHANT_PID: &str = env!("ALI_MERCHANT_PID");

/// 应用id
const APPID: &str = env!("ALI_APPID");

/// 支付宝网关地址
const GATEWAY_ADDR: &str = env!("ALI_GATEWAY_ADDR");

/// 接口内容加密方式
const CONTENT_ENCRYPT: &str = env!("ALI_CONTENT_ENCRYPT");

/// 应用公钥证书
const PUBLIC_KEY_CERT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    env!("ALI_PUBLIC_KEY_CERT"),
));

/// 应用私钥
const PRIVATE_KEY: &str = env!("ALI_PRIVATE_KEY");

/// 支付宝公钥（注：在公钥模式获取这个，证书模式不需要）
const ALIPAY_PUBLIC_KEY: &str = env!("ALI_ALIPAY_PUBLIC_KEY");

/// 支付宝公钥证书
const ALIPAY_PUBLIC_KEY_CERT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    env!("ALI_ALIPAY_PUBLIC_KEY_CERT"),
));

/// 支付宝根证书
const ALIPAY_ROOT_CERT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    env!("ALI_ALIPAY_ROOT_CERT"),
));

/// 从 X.509 公钥证书格式中，获取此证书的颁发者名称
///
/// issuer: CN=Ant Financial Certification Authority Class 2 R1 test,OU=Certification Authority test,O=Ant Financial test,C=CN
///
/// issuer: CN=Ant Financial Certification Authority R1,OU=Certification Authority,O=Ant Financial,C=CN
///
/// issuer: CN=iTrusChina Class 2 Root CA - G3,OU=China Trust Network,O=iTrusChina,C=CN
fn get_x509_issuer(x509: &X509) -> R<String> {
    Ok(x509
        .issuer_name()
        .entries()
        .filter_map(|value| {
            let key = value.object().nid().short_name().ok()?;
            let data = value.data().as_utf8().ok()?;
            Some(format!("{}={}", key, data))
        })
        .collect::<Vec<String>>()
        .into_iter()
        .rev() // 需要反转一下
        .collect::<Vec<String>>()
        .join(","))
}

/// 将证书的颁发者名称和证书的序列号拼接后,返回md5
fn get_cert_md5(content: &[u8]) -> R<String> {
    // SN 值是通过解析 X.509 证书文件中签发机构名称（name）以及内置序列号（serialNumber），将二者拼接后的字符串计算 MD5 值获取
    let ssl = X509::from_pem(content)?;

    // 返回此证书的颁发者名称
    let issuer = get_x509_issuer(&ssl)?;
    // log::info!("issuer: {}", issuer);

    // 返回此证书的序列号
    let serial_number = ssl.serial_number().to_bn()?.to_dec_str()?;

    // 拼接二者后获取MD5值
    Ok(md5!(format!("{}{}", issuer, serial_number).as_ref()))
}

pub struct AlipayConfig<'a> {
    pub public_params: HashMap<&'a str, String>,
}

impl<'a> Default for AlipayConfig<'a> {
    fn default() -> Self {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            public_params: HashMap::from([
                ("app_id", APPID.into()),
                ("method", "alipay.trade.page.pay".into()),
                ("charset", "UTF-8".into()),
                ("sign_type", "RSA2".into()),
                ("timestamp", timestamp),
                ("version", "1.0".into()),
            ]),
        }
    }
}

impl<'a> AlipayConfig<'a> {
    /// 请求参数的集合，最大长度不限，除公共参数外所有请求参数都必须放在这个参数中传递，具体参照各产品快速接入文档
    pub fn biz_content(&mut self, biz_content: &serde_json::Value) -> &mut AlipayConfig<'a> {
        self.public_params
            .insert("biz_content", biz_content.to_string());
        self
    }

    pub fn version(&mut self, version: &str) -> &mut AlipayConfig<'a> {
        self.public_params.insert("version", version.to_owned());
        self
    }

    pub fn method(&mut self, method: &str) -> &mut AlipayConfig<'a> {
        self.public_params.insert("method", method.to_owned());
        self
    }

    pub fn format(&mut self, format: &str) -> &mut AlipayConfig<'a> {
        self.public_params.insert("format", format.to_owned());
        self
    }

    pub fn charset(&mut self, charset: &str) -> &mut AlipayConfig<'a> {
        self.public_params.insert("charset", charset.to_owned());
        self
    }

    pub fn notify_url(&mut self, notify_url: &str) -> &mut AlipayConfig<'a> {
        self.public_params
            .insert("notify_url", notify_url.to_owned());
        self
    }

    pub fn return_url(&mut self, return_url: &str) -> &mut AlipayConfig<'a> {
        self.public_params
            .insert("return_url", return_url.to_owned());
        self
    }

    /// 使用 证书模式
    pub fn cert(&mut self) -> R<&mut AlipayConfig<'a>> {
        // 获取证书序列号
        let app_cert_sn = get_cert_md5(PUBLIC_KEY_CERT.as_ref())?;
        self.public_params.insert("app_cert_sn", app_cert_sn);

        // 获取根证书序列号
        let alipay_root_cert_sn: String = ALIPAY_ROOT_CERT
            .split_inclusive("-----END CERTIFICATE-----")
            .filter(|cert| {
                let ssl = X509::from_pem(cert.as_ref()).unwrap();
                // 返回证书的签名算法
                let algorithm = ssl.signature_algorithm().object().nid();
                algorithm == Nid::SHA256WITHRSAENCRYPTION || algorithm == Nid::SHA1WITHRSAENCRYPTION
            })
            .filter_map(|cert| get_cert_md5(cert.as_ref()).ok())
            .collect::<Vec<String>>()
            .join("_");
        self.public_params
            .insert("alipay_root_cert_sn", alipay_root_cert_sn);

        Ok(self)
    }

    /// 拼接 [public_params] 参数
    fn get_param_qs(&self) -> (String, String) {
        let mut sort_keys = self.public_params.keys().copied().collect::<Vec<_>>();
        sort_keys.sort();

        // qs_data 代签名字符串
        // qs_data_encode 将所有一级 key 的 value 值进行 encode 操作
        let (mut qs_data, qs_data_encode) = sort_keys.iter().fold(
            (String::new(), String::new()),
            |(mut acc, mut acc_encode), k| {
                let v = self.public_params.get(k).unwrap();
                acc.push_str(&format!("{}={}&", k, v));
                acc_encode.push_str(&format!("{}={}&", k, urlencoding::encode(v)));
                (acc, acc_encode)
            },
        );
        qs_data.pop();
        (qs_data, qs_data_encode)
    }

    /// rsa256签名
    fn rsa2_sign(&self, content: &[u8]) -> R<String> {
        // 解码 DER 编码的 PKCS#1 RSAPrivateKey 结构
        let cert_content = base64::decode_block(PRIVATE_KEY)?;
        let rsa = Rsa::private_key_from_der(cert_content.as_ref())?;

        // 创建一个包含 RSA 密钥的新 PKey
        let private_key = PKey::from_rsa(rsa)?;

        // Signer 一种计算数据加密签名的类型
        // 创建一个新的签名者
        let mut signer = Signer::new(MessageDigest::sha256(), &private_key)?;

        // 向签名者提供更多数据
        signer.update(content)?;

        // 获取签名
        let sign = base64::encode_block(signer.sign_to_vec()?.as_ref());
        Ok(sign)
    }

    /// 公钥方式
    pub fn build(&mut self) -> R<String> {
        // 拼接
        let (qs_data, mut qs_data_encode) = self.get_param_qs();
        // log::info!("qs_data: {}", &qs_data);

        // 调用签名函数
        let sign = self.rsa2_sign(qs_data.as_ref())?;

        // sign永远拼接在最后面
        qs_data_encode.push_str(&format!("sign={}", urlencoding::encode(&sign)));

        // 返回html表单，让前端渲染执行表单中的post发起请求
        let form_html_str = format!(
            r#"<form name="submit_form" method="post" action="{action}"><input type="submit" value="提交" style="display:none" ></form><script>document.forms[0].submit();</script>"#,
            action = format!("{}?{}", GATEWAY_ADDR, qs_data_encode)
        );
        Ok(form_html_str)

        // Ok(qs_data_encode)
    }
}

/// 统一收单下单并支付页面接口
///
/// PC场景下单并支付
///
/// https://opendocs.alipay.com/apis/028r8t?scene=22
pub async fn trade_page_pay(
    _redis_pool: &RedisPool,
    _clint: &awc::Client,
    biz_content: &serde_json::Value,
) -> R<String> {
    // let mut con = redis_pool.get().await?;

    let mut alipay_config = AlipayConfig::default();
    alipay_config
        .cert()?
        .biz_content(biz_content)
        .return_url("https://baidu.com");

    let form_html_str = alipay_config.build()?;

    // let body: web::Bytes = clint
    //     .post(GATEWAY_ADDR)
    //     .content_type("application/x-www-form-urlencoded;charset=utf-8")
    //     .send_body(form_html_str)
    //     .await
    //     .map_err(eany)?
    //     .body()
    //     .await
    //     .map_err(eany)?;

    Ok(form_html_str)
}
