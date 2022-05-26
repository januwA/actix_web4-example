use crate::prelude::*;
use actix_web::http::header::ContentType;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/echo")
            .route("", web::get().to(verification_message))
            .route("", web::post().to(receive_message)),
    )
    .service(test_pay);
}

async fn verification_message() -> HttpResult {
    res_ok!("")
}

async fn receive_message(body: web::Bytes) -> HttpResult {
    log::info!("{:#?}", &body);
    res_ok!("")
}

#[http_get("test_pay")]
async fn test_pay(redis_pool: web::Data<RedisPool>, client: web::Data<awc::Client>) -> HttpResult {
    use std::str::FromStr;
    let mut con = redis_pool.get().await.map_err(ej)?;

    let biz_content = json!({
        // 订单号，支付的订单号不能重复
        "out_trade_no": get_order_id!(con, "alipay"),
        "total_amount": BigDecimal::from_str("0.01").unwrap(),
        "subject":"测试商品名称",
        "product_code": "FAST_INSTANT_TRADE_PAY",
    });

    let form_html_str = alipay::trade_page_pay(&redis_pool, &client, &biz_content)
        .await
        .map_err(ej)?;

    // res_ok!(form_html_str)

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(form_html_str))
}
