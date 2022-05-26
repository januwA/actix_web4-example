use crate::prelude::*;
use schema::*;

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[diesel(belongs_to(models::auth_permission::Permission, foreign_key = permission_id))]
#[diesel(belongs_to(models::admin::Admin, foreign_key = admin_id))]
#[diesel(table_name = m2m_admin_permission)]
pub struct AdminPerm {
    pub id: PK,
    pub admin_id: PK,
    pub permission_id: PK,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}
