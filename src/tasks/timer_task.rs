use crate::prelude::*;
use redis::streams::{StreamId, StreamKey, StreamReadOptions, StreamReadReply};
use tokio::time::{sleep, Duration};

const KEY: &str = concat!(env!("APP_NAME"), ":task:timer");

/// redis stream 重复任务，发送到这个队列任务将在指定的间隔时间执行
///
/// XADD timer_task * name 1
pub fn init(redis_pool: RedisPool) {
    tokio::spawn(async move {
        let mut con = redis_pool.get().await.unwrap();
        let opts = StreamReadOptions::default().count(1).block(5000);

        loop {
            // 间隔执行时间
            let timer_ms = 10 * 1000;

            let read_reply: StreamReadReply = match con.xread_options(&[KEY], &["0"], &opts).await {
                Ok(x) => x,
                Err(err) => {
                    log::error!("读取消息失败: {:?}", err);
                    continue;
                }
            };

            for StreamKey { key: _, ids } in read_reply.keys {
                for StreamId { id, map } in &ids {
                    let create_ms: i64 = redis_extract_create_ms!(id);
                    if create_ms == 0 {
                        log::error!("错误的消息id: {}", &id);
                        let _: usize = con.xdel(KEY, &[id]).await.unwrap();
                        continue;
                    }
                    let cur_ms: i64 = timestamp_ms!();

                    if cur_ms - create_ms >= timer_ms {
                        log::info!("key:{} | data:{:?}", KEY, map);
                        let _: usize = con.xdel(KEY, &[id]).await.unwrap();

                        let mut items: Vec<(&str, &str)> = Vec::new();
                        for (f, v) in map {
                            if let redis::Value::Data(bytes) = v {
                                items.push((f, std::str::from_utf8(bytes).unwrap()));
                            } else {
                                panic!("Weird data")
                            }
                        }
                        // 添加新消息，将数据全部拷贝到新消息
                        let _id: String = con.xadd(KEY, "*", &items).await.unwrap();
                    }
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }
    });
}
