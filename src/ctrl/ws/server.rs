use crate::prelude::*;
pub use actix::prelude::*;
use actix_web_actors::ws;

use std::time::{Duration, Instant};

use super::manage::{self, SKID};

/// 发送心跳 ping 的频率
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// 多久没有客户端响应导致超时
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// websocket 连接是长时间运行的连接，更容易与actor一起处理
pub struct ChatSocket {
    /// 唯一的会话 ID，通常是已登陆用户的user_id
    pub id: SKID,

    /// 当前加入的Chat房间
    pub room_name: Option<String>,

    /// 客户端必须至少每 10 秒发送一次 ping (CLIENT_TIMEOUT),否则我们断开连接。
    pub hb: Instant,

    /// 聊天服务器，管理所有的连接
    pub cs: Addr<manage::ChatServer>,
}

impl ChatSocket {
    /// 每秒向客户端发送 ping 的辅助方法
    ///
    /// 此方法还检查来自客户端的心跳
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        // 生成一个job，以指定的固定间隔定期执行给定的闭包
        // act是当前socket服务器，ctx是连接方
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // 检查客户端心跳
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // 心跳超时
                log::info!("Websocket客户端心跳失败，断开连接!");

                // stop actor
                ctx.stop();

                // 对等端断开时，向聊天服务器发送断开消息
                act.cs.do_send(manage::Disconnect {
                    id: act.id.to_owned(),
                });

                // 不要尝试发送 ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for ChatSocket {
    type Context = ws::WebsocketContext<Self>;

    /// 在 Actor 开始时调用方法
    fn started(&mut self, ctx: &mut Self::Context) {
        // 我们将在会话开始时启动心跳进程
        self.hb(ctx);

        // 在聊天服务器中注册自己。
        // `AsyncContext::wait` 在上下文中注册未来，但在处理任何其他事件之前，上下文会等到这个未来解决。
        // HttpContext::state() 是 WsChatSessionState 的实例，状态在应用程序内的所有路由之间共享
        let addr = ctx.address(); // 对等放地址
        self.cs
            .send(manage::Connect {
                id: self.id.to_owned(),
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.id = res;
                        log::info!("<{}> 连接ok", res)},
                    _ => {
                        log::info!("链接失败，聊天服务器有问题");
                        ctx.stop()
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // 通知聊天服务器
        self.cs.do_send(manage::Disconnect {
            id: self.id.to_owned(),
        });
        Running::Stop
    }
}

/// 处理来自聊天服务器的消息，我们只需将其发送到对等 websocket
impl Handler<manage::Message> for ChatSocket {
    type Result = ();

    fn handle(&mut self, msg: manage::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TextMeg {
    mt: u8, // 1获取所有房间,2加入房间,3用户向房间内其他用户发送消息
    message: String,
}

/// `ws::Message` 的处理程序
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // 处理 websocket 消息
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let msg: TextMeg = serde_json::from_str(&text).unwrap();
                let mt = msg.mt;
                match mt {
                    // 获取房间列表
                    1 => self
                        .cs
                        .send(manage::ListRooms)
                        .into_actor(self)
                        .then(move |res, _, ctx| {
                            match res {
                                Ok(rooms) => {
                                    ctx.text(json!({ "message": rooms, "mt": mt }).to_string());
                                }
                                _ => ctx.text(json!({ "message": "获取失败!!", "mt": 0 }).to_string()),
                            }
                            fut::ready(())
                        })
                        .wait(ctx),
                    // .wait(ctx) 暂停上下文中的所有事件，因此actor在获取房间列表之前不会收到任何新消息
                    2 => {
                        self.room_name = Some(msg.message.to_owned());
                        self.cs.do_send(manage::Join {
                            id: self.id.to_owned(),
                            room_name: msg.message,
                        });
                        ctx.text(json!({ "message": "加入成功", "mt": mt }).to_string());
                    }

                    3 => {
                        if let Some(room_name) = self.room_name.to_owned() {
                            self.cs.do_send(manage::ClientMessage {
                                id: self.id.to_owned(),
                                msg: msg.message.to_owned(),
                                room_name: room_name.to_owned(),
                            });
                        } else {
                            ctx.text(json!({ "message": "加入房间才能发送消息", "mt": 0 }).to_string());
                        }
                    }

                    _ => ctx.text(json!({ "message": "未知命令!!!" }).to_string()),
                }
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
