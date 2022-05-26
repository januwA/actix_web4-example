use crate::{
    models::{
        auth_group::Group,
        auth_permission::Permission,
        book::Book,
        m2m_admin_group::AdminGroup,
        m2m_admin_permission::AdminPerm,
        post::Post,
        user::{UpdateUser, User},
    },
    prelude::*,
    ser::user::Paging,
};

use schema::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list)
        .service(sub_account_list)
        .service(list_all)
        .service(current_user)
        .service(change_pwd)
        .service(del_many)
        .service(update_user);

    cfg.service(
        web::scope("/{pk}")
            .route(
                "",
                web::get().to(|db_pool: web::Data<DbPool>, pk: web::Path<PK>| async {
                    res_ok!(User::obj(db_pool, pk.into_inner()).await.map_err(ej)?)
                }),
            )
            .route(
                "del",
                web::post().to(|db_pool: web::Data<DbPool>, pk: web::Path<PK>| async {
                    let rows = User::del(db_pool, pk.into_inner()).await.map_err(ej)?;
                    res_ok!(rows)
                }),
            )
            .route(
                "update",
                web::post().to(
                    |db_pool: web::Data<DbPool>,
                     pk: web::Path<PK>,
                     web::Json(data): web::Json<UpdateUser>| async {
                        data.validate().map_err(OkErr::ev)?;
                        let rows = serv::user::update_user(db_pool, pk.into_inner(), data)
                            .await
                            .map_err(ej)?;
                        res_ok!(rows)
                    },
                ),
            ),
    );
}

#[http_get("current_user")]
pub async fn current_user(
    db_pool: web::Data<DbPool>,
    claims: Option<web::ReqData<Claims>>,
) -> HttpResult {
    let claims = claims.ok_or("请求错误").map_err(ej)?;
    let user = user!(db_pool, claims.id);
    res_ok!(user)
}

#[http_get("list")]
pub async fn list(
    data: web::Query<ser::user::UserListQuery>,
    db_pool: web::Data<DbPool>,
) -> HttpResult {
    let data = data.into_inner();

    if data.has_page() {
        let res = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            let (limit, offset) = get_limit_offset!(&data);

            // let mut q = users::table.into_boxed();
            let mut q = users::table.limit(limit).offset(offset).into_boxed();

            if let Some(email) = data.user.email {
                if !email.is_empty() {
                    q = q.filter(users::email.like(format!("%{}%", email)));
                }
            }

            let list = q.load::<User>(&mut conn)?;
            let total: i64 = users::table.count().get_result(&mut conn)?;

            Ok((list, total))
        })
        .await
        .map_err(ej)?
        .map_err(ej)?;

        res_ok!(res.0, res.1)
    } else {
        res_ok!(User::list(db_pool).await.map_err(ej)?)
    }
}

#[http_get("sub_account_list")]
pub async fn sub_account_list(
    data: web::Query<ser::user::UserListQuery>,
    db_pool: web::Data<DbPool>,
) -> HttpResult {
    let data = data.into_inner();

    if data.has_page() {
        let res = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            let (limit, offset) = get_limit_offset!(&data);

            // let mut q = users::table.into_boxed();
            let mut q = users::table.limit(limit).offset(offset).into_boxed();

            if let Some(email) = data.user.email {
                if !email.is_empty() {
                    q = q.filter(users::email.like(format!("%{}%", email)));
                }
            }

            // q = q.filter(users::is_staff.eq(true).and(users::is_superadmin.eq(false)));
            let list_data: Vec<User> = q.load::<User>(&mut conn)?;
            let total: i64 = users::table.count().get_result(&mut conn)?;

            Ok((list_data, total))
        })
        .await
        .map_err(ej)?
        .map_err(ej)?;

        res_ok!(res.0, res.1)
    } else {
        res_ok!(User::list(db_pool).await.map_err(ej)?)
    }
}

/// 反向查询
#[http_get("all")]
pub async fn list_all(db_pool: web::Data<DbPool>) -> HttpResult {
    res_ok!()
    // let res = web::block(move || -> Result<_, anyhow::Error> {
    //     let mut conn = db_pool.get()?;
    //     let user_set = users::table.load::<User>(&mut conn)?;

    //     let book_set = Book::belonging_to(&user_set)
    //         .load::<Book>(&mut conn)?
    //         .grouped_by(&user_set);

    //     let post_set = Post::belonging_to(&user_set)
    //         .load::<Post>(&mut conn)?
    //         .grouped_by(&user_set);

    //     let group_set: Vec<Vec<Group>> = AdminGroup::belonging_to(&user_set)
    //         .inner_join(schema::auth_group::table)
    //         .load::<(AdminGroup, Group)>(&mut conn)? // 这里需要返回 AdminGroup 用于 grouped_by
    //         .grouped_by(&user_set)
    //         .into_iter()
    //         .map(|x| x.into_iter().map(|(_, b)| b).collect::<_>())
    //         .collect();

    //     let perm_set: Vec<Vec<Permission>> = AdminPerm::belonging_to(&user_set)
    //         .inner_join(schema::auth_permission::table)
    //         .load::<(AdminPerm, Permission)>(&mut conn)?
    //         .grouped_by(&user_set)
    //         .into_iter()
    //         .map(|x| x.into_iter().map(|(_, b)| b).collect::<_>())
    //         .collect();

    //     let data = izip!(user_set, book_set, post_set, group_set, perm_set)
    //         .map(ser::user::UserWithAll::from)
    //         .collect::<Vec<_>>();
    //     Ok(data)
    // })
    // .await?
    // .map_err(ej)?;
    // res_ok!(res)
}

/// 更新当前登录用户的信息
#[http_post("update")]
pub async fn update_user(
    db_pool: web::Data<DbPool>,
    claims: Option<web::ReqData<Claims>>,
    web::Json(data): web::Json<UpdateUser>,
) -> HttpResult {
    data.validate().map_err(OkErr::ev)?;
    let pk = claims.unwrap().id;
    let rows = serv::user::update_user(db_pool, pk, data)
        .await
        .map_err(ej)?;
    res_ok!(rows)
}

/// 修改当前登录用户密码
#[http_post("change_pwd")]
pub async fn change_pwd(
    db_pool: web::Data<DbPool>,
    claims: Option<web::ReqData<Claims>>,
    web::Json(data): web::Json<ser::user::ChangePwd>,
) -> HttpResult {
    data.validate().map_err(OkErr::ev)?;

    if data.new_pwd != data.new_pwd2 {
        return res_err!("两次密码不一样");
    }

    let pk = claims.ok_or("请先登录").map_err(ej)?.id;

    let rows = serv::user::change_pwd(db_pool, pk, data.pwd, data.new_pwd)
        .await
        .map_err(ej)?;
    res_ok!(rows)
}

/// 批量删除
#[http_post("del_many")]
pub async fn del_many(
    db_pool: web::Data<DbPool>,
    web::Json(data): web::Json<ser::user::DelMany>,
) -> HttpResult {
    let rows = User::del_many(db_pool, data.ids).await.map_err(ej)?;
    res_ok!(rows)
}
