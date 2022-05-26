use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::prelude::*;
pub use actix::prelude::*;
use rand::{prelude::ThreadRng, Rng};

/// 每个连接都有独立的ID
pub type SKID = usize;

/// 聊天服务器将此消息发送到会话
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// 用于聊天服务器通信的消息
/// 新的聊天会话已创建
#[derive(Message)]
#[rtype(SKID)]
pub struct Connect {
    pub id: SKID,
    pub addr: Recipient<Message>,
}

/// 会话已断开
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: SKID,
}

/// 向特定房间发送消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: SKID,          // 谁发的
    pub msg: String,       // 消息内容
    pub room_name: String, // 哪个房间
}

/// 当前可加入的房间列表
pub struct ListRooms;
impl actix::Message for ListRooms {
    type Result = Vec<String>; // 返回房间名列表
}

/// 加入房间，如果房间不存在，则创建新房间
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: SKID,
    pub room_name: String,
}

/// 管理连接进Chat中的socket连接
#[derive(Debug)]
pub struct ChatServer {
    users: HashMap<SKID, Recipient<Message>>, // 所有连接的socket，以及对等连接
    rooms: HashMap<String, HashSet<SKID>>,    // 所有房间，以及每个房间中有哪些用户
    visitor_count: Arc<AtomicUsize>,
    rng: ThreadRng,
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        ChatServer {
            users: HashMap::new(),
            rooms: HashMap::new(),
            rng: rand::thread_rng(),
            visitor_count,
        }
    }
}

impl ChatServer {
    /// 向房间内的所有用户发送消息
    fn send_message(&self, room_name: &str, message: &str, skip_id: Option<SKID>) {
        if let Some(users) = self.rooms.get(room_name) {
            if let Some(skip_id) = skip_id {
                for id in users {
                    if *id != skip_id {
                        if let Some(socket) = self.users.get(id) {
                            socket.do_send(Message(
                                json!({
                                  "mt": 3,
                                  "message":  message,
                                })
                                .to_string(),
                            ));
                        } else {
                            log::info!("没有获取到: {}", id);
                        }
                    }
                }
            } else {
                for id in users {
                    if let Some(socket) = self.users.get(id) {
                        socket.do_send(Message(
                            json!({
                              "mt": 3,
                              "message":  message,
                            })
                            .to_string(),
                        ));
                    } else {
                        log::info!("没有获取到: {}", id);
                    }
                }
            }
        }
    }
}

/// 从`ChatServer`制作actor
impl Actor for ChatServer {
    /// 我们将使用简单的上下文，我们只需要能够与其他参与者交流。
    type Context = Context<Self>;
}

/// 处理 Connect 消息
impl Handler<Connect> for ChatServer {
    type Result = SKID;
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // 使用随机 ID 注册会话
        let id = self.rng.gen::<usize>();

        // 保存链接
        self.users.insert(id, msg.addr);

        // 总链接数 +1
        self.visitor_count.fetch_add(1, Ordering::SeqCst);
        id
    }
}

/// 处理 Disconnect 消息
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("<{}> 断开连接", msg.id);

        let mut rooms: Vec<String> = Vec::new();

        // 删除地址
        if self.users.remove(&msg.id).is_some() {
            // 从所有房间中删除会话
            for (toom_name, users) in &mut self.rooms {
                if users.remove(&msg.id) {
                    rooms.push(toom_name.to_owned());
                }
            }
        }
        // 向其他用户发送消息
        for room_name in rooms {
            self.send_message(&room_name, &format!("<{}> 断开了", msg.id), None);
        }
    }
}

/// 处理 ClientMessage 消息
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        // 向房间内的其他人发送消息
        self.send_message(&msg.room_name, &msg.msg, None);
    }
}

/// 处理 ListRooms 消息
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();
        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }
        MessageResult(rooms)
    }
}

/// 加入房间，向旧房间发送断开连接消息
/// 向新房间发送加入消息
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, room_name } = msg;
        log::info!("<{}> 加入 {} 房间", &id, &room_name);

        let mut rooms = Vec::new();

        // 从所有房间中删除会话
        for (room_name, users) in &mut self.rooms {
            if users.remove(&id) {
                rooms.push(room_name.to_owned());
            }
        }

        // 向其他用户发送消息
        for room_name in rooms {
            self.send_message(&room_name, &format!("<{}> 断开了", id), None);
        }

        self.rooms
            .entry(room_name.clone()) // 获取房间
            .or_insert_with(HashSet::new) // 没有这个房间就创建
            .insert(id.to_owned()); // 加入用户

        log::info!("当前房间总数: {}", self.rooms.keys().len());
        self.send_message(&room_name, &format!("<{}> 加入", id), Some(id));
    }
}
