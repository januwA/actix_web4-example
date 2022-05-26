use crate::prelude::*;
use models::admin::Admin;

use schema::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list)
        .service(current)
        .service(obj)
        .service(create)
        .service(update)
        .service(change_pwd)
        .service(del);
}

/// 列表
#[http_get("list")]
pub async fn list(
    data: web::Query<ser::admin::ListQuery>,
    db_pool: web::Data<DbPool>,
) -> HttpResult {
    let data = data.into_inner();

    let mut q = admin::table
        .filter(admin::is_superadmin.eq(false))
        .into_boxed();

    if let Some(ref value) = data.admin.email {
        if !value.is_empty() {
            q = q.filter(admin::email.like(format!("%{}%", value)));
        }
    }

    if let Some(ref value) = data.admin.username {
        if !value.is_empty() {
            q = q.filter(admin::username.like(format!("%{}%", value)));
        }
    }

    if let Some(ref value) = data.admin.realname {
        if !value.is_empty() {
            q = q.filter(admin::realname.like(format!("%{}%", value)));
        }
    }

    if let Some(value) = data.admin.is_active {
        q = q.filter(admin::is_active.eq(value));
    }

    if let Some(ref value) = data.admin.remark {
        if !value.is_empty() {
            q = q.filter(admin::remark.like(format!("%{}%", value)));
        }
    }

    if let Some(ref value) = data.admin.last_login {
        if !value.is_empty() {
            let v: Vec<&str> = value.split('~').collect();
            match v.len() {
                1 => {
                    q =
                        q.filter(admin::last_login.eq(
                            NaiveDateTime::parse_from_str(v[0], "%Y-%m-%d %H:%M:%S").map_err(ej)?,
                        ));
                }
                2 => {
                    q = q.filter(admin::last_login.between(
                        NaiveDateTime::parse_from_str(v[0], "%Y-%m-%d %H:%M:%S").map_err(ej)?,
                        NaiveDateTime::parse_from_str(v[1], "%Y-%m-%d %H:%M:%S").map_err(ej)?,
                    ));
                }
                _ => (),
            }
        }
    }

    if data.limit.is_some() {
        let res = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            let (limit, offset) = get_limit_offset!(&data);

            // let total: i64 = admin::table.count().get_result(&mut conn)?;
            // let total: i64 = q.select(dsl::count_star()).first(&mut conn)?;
            let total: i64 = 0;

            q = q.limit(limit).offset(offset);
            let list = q.load::<Admin>(&mut conn)?;

            Ok((list, total))
        })
        .await
        .map_err(ej)?
        .map_err(ej)?;

        res_ok!(res.0, res.1)
    } else {
        let res = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            let list = q.load::<Admin>(&mut conn)?;
            Ok(list)
        })
        .await
        .map_err(ej)?
        .map_err(ej)?;
        res_ok!(res)
    }
}

/// 当前登陆admin信息
#[http_get("current")]
pub async fn current(
    db_pool: web::Data<DbPool>,
    claims: Option<web::ReqData<Claims>>,
) -> HttpResult {
    let claims = claims.ok_or("请求错误").map_err(ej)?;
    let res: Admin = Admin::obj(db_pool, claims.id).await.map_err(ej)?;
    res_ok!(res)
}

/// 使用id获取信息
#[http_get("get/{pk}")]
pub async fn obj(db_pool: web::Data<DbPool>, pk: web::Path<PK>) -> HttpResult {
    let res: Admin = Admin::obj(db_pool, pk.into_inner()).await.map_err(ej)?;
    res_ok!(res)
}

/// 创建一个非超级管理员账号
#[http_post("/create")]
pub async fn create(
    db_pool: web::Data<DbPool>,
    data: HttpBody<ser::admin::NewAdmin>,
) -> HttpResult {
    let mut data = data.into_inner();
    data.validate().map_err(OkErr::ev)?;
    data.password = pwd_encode!(&data.password);
    let new_id = Admin::create(db_pool, data).await.map_err(ej)?;
    res_ok!(new_id)
}

/// 批量删除
#[http_post("del")]
pub async fn del(
    db_pool: web::Data<DbPool>,
    web::Json(data): web::Json<ser::admin::DelMany>,
) -> HttpResult {
    let rows = Admin::del(db_pool, data.ids).await.map_err(ej)?;
    res_ok!(rows)
}

/// 当前登录admin修改密码
#[http_post("change_pwd")]
pub async fn change_pwd(
    db_pool: web::Data<DbPool>,
    claims: Option<web::ReqData<Claims>>,
    web::Json(data): web::Json<ser::ChangePwd>,
) -> HttpResult {
    data.validate().map_err(OkErr::ev)?;
    let pk = claims.ok_or("请先登录").map_err(ej)?.id;
    let rows = Admin::change_pwd(db_pool, pk, data.pwd, data.new_pwd, data.new_pwd2)
        .await
        .map_err(ej)?;
    res_ok!(rows)
}

/// 更新
#[http_post("update")]
pub async fn update(
    db_pool: web::Data<DbPool>,
    web::Json(data): web::Json<ser::admin::UpdateAdmin>,
) -> HttpResult {
    data.validate().map_err(OkErr::ev)?;
    let rows = Admin::update(db_pool, data).await.map_err(ej)?;
    res_ok!(rows)
}
