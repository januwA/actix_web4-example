use crate::prelude::*;
use schema::*;

pub enum ActionFlag {
    ADD = 1,    // 添加
    DEL = 2,    // 删除
    UPDATE = 3, // 更新
}

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name = admin_log)]
pub struct AdminLog {
    pub id: PK,
    pub sys_user_id: PK,
    pub action_flag: u8, // (*&ActionFlag::ADD as u8) == action_flag
    pub action_msg: String,
    pub desc: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}
