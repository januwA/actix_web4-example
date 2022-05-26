use crate::prelude::*;
pub use actix::prelude::*;
use actix_web_actors::ws;
use std::{sync::atomic::AtomicUsize, time::Instant};
mod manage;
mod server;

pub fn config(cfg: &mut web::ServiceConfig) {
    // 设置应用状态
    // 统计访问者的数量
    let app_state = Arc::new(AtomicUsize::new(0));

    // 开始聊天服务器 Actor
    let server = manage::ChatServer::new(app_state.clone()).start();

    cfg.app_data(web::Data::from(app_state))
        .app_data(web::Data::new(server))
        .service(chat_ws);
}

#[http_get("/chat")]
async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<manage::ChatServer>>,
) -> HttpResult {
    ws::start(
        server::ChatSocket {
            id: 0,
            room_name: None,
            hb: Instant::now(),
            cs: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
