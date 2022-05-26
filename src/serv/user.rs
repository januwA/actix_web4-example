use crate::{models::user::UpdateUser, prelude::*, schema::*};

/// 修改用户信息
pub async fn update_user(db_pool: web::Data<DbPool>, pk: PK, data: UpdateUser) -> R<usize> {
    let rows = web::block(move || -> R<usize> {
        let mut conn = db_pool.get()?;
        let target = users::table.filter(users::id.eq(pk));
        Ok(diesel::update(target).set(&data).execute(&mut conn)?)
    })
    .await??;

    Ok(rows)
}

/// 修改用户密码
pub async fn change_pwd(
    db_pool: web::Data<DbPool>,
    pk: PK,
    pwd: String,
    new_pwd: String,
) -> R<usize> {
    // 验证旧密码
    let user = models::user::User::obj(db_pool.clone(), pk).await?;
    if !pwd_decode!(&user.password.unwrap_or_default(), pwd.as_ref()) {
        bail!("密码错误");
    }

    // 设置新密码
    let new_password = pwd_encode!(new_pwd);
    let rows = web::block(move || -> R<usize> {
        let mut conn = db_pool.get()?;
        let target = users::table.filter(users::id.eq(pk));
        Ok(diesel::update(target)
            .set(users::password.eq(new_password))
            .execute(&mut conn)?)
    })
    .await??;

    Ok(rows)
}
