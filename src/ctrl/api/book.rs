use crate::{
    models::book::{Book, BookNew, BookUpdate},
    prelude::*,
};
use schema::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(list)
        .service(list_all)
        .service(pay)
        .service(update)
        .service(del)
        .service(retrieve);
}

#[http_post("")]
pub async fn create(db_pool: web::Data<DbPool>, form: HttpBody<BookNew>) -> HttpResult {
    let form = form.into_inner();
    form.validate().map_err(OkErr::ev)?;

    res_ok!(Book::create(db_pool, form).await.map_err(ej)?)
}

#[http_get("{pk}")]
pub async fn retrieve(db_pool: web::Data<DbPool>, pk: web::Path<PK>) -> HttpResult {
    res_ok!(Book::obj(db_pool, pk.into_inner(),).await.map_err(ej)?)
}

#[http_post("update")]
pub async fn update(db_pool: web::Data<DbPool>, form: HttpBody<BookUpdate>) -> HttpResult {
    let form = form.into_inner();
    form.validate().map_err(OkErr::ev)?;

    res_ok!(Book::update(db_pool, form.id, form).await.map_err(ej)?)
}

#[http_post("{pk}/del")]
pub async fn del(db_pool: web::Data<DbPool>, pk: web::Path<PK>) -> HttpResult {
    res_ok!(Book::del(db_pool, pk.into_inner()).await.map_err(ej)?)
}

#[http_get("")]
pub async fn list(query: web::Query<ListRequest>, db_pool: web::Data<DbPool>) -> HttpResult {
    let query = query.into_inner();

    if query.page.is_none() || query.limit.is_none() {
        res_ok!(web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(books::table
                .order_by(books::price.desc())
                .load::<models::book::Book>(&mut conn)?)
        })
        .await
        .map_err(ej)?
        .map_err(ej)?)
    } else {
        res_ok!(web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            let (limit, offset) = get_limit_offset!(&query);
            let data = books::table
                .limit(limit)
                .offset(offset)
                .load::<models::book::Book>(&mut conn)?;
            let total: i64 = books::table.count().get_result(&mut conn)?;

            Ok((data, total))
        })
        .await
        .map_err(ej)?
        .map_err(ej)?)
    }
}

/// 正向链表查询
#[http_get("/all")]
pub async fn list_all(db_pool: web::Data<DbPool>) -> HttpResult {
    let res = web::block(move || -> R<_> {
        let mut conn = db_pool.get()?;
        let res = books::table
            .inner_join(users::table)
            .load::<(Book, models::user::User)>(&mut conn)
            .map(|x| x.into_iter().map(models::book::BookWithUser::from))?
            .collect::<Vec<_>>();
        Ok(res)
    })
    .await?
    .map_err(ej)?;
    res_ok!(res)
}

/// 购买
#[http_post("/pay")]
pub async fn pay(db_pool: web::Data<DbPool>, form: HttpBody<BookPay>) -> HttpResult {
    let form = form.into_inner();

    let res = web::block(move || -> Result<_, String> {
        let mut conn = db_pool.get().map_err(es)?;
        let rows = diesel::update(
            books::table.filter(books::id.eq(form.book_id).and(books::stock.ge(form.num))),
        )
        .set(books::stock.eq(books::stock - form.num))
        .execute(&mut conn)
        .map_err(es)?;

        if rows == 0 {
            Err("购买失败".into())
        } else {
            Ok("购买成功")
        }
    })
    .await?
    .map_err(ej)?;

    res_ok!(res)
}

#[derive(Debug, Deserialize)]
pub struct ListRequest {
    /// page，从1开始
    pub page: Option<i64>,
    /// limit 需要多少条数据
    pub limit: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct BookPay {
    pub book_id: PK,
    pub num: u32,
}
