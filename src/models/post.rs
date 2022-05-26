use crate::prelude::*;
use schema::*;

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug)]
#[diesel(belongs_to(models::user::User, foreign_key = user_id))]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: PK,
    pub title: String,
    pub content: String,
    pub user_id: PK,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}
