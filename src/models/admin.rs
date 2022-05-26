use crate::prelude::*;
use schema::*;

#[derive(Identifiable, Queryable, Serialize, Deserialize, Validate, Debug)]
#[diesel(table_name = admin)]
pub struct Admin {
    pub id: PK,
    pub username: String,

    #[serde(skip_serializing)]
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub password: String,

    pub realname: Option<String>,

    #[validate(custom(function = "vf::email", message = "邮箱格式错误"))]
    pub email: Option<String>,

    pub remark: Option<String>,
    pub is_superadmin: bool,
    pub is_active: bool,
    pub last_login: Option<NaiveDateTime>,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

impl Admin {
    pub async fn create(db_pool: web::Data<DbPool>, data: ser::admin::NewAdmin) -> R<PK> {
        let new_id = web::block(move || -> R<PK> {
            let mut conn = conn!(&db_pool);
            diesel::insert_into(admin::table)
                .values(data)
                .execute(&mut conn)
                .map_err(|e| db_unique_err(e, "注册失败"))?;
            let id: PK = diesel::dsl::select(last_insert_id()).get_result::<PK>(&mut conn)?;
            Ok(id)
        })
        .await??;
        Ok(new_id)
    }

    pub async fn obj(db_pool: web::Data<DbPool>, pk: PK) -> R<Admin> {
        let user: Admin = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(admin::table
                .find(pk)
                .first(&mut conn)
                .map_err(|e| db_not_found_err(e, "未找到"))?)
        })
        .await??;
        Ok(user)
    }

    pub async fn list(db_pool: web::Data<DbPool>) -> R<Vec<Admin>> {
        let data = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(admin::table
                .filter(admin::is_superadmin.eq(false))
                .load::<Admin>(&mut conn)?)
        })
        .await??;
        Ok(data)
    }

    /// 批量删除
    pub async fn del(db_pool: web::Data<DbPool>, pks: Vec<PK>) -> R<usize> {
        let rows: usize = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(diesel::delete(admin::table)
                .filter(admin::id.eq_any(pks))
                .execute(&mut conn)?)
        })
        .await??;
        Ok(rows)
    }

    pub async fn update(db_pool: web::Data<DbPool>, data: ser::admin::UpdateAdmin) -> R<usize> {
        let rows = web::block(move || -> R<usize> {
            let mut conn = db_pool.get()?;
            let target = admin::table.filter(admin::id.eq(data.id));
            let rows = diesel::update(target).set(&data).execute(&mut conn)?;

            if rows == 0 {
                bail!("更新失败!");
            }

            Ok(rows)
        })
        .await??;
        Ok(rows)
    }

    /// 修改密码
    pub async fn change_pwd(
        db_pool: web::Data<DbPool>,
        pk: PK,
        pwd: String,
        new_pwd: String,
        new_pwd2: String,
    ) -> R<usize> {
        if new_pwd != new_pwd2 {
            bail!("两次密码不一样");
        }

        // 验证旧密码
        let user = Admin::obj(db_pool.clone(), pk).await?;
        if !pwd_decode!(&user.password, pwd.as_ref()) {
            bail!("密码错误");
        }

        // 设置新密码
        let rows = web::block(move || -> R<usize> {
            let mut conn = db_pool.get()?;
            let new_password = pwd_encode!(new_pwd);
            let target = admin::table.filter(admin::id.eq(pk));
            Ok(diesel::update(target)
                .set(admin::password.eq(new_password))
                .execute(&mut conn)?)
        })
        .await??;

        Ok(rows)
    }

    /// 更新最后登陆时间
    pub async fn update_last_login(db_pool: web::Data<DbPool>, pk: PK) -> R<()> {
        web::block(move || -> R<()> {
            let mut conn = conn!(db_pool);
            let rows: usize = diesel::update(admin::table.find(pk))
                .set(admin::last_login.eq(current_timestamp().nullable()))
                .execute(&mut conn)?;
            if rows == 0 {
                bail!("更新登陆时间失败");
            }
            Ok(())
        })
        .await??;
        Ok(())
    }

    /// 使用登陆账号查找
    pub async fn obj_with_username(db_pool: web::Data<DbPool>, username: String) -> R<Self> {
        let user: Self = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(admin::table
                .filter(admin::username.eq(&username))
                .first(&mut conn)
                .map_err(|e| db_not_found_err(e, "账号或密码错误！"))?)
        })
        .await??;
        Ok(user)
    }

    pub async fn login(
        db_pool: web::Data<DbPool>,
        username: String,
        password: String,
    ) -> R<String> {
        let user: Self = Self::obj_with_username(db_pool.clone(), username).await?;

        if !pwd_decode!(&user.password, password.as_ref()) {
            bail!("账号或密码错误");
        }

        if !user.is_active {
            bail!("账号未激活");
        }

        Self::update_last_login(db_pool, user.id).await?;
        Ok(Claims::new(user.id, 0).jwt()?)
    }
}
