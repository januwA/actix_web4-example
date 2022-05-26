use crate::prelude::*;
use schema::*;

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[diesel(belongs_to(models::auth_group::Group, foreign_key = group_id))]
#[diesel(belongs_to(models::auth_permission::Permission, foreign_key = permission_id))]
#[diesel(table_name = m2m_group_permission)]
pub struct GroupPermission {
    pub id: PK,
    pub group_id: PK,
    pub permission_id: PK,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

#[derive(Insertable, Validate, Deserialize, PartialEq, Debug)]
#[diesel(table_name = m2m_group_permission)]
pub struct GroupPermissionNew {
    pub group_id: PK,
    pub permission_id: PK,
}

#[derive(AsChangeset, Validate, Deserialize, PartialEq, Debug)]
#[diesel(table_name = m2m_group_permission)]
pub struct GroupPermissionUpdate {
    pub id: Option<PK>,
    pub group_id: Option<PK>,
    pub permission_id: Option<PK>,
    pub create_at: Option<NaiveDateTime>,
    pub update_at: Option<NaiveDateTime>,
}
