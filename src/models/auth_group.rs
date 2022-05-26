use crate::prelude::*;
use schema::*;

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name = auth_group)]
pub struct Group {
    pub id: PK,
    pub name: String,
    pub desc: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

#[derive(Insertable, Validate, Deserialize, PartialEq, Debug)]
#[diesel(table_name = auth_group)]
pub struct GroupNew {
    pub name: String,
    pub desc: String,
}

#[derive(AsChangeset, Validate, Deserialize, PartialEq, Debug)]
#[diesel(table_name = auth_group)]
pub struct GroupUpdate {
    pub id: Option<PK>,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub create_at: Option<NaiveDateTime>,
    pub update_at: Option<NaiveDateTime>,
}
