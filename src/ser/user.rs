use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct DelMany {
    pub ids: Vec<PK>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePwd {
    /// 当前密码
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub pwd: String,

    /// 新密码
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub new_pwd: String,

    /// 确认新密码
    #[validate(length(min = 3, max = 20, message = "密码3到20位!"))]
    pub new_pwd2: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserWithAll<A, B, C, D, E> {
    #[serde(flatten)]
    pub user: A,
    pub books: B, // Vec<models::book::Book>,
    pub posts: C, // Vec<models::book::Post>,
    pub groups: D,
    pub perms: E,
}

impl<A, B, C, D, E> From<(A, B, C, D, E)> for UserWithAll<A, B, C, D, E> {
    fn from((user, books, posts, groups, perms): (A, B, C, D, E)) -> Self {
        Self {
            user,
            books,
            posts,
            groups,
            perms,
        }
    }
}

pub trait Paging {
    fn has_page(&self) -> bool;
}

#[derive(Deserialize, Debug)]
pub struct SearchUser {
    pub user_type: Option<u8>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub is_superadmin: Option<bool>,
    pub is_active: Option<bool>,
    pub is_staff: Option<bool>,
    pub last_login: Option<NaiveDateTime>,
    pub create_at: Option<NaiveDateTime>,
    pub update_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    /// page，从1开始
    pub page: Option<i64>,
    /// limit 需要多少条数据
    pub limit: Option<i64>,

    #[serde(flatten)]
    pub user: SearchUser,
}

impl Paging for UserListQuery {
    fn has_page(&self) -> bool {
        self.page.is_some() && self.limit.is_some()
    }
}
