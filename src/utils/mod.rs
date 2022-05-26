pub mod jwt;
pub mod valid;

/// 自定义的接口返回错误
pub mod error;

pub mod init;

#[cfg(feature = "dev")]
/// GET的404请求重新处理
pub async fn all_try_files(
    req_method: actix_web::http::Method,
) -> actix_web::Result<impl actix_web::Responder> {
    match req_method {
        actix_web::http::Method::GET => {
            let file = actix_files::NamedFile::open("public/app/index.html")?
                .set_status_code(actix_web::http::StatusCode::OK);
            Ok(actix_web::Either::Left(file))
        }
        _ => Ok(actix_web::Either::Right(
            actix_web::HttpResponse::MethodNotAllowed().finish(),
        )),
    }
}
