use crate::prelude::*;
use actix_web::web;
use schema::*;

pub enum UserType {
    Account = 1,
    Phone = 2,
}

impl UserType {
    pub fn account() -> u8 {
        UserType::Account as u8
    }
    pub fn phone() -> u8 {
        UserType::Phone as u8
    }
}

#[derive(Identifiable, Queryable, Serialize, Deserialize, Validate, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: PK,

    /// [UserType]
    pub user_type: u8,
    pub email: Option<String>,
    pub username: Option<String>,

    #[serde(skip_serializing)]
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub password: Option<String>,

    #[validate(custom(function = "vf::phone", message = "手机号格式错误"))]
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub is_active: bool,
    pub last_login: Option<NaiveDateTime>,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

impl User {
    /// 获取子账号列表，后台管理员
    pub async fn sub_account_list(db_pool: web::Data<DbPool>) -> R<Vec<User>> {
        let data: Vec<User> = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(users::table
                // .filter(users::is_staff.eq(true).and(users::is_superadmin.eq(false)))
                .load::<User>(&mut conn)?)
        })
        .await??;
        Ok(data)
    }

    /// 获取用户列表
    pub async fn list(db_pool: web::Data<DbPool>) -> R<Vec<User>> {
        let data: Vec<User> = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(users::table.load::<User>(&mut conn)?)
        })
        .await??;
        Ok(data)
    }

    /// 使用pk获取用户
    pub async fn obj(db_pool: web::Data<DbPool>, pk: PK) -> R<User> {
        let user: User = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(users::table
                .find(pk)
                .first(&mut conn)
                .map_err(|e| db_not_found_err(e, "未找到"))?)
        })
        .await??;
        Ok(user)
    }

    /// 使用pk删除用户
    pub async fn del(db_pool: web::Data<DbPool>, pk: PK) -> R<usize> {
        let rows: usize = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(diesel::delete(users::table)
                .filter(users::id.eq(pk))
                .execute(&mut conn)?)
        })
        .await??;
        Ok(rows)
    }

    /// 使用pk删除多个用户
    pub async fn del_many(db_pool: web::Data<DbPool>, pks: Vec<PK>) -> R<usize> {
        let rows: usize = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(diesel::delete(users::table)
                .filter(users::id.eq_any(pks))
                .execute(&mut conn)?)
        })
        .await??;
        Ok(rows)
    }

    /// 使用登陆账号查找
    pub async fn obj_with_username(db_pool: web::Data<DbPool>, username: String) -> R<User> {
        let user: User = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(users::table
                .filter(users::username.eq(&username))
                .first(&mut conn)
                .map_err(|e| db_not_found_err(e, "账号或密码错误！"))?)
        })
        .await??;
        Ok(user)
    }

    /// 使用手机号查找
    pub async fn obj_with_phone(db_pool: web::Data<DbPool>, phone: String) -> R<User> {
        let user: User = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(users::table
                .filter(users::phone.eq(&phone))
                .first(&mut conn)
                .map_err(|e| db_not_found_err(e, "未找到手机号！"))?)
        })
        .await??;
        Ok(user)
    }

    /// 更新最后登陆时间
    pub async fn update_last_login(db_pool: web::Data<DbPool>, pk: PK) -> R<()> {
        web::block(move || -> R<()> {
            let mut conn = conn!(db_pool);
            let rows: usize = diesel::update(users::table.find(pk))
                .set(users::last_login.eq(current_timestamp().nullable()))
                .execute(&mut conn)?;

            if rows == 0 {
                bail!("更新登陆时间失败");
            }

            Ok(())
        })
        .await??;
        Ok(())
    }

    /// 使用账号+密码创建一个用户
    pub async fn create_account_user(
        db_pool: web::Data<DbPool>,
        user_form: NewAccountUser,
    ) -> R<PK> {
        let new_id = web::block(move || -> R<PK> {
            let mut conn = conn!(&db_pool);

            diesel::insert_into(users::table)
                .values(user_form)
                .execute(&mut conn)
                .map_err(|e| db_unique_err(e, "注册失败"))?;
            let id: PK = diesel::dsl::select(last_insert_id()).get_result::<PK>(&mut conn)?;
            Ok(id)
        })
        .await??;
        Ok(new_id)
    }

    /// 使用手机号创建一个用户
    pub async fn create_phone_user(db_pool: web::Data<DbPool>, user_form: NewPhoneUser) -> R<PK> {
        let new_id = web::block(move || -> R<PK> {
            let mut conn = conn!(&db_pool);

            diesel::insert_into(users::table)
                .values(user_form)
                .execute(&mut conn)
                .map_err(|e| db_unique_err(e, "手机号已注册"))?;
            let id: PK = diesel::dsl::select(last_insert_id()).get_result::<PK>(&mut conn)?;
            Ok(id)
        })
        .await??;
        Ok(new_id)
    }
}

#[derive(AsChangeset, Serialize, Deserialize, Validate, Debug)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    #[validate(custom(function = "vf::email", message = "邮箱格式错误"))]
    pub email: Option<String>,

    pub username: Option<String>,

    #[validate(custom(function = "vf::phone", message = "手机号格式错误"))]
    pub phone: Option<String>,

    pub avatar: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Insertable, Deserialize, Validate, Debug)]
#[diesel(table_name = users)]
pub struct NewPhoneUser {
    #[serde(default = "UserType::phone")]
    pub user_type: u8,

    #[validate(custom(function = "vf::phone", message = "手机号格式错误"))]
    pub phone: String,

    pub avatar: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Insertable, Deserialize, Validate, Debug)]
#[diesel(table_name = users)]
pub struct NewAccountUser {
    #[serde(default = "UserType::account")]
    pub user_type: u8,

    pub username: String,

    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub password: String,

    pub avatar: Option<String>,
    pub is_active: Option<bool>,
}
