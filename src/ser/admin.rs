use crate::prelude::*;

#[derive(Deserialize, Debug)]
pub struct SearchAdmin {
    pub username: Option<String>,
    pub realname: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
    pub remark: Option<String>,
    pub last_login: Option<String>,
    pub create_at: Option<NaiveDateTime>,
    pub update_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// page，从1开始
    pub page: Option<i64>,
    /// limit 需要多少条数据
    pub limit: Option<i64>,

    #[serde(flatten)]
    pub admin: SearchAdmin,
}

#[derive(Debug, Deserialize)]
pub struct DelMany {
    pub ids: Vec<PK>,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Insertable, Deserialize, Validate, Debug)]
#[diesel(table_name = schema::admin)]
pub struct NewAdmin {
    pub username: String,

    #[serde(skip_serializing)]
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub password: String,

    pub realname: Option<String>,

    #[validate(custom(function = "vf::email", message = "邮箱格式错误"))]
    pub email: Option<String>,

    pub remark: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(AsChangeset, Deserialize, Validate, Debug)]
#[diesel(table_name = schema::admin)]
pub struct UpdateAdmin {
    pub id: PK,

    pub username: Option<String>,
    pub remark: Option<String>,
    pub realname: Option<String>,

    #[validate(custom(function = "vf::email", message = "邮箱格式错误"))]
    pub email: Option<String>,
    pub is_active: Option<bool>,
    pub last_login: Option<NaiveDateTime>,
    pub create_at: Option<NaiveDateTime>,
}
