use crate::prelude::*;

#[cfg(feature = "qiniu")]
#[derive(Debug, Serialize, Deserialize)]
pub struct QuUploadResult {
    pub hash: String,
    pub key: String,
    pub w: Option<String>,
    pub h: Option<String>,
}
