use crate::prelude::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/echo")
            .route("", web::get().to(verification_message))
            .route("", web::post().to(receive_message)),
    );
}

async fn verification_message(req: HttpRequest) -> HttpResult {
  log::info!("{:#?}", &req);
    res_ok!("1")
    // let is_ok = wx::validation_message(&query);
    // if is_ok {
    //     return Ok(HttpResponse::Ok().body(query.echostr));
    // } else {
    //     return Ok(HttpResponse::Ok().body("false"));
    // }
}

async fn receive_message(body: web::Bytes) -> HttpResult {
    log::info!("{:#?}", &body);
    res_ok!("1")
}
