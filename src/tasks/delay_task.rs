use crate::prelude::*;
use redis::streams::{StreamId, StreamKey, StreamReadOptions, StreamReadReply};
use tokio::time::{sleep, Duration};

const KEY: &str = concat!(env!("APP_NAME"), ":task:delay");

/// redis stream 延迟任务，发送到这个队列任务将固定延迟10s后才执行
///
/// XADD delay_task * name 1
pub fn init(redis_pool: RedisPool) {
    tokio::spawn(async move {
        let mut con = redis_pool.get().await.unwrap();
        let opts = StreamReadOptions::default().count(1).block(5000);

        loop {
            // 延迟 10s 执行，这个值可以从redis中获取，便于配置
            let delay_ms = 10 * 1000;
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

                    if cur_ms - create_ms >= delay_ms {
                        // 执行任务， 发送到另一个队列执行任务，不要阻塞这个消费者
                        log::info!("key:{} | data:{:?}", KEY, map);

                        // 完成后删除
                        let _: usize = con.xdel(KEY, &[id]).await.unwrap();
                    }
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }
    });
}
