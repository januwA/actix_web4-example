use crate::prelude::*;
use jsonwebtoken as jwt;
use jwt::errors::ErrorKind;

/// 生成token的payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub id: u64,
    pub user_type: u8,
    /// 必需（validate_exp 在验证中默认为 true）。 到期时间（作为 UTC 时间戳）
    pub exp: usize,
}

impl Claims {
    pub fn new(user_id: u64, user_type: u8) -> Self {
        let exp: usize = (Utc::now() + chrono::Duration::seconds(env!("JWT_EXP").parse().unwrap()))
            .timestamp() as usize;
        Self {
            id: user_id,
            user_type,
            exp,
        }
    }

    /// 创建 jwt
    pub fn jwt(&self) -> R<String> {
        Ok(jwt::encode(
            &jwt::Header::default(),
            self,
            &jwt::EncodingKey::from_secret(env!("JWT_SECRET").as_ref()),
        )?)
    }

    /// 解码验证 jwt token
    pub fn decode(token: &str) -> R<jwt::TokenData<Self>> {
        let validation = jwt::Validation::new(jwt::Algorithm::HS256);
        let token_data = jwt::decode::<Self>(
            token,
            &jwt::DecodingKey::from_secret(env!("JWT_SECRET").as_ref()),
            &validation,
        )
        .map_err(|err| {
            let err = match *err.kind() {
                ErrorKind::InvalidToken => "token 无效",
                ErrorKind::InvalidIssuer => "发行人无效",
                ErrorKind::ExpiredSignature => "登录过期",
                _ => "登录验证失败",
            };
            anyhow!(err)
        })?;
        Ok(token_data)
    }
}
