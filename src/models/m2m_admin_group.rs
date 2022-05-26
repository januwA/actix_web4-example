use crate::prelude::*;
use schema::*;

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[diesel(belongs_to(models::auth_group::Group, foreign_key = group_id))]
#[diesel(belongs_to(models::admin::Admin, foreign_key = admin_id))]
#[diesel(table_name = m2m_admin_group)]
pub struct AdminGroup {
    pub id: PK,
    pub admin_id: PK,
    pub group_id: PK,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}
