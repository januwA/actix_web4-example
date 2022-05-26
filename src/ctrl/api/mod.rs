use crate::middleware::JWTAuth;
use actix_web::web;

pub mod admin;
pub mod auth;
pub mod book;
pub mod redis;
pub mod upload;
pub mod user;

#[cfg(feature = "wx")]
mod wx;

#[cfg(feature = "wxmp")]
mod wxmp;

#[cfg(feature = "alimp")]
mod alimp;

#[cfg(feature = "ali")]
mod ali;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::config));
    cfg.service(web::scope("/user").wrap(JWTAuth).configure(user::config));
    cfg.service(
        web::scope("/upload")
            .wrap(JWTAuth)
            .configure(upload::config),
    );
    cfg.service(web::scope("/book").configure(book::config));

    // redis 示例代码
    cfg.service(web::scope("/redis").configure(redis::config));

    cfg.service(web::scope("/admin").wrap(JWTAuth).configure(admin::config));

    #[cfg(feature = "wx")]
    cfg.service(web::scope("/wx").configure(wx::config));

    #[cfg(feature = "wxmp")]
    cfg.service(web::scope("/wxmp").configure(wxmp::config));

    #[cfg(feature = "alimp")]
    cfg.service(web::scope("/alimp").configure(alimp::config));

    #[cfg(feature = "ali")]
    cfg.service(web::scope("/ali").configure(ali::config));
}
