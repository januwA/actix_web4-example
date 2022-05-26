use crate::middleware::ContentLengthLimit;
use crate::prelude::*;
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::path::PathBuf;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

#[cfg(feature = "qiniu")]
use qiniu_sdk::upload::{
    apis::credential::Credential, AutoUploader, AutoUploaderObjectParams, UploadManager,
    UploadTokenSigner,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    #[cfg(feature = "qiniu")]
    cfg.service(upload_qiniu);

    // 对服务器上传文件进行大小限制
    cfg.service(
        web::scope("/local")
            .wrap(ContentLengthLimit::default())
            .service(upload_local),
    );
}

/// 单文件上传
#[http_post("")]
pub async fn upload_local(mut payload: Multipart, redis_pool: web::Data<RedisPool>) -> HttpResult {
    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "file" {
            // 提前准备好 5kb 的大小
            let mut chunk_vec: Vec<web::Bytes> = Vec::with_capacity(1024 * 5);
            while let Some(chunk) = field.try_next().await? {
                chunk_vec.push(chunk);
            }
            let file_byte_all = chunk_vec.concat();

            // 计算文件的md5值
            let mut con = redis_pool.get().await.map_err(ej)?;
            let md5_hash = md5!(file_byte_all.as_ref());

            let key = rk_media_md5!(md5_hash);

            let file_path: String = match con.get(&key).await.map_err(ej)? {
                Some(file_path) => file_path,
                _ => {
                    let content_type = match field.content_type().type_() {
                        mime::IMAGE => "image",
                        mime::VIDEO => "video",
                        _ => "file",
                    };
                    // 多部分/表单数据流必须包含 `content_disposition`
                    let content_disposition = field.content_disposition();

                    let err_file_msg = "错误的文件";
                    // 获取文件名
                    let filename = sanitize_filename::sanitize(
                        content_disposition
                            .get_filename()
                            .ok_or(err_file_msg)
                            .map_err(ej)?,
                    );

                    // 获取文件扩展名
                    let extension: &str = std::path::Path::new(&filename)
                        .extension()
                        .ok_or(err_file_msg)
                        .map_err(ej)?
                        .to_str()
                        .ok_or(err_file_msg)
                        .map_err(ej)?;

                    // 使用 uuid4 创建新的文件名
                    let filename = format!("{}.{}", Uuid::new_v4(), extension);

                    let now = Local::now();
                    let filepath = PathBuf::from(format!(
                        "public/media/{content_type}/{y}/{m}/{d}/{f}",
                        content_type = content_type,
                        y = now.year(),
                        m = now.month(),
                        d = now.day(),
                        f = &filename
                    ));

                    let save_dir = filepath.parent().unwrap();
                    fs::create_dir_all(&save_dir).await?; // 目录不存在递归创建
                    let mut f = File::create(&filepath).await?; // 创建文件

                    f.write_all(&file_byte_all).await?;

                    let file_path: String = filepath
                        .strip_prefix("public")
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    // 缓存 file_path
                    let _: () = con.set(&key, &file_path).await.map_err(ej)?;

                    file_path
                }
            };

            // 返回的时候拼接域名
            let media_uri = Url::parse(env!("SITE_NAME"))
                .unwrap()
                .join(&file_path)
                .unwrap()
                .to_string();
                
            return res_ok!(media_uri);
        }
    }

    res_err!("上传失败")
}

/// 上传到七牛云
#[cfg(feature = "qiniu")]
#[http_post("qiniu")]
pub async fn upload_qiniu(mut payload: Multipart, redis_pool: web::Data<RedisPool>) -> HttpResult {
    use std::time::Duration;

    let access_key = env!("QN_ACCESS_KEY");
    let secret_key = env!("QN_SECRET_KEY");
    let bucket_name = env!("QN_BUCKET_NAME");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "file" {
            // 收集文件字节
            let mut chunk_vec: Vec<web::Bytes> = Vec::with_capacity(1024 * 5);
            while let Some(chunk) = field.try_next().await? {
                chunk_vec.push(chunk);
            }
            let file_bytes = chunk_vec.concat();

            // 计算etag
            let etag = qn_etag!(&file_bytes);

            // 获取缓存
            let mut con = redis_pool.get().await.map_err(ej)?;
            let key = rk_qn_etag!(etag);
            let cache_file: Option<String> = con.get(&key).await.map_err(ej)?;

            let res: ser::upload::QuUploadResult = match cache_file {
                Some(res) => serde_json::from_str(&res).map_err(ej)?,
                _ => {
                    let content_disposition = field.content_disposition();

                    let content_type = match field.content_type().type_() {
                        mime::IMAGE => "image",
                        mime::VIDEO => "video",
                        _ => "file",
                    };

                    let filename_err = "错误的文件名";

                    // 获取文件名
                    let filename = sanitize_filename::sanitize(
                        content_disposition
                            .get_filename()
                            .ok_or(filename_err)
                            .map_err(ej)?,
                    );

                    // 获取文件扩展名
                    let extension: &str = std::path::Path::new(&filename)
                        .extension()
                        .ok_or(filename_err)
                        .map_err(ej)?
                        .to_str()
                        .ok_or(filename_err)
                        .map_err(ej)?;

                    // 使用 uuid4 创建新的文件名
                    let filename = format!("{}.{}", Uuid::new_v4(), extension);

                    let credential = Credential::new(access_key, secret_key);
                    let upload_manager = UploadManager::builder(
                        UploadTokenSigner::new_credential_provider_builder(
                            credential,
                            bucket_name,
                            Duration::from_secs(3600),
                        )
                        .on_policy_generated(|builder| {
                            builder.return_body(
                                json!({
                                "key": "$(key)",
                                "hash": "$(hash)",
                                // "w": "$(imageInfo.width)",
                                // "h": "$(imageInfo.height)",
                                })
                                .to_string(),
                            );
                        })
                        .build(),
                    )
                    .build();

                    let uploader: AutoUploader = upload_manager.auto_uploader();
                    let params = AutoUploaderObjectParams::builder()
                        .object_name(format!("{}/{}", content_type, filename))
                        .file_name(&filename)
                        .build();

                    // Cursor 包装内存缓冲区，并为其提供AsyncSeek实现
                    let buffer = futures_util::io::Cursor::new(file_bytes);

                    let res = uploader
                        .async_upload_reader(buffer, params)
                        .await
                        .map_err(ej)?;

                    let res: ser::upload::QuUploadResult =
                        serde_json::from_value(res).map_err(ej)?;

                    // 检查一下 etag 是否计算正确
                    if etag != res.hash {
                        return res_err!(format!("ETag 计算错误，预计{}，实际{}", etag, res.hash));
                    }

                    // 将结果缓存为json字符串
                    let res_string = serde_json::to_string(&res).map_err(ej)?;
                    let _: () = con.set(&key, res_string).await.map_err(ej)?;
                    res
                }
            };

            return res_ok!(res);
        }
    }

    res_err!("上传失败")
}
