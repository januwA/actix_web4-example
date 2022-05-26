use crate::prelude::*;
use askama::Template;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(chat_index).service(chat_room);
}

#[derive(Template)]
#[template(path = "chat/index.html")]
struct ChatIndex;

#[http_get("")]
async fn chat_index() -> HttpResult {
    let s = ChatIndex.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[derive(Template)]
#[template(path = "chat/room.html")]
struct ChatRoot {
    room_name: String,
}
#[http_get("/{room_name}")]
async fn chat_room(path: web::Path<String>) -> HttpResult {
    let s = ChatRoot {
        room_name: path.into_inner(),
    }
    .render()
    .unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
