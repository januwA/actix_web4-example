use crate::prelude::*;
use askama::Template;
use deadpool_redis::Connection;
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use redis::streams::{StreamId, StreamKey, StreamReadOptions, StreamReadReply};
use tokio::{
    sync::mpsc::{self, Sender},
    time::{sleep, Duration},
};

/// 队列名
const KEY: &str = concat!(env!("APP_NAME"), ":queue:mail");

/// 消费者组名
const GROUP_NAME: &str = "g1";

/// 邮箱配置
const MAIL_FROM: &str = env!("MAIL_FROM");
const MAIL_USER: &str = env!("MAIL_USER");
const MAIL_PWD: &str = env!("MAIL_PWD");
const MAIL_HOST: &str = env!("MAIL_HOST");

#[derive(Debug)]
enum ErrMsg {
    /// 重试
    Retry {
        /// 消息id
        id: String,

        /// 消息的消费者名
        consumer_name: String,
    },
}

/// 不要传递过多的消息体，应该在消费时才进行组装
///
/// ```
/// let _id: String = con
/// .xadd(
///     key,
///     "*",
///     &[
///         ("to", "ajanuw1995@gmail.com"),
///         ("subject", "Happy new year"),
///         ("link", "http://192.168.17.131:7777"),
///     ],
/// )
/// .await
/// .map_err(ej)?;
/// ```
///
/// ```
/// XADD ab:mail * to ajanuw1995@gmail.com subject title link https://baidu.com
/// ```
pub fn init(redis_pool: RedisPool) {
    let redis_pool = Arc::new(redis_pool);

    // 创建一个最大容量为 32 的新通道
    let (tx, mut rx) = mpsc::channel::<ErrMsg>(32);

    for i in 1..=2 {
        let c_tx = tx.clone();
        let consumer_name = format!("c{}", i);
        let opts = StreamReadOptions::default()
            .group(GROUP_NAME, &consumer_name)
            .count(1)
            .block(5000);

        let c_redis_pool = Arc::clone(&redis_pool);

        // 处理最新的消息
        tokio::spawn(async move {
            let mut con = c_redis_pool.get().await.unwrap();
            let _: Result<(), _> = con.xgroup_create_mkstream(KEY, GROUP_NAME, "$").await;

            loop {
                let read_reply: StreamReadReply =
                    match con.xread_options(&[KEY], &[">"], &opts).await {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                if !read_reply.keys.is_empty() {
                    handle_msg(&mut con, read_reply, &consumer_name, &c_tx).await;
                }

                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    // 处理错误消息
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            use ErrMsg::Retry;
            match msg {
                Retry { id, consumer_name } => {
                    let mut con = redis_pool.get().await.unwrap();
                    log::info!("{} {}", id, consumer_name);

                    let pmsg: Vec<Vec<(String, String, i64, i64)>> = match con
                        .xpending_consumer_count(KEY, GROUP_NAME, &id, "+", 1, &consumer_name)
                        .await
                    {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    // pending-list 为空
                    if pmsg.is_empty() {
                        continue;
                    }

                    // <id> <consumer_name> <耗时毫秒> <消息被读取次数>
                    let (_, _, _, retry_count) = pmsg.first().unwrap().first().unwrap();

                    // 第一次消费和重试两次，加起来一共三次
                    if *retry_count >= 3 {
                        // TODO: 将多次不成功的消息发送到其他队列或存数据库手动处理或直接删除
                        log::error!("{}消费了{}次，强制ack", &id, retry_count);
                        let _: usize = con.xack(KEY, GROUP_NAME, &[id]).await.unwrap();
                        continue;
                    }

                    let create_ms: i64 = redis_extract_create_ms!(id);

                    if create_ms == 0 {
                        log::error!("错误的消息id: {}", &id);
                        let _: usize = con.xdel(KEY, &[id]).await.unwrap();
                        continue;
                    }

                    let opts = StreamReadOptions::default()
                        .group(GROUP_NAME, &consumer_name)
                        .count(1)
                        .block(5000);
                    let read_reply: StreamReadReply = match con
                        .xread_options(&[KEY], &[format!("{}-{}", create_ms - 1, create_ms)], &opts)
                        .await
                    {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                    if !read_reply.keys.is_empty() {
                        handle_msg(&mut con, read_reply, &consumer_name, &tx).await;
                    }
                }
            }
        }
    });
}

fn get_msg_args(msg: &StreamId) -> Result<(String, String, String), ()> {
    Ok((
        msg.get::<String>("to").ok_or(())?,
        msg.get::<String>("subject").ok_or(())?,
        msg.get::<String>("link").ok_or(())?,
    ))
}

#[derive(Template)]
#[template(path = "mail/hello.html")]

struct HelloTemplate {
    link: String,
}

async fn handle_msg(
    con: &mut Connection,
    read_reply: StreamReadReply,
    consumer_name: &str,
    tx: &Sender<ErrMsg>,
) {
    for StreamKey { key, ids } in read_reply.keys {
        for msg in &ids {
            let id = &msg.id;
            match get_msg_args(msg) {
                Err(_) => {
                    log::error!("消息参数错误，删除消息: {}", &id);
                    let _: usize = con.xdel(&key, &[id]).await.unwrap();
                    continue;
                }
                Ok((to, subject, link)) => {
                    let hello = HelloTemplate { link };
                    let html = hello.render().unwrap();

                    let email = match Message::builder()
                        .from(MAIL_FROM.parse().unwrap())
                        .reply_to(MAIL_FROM.parse().unwrap())
                        .to(to.parse().unwrap())
                        .subject(subject)
                        // .body(body) // 发送纯文本
                        .multipart(
                            MultiPart::alternative() // 他由两部分组成
                                .singlepart(
                                    SinglePart::builder()
                                        .header(header::ContentType::TEXT_PLAIN)
                                        .body(html.clone()), // 每条消息都应该有一个纯文本后备
                                )
                                .singlepart(
                                    SinglePart::builder()
                                        .header(header::ContentType::TEXT_HTML)
                                        .body(html),
                                ),
                        ) {
                        Ok(m) => m,
                        Err(err) => {
                            log::error!("Message::builder 错误: {} {}", &id, err);
                            tx.send(ErrMsg::Retry {
                                id: id.clone(),
                                consumer_name: consumer_name.into(),
                            })
                            .await
                            .unwrap();
                            continue;
                        }
                    };

                    // 打开到 mail 的远程连接
                    let mailer = match AsyncSmtpTransport::<Tokio1Executor>::relay(MAIL_HOST) {
                        Ok(m) => m,
                        Err(err) => {
                            log::error!("AsyncSmtpTransport::relay 错误: {} {}", &id, err);
                            tx.send(ErrMsg::Retry {
                                id: id.clone(),
                                consumer_name: consumer_name.into(),
                            })
                            .await
                            .unwrap();
                            continue;
                        }
                    }
                    .credentials(Credentials::from((MAIL_USER, MAIL_PWD)))
                    .build();

                    // 发送邮件
                    match mailer.send(email).await {
                        Ok(_) => {
                            log::info!("{} 邮件发送成功: {}", consumer_name, id);
                            let _: usize = con.xack(&key, GROUP_NAME, &[id]).await.unwrap();
                        }
                        Err(err) => {
                            tx.send(ErrMsg::Retry {
                                id: id.clone(),
                                consumer_name: consumer_name.into(),
                            })
                            .await
                            .unwrap();
                            log::error!("发送邮件失败: {:?}", err);
                        }
                    }
                }
            }
        }
    }
}
