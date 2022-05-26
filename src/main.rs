// 该属性用于隐藏对未使用代码的警告
#![allow(dead_code)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate dotenv_codegen;

#[macro_use]
pub mod macros;

use actix_web::{web, App, HttpServer};

mod ctrl;
mod middleware;
pub mod models;
mod modules;
mod prelude;
mod queues;
pub mod schema;
pub mod ser;
pub mod serv;
mod tasks;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    utils::init::env();
    env_logger::init_from_env(env_logger::Env::new());

    #[cfg(feature = "dev")]
    log::info!(
        "starting HTTP server at http://{}:{}",
        local_ipaddress::get().unwrap(),
        env!("ACTIX_PORT")
    );
    let redis_pool = utils::init::redis_pool();
    let mysql_pool = utils::init::mysql_pool();

    #[cfg(feature = "mail")]
    queues::mail::init(redis_pool.clone());

    tasks::delay_task::init(redis_pool.clone());
    tasks::timer_task::init(redis_pool.clone());
    tasks::persistent_task::init(redis_pool.clone());

    let serve = HttpServer::new(move || {
        let app = App::new();

        #[cfg(feature = "dev")]
        let app = app
            .service(actix_files::Files::new(env!("STATIC_URL"), "public/static"))
            .service(actix_files::Files::new(env!("MEDIA_URL"), "public/media"))
            .service(actix_files::Files::new("/admin", "public/admin").index_file("index.html"));

        let app = app.app_data(web::Data::new(awc::Client::default()));
        let app = app.app_data(web::Data::new(redis_pool.clone()));
        let app = app.app_data(web::Data::new(mysql_pool.clone()));

        #[cfg(feature = "cors")]
        let app = app.wrap(actix_cors::Cors::default().allowed_origin_fn(|_, _| true));

        let app = app
            // .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default());

        #[cfg(feature = "template")]
        let app = app.service(web::scope("/chat").configure(ctrl::chat::config));

        #[cfg(feature = "ws")]
        let app = app.service(web::scope("/ws").configure(ctrl::ws::config));

        let app = app.service(web::scope("/api").configure(ctrl::api::config));

        // 根路径应始终定义为最后一项
        #[cfg(feature = "dev")]
        let app = app.service(actix_files::Files::new("", "public/app").index_file("index.html"));
        // let app = app.default_service(web::to(utils::all_try_files));

        app
    });

    if cfg!(feature = "dev") {
        let port = get_env!("ACTIX_PORT", u16);
        serve.bind(("0.0.0.0", port))?
    } else {
        let port = get_env!("ACTIX_PORT", u16);
        serve.bind(("0.0.0.0", port))?

        // // 断掉服务文件会被自动删除，若文件存在则会启动失败
        // let p = format!("/tmp/{}.socket", env!("APP_NAME"));
        // let r = serve.bind_uds(&p)?;

        // // 设置权限，不然nginx没有权限访问
        // let mut perms = std::fs::metadata(&p)?.permissions();
        // perms.set_readonly(false);
        // std::fs::set_permissions(&p, perms)?;

        // r
    }
    .run()
    .await
}
