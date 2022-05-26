use crate::prelude::*;
use schema::*;

/// 修改密码
pub async fn change_pwd(
    db_pool: web::Data<DbPool>,
    pk: PK,
    pwd: String,
    new_pwd: String,
) -> R<usize> {
    // 验证旧密码
    let user = models::admin::Admin::obj(db_pool.clone(), pk).await?;
    if !pwd_decode!(&user.password, pwd.as_ref()) {
        bail!("密码错误");
    }

    // 设置新密码
    let new_password = pwd_encode!(new_pwd);
    let rows = web::block(move || -> R<usize> {
        let mut conn = db_pool.get()?;
        let target = admin::table.filter(admin::id.eq(pk));
        Ok(diesel::update(target)
            .set(admin::password.eq(new_password))
            .execute(&mut conn)?)
    })
    .await??;

    Ok(rows)
}
