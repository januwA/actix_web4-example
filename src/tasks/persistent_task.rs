use crate::prelude::*;
use redis::streams::{StreamId, StreamKey, StreamReadOptions, StreamReadReply};
use tokio::time::{sleep, Duration};

const KEY: &str = concat!(env!("APP_NAME"), ":task:persistent");

/// redis stream 实时计时器，在指定时间执行队列中所有任务
///
///  XADD persistent_task * name 1
pub fn init(redis_pool: RedisPool) {
    tokio::spawn(async move {
        let opts = StreamReadOptions::default().count(0);
        loop {
            let cur_date = Local::now();

            // 每分钟
            if cur_date.second() == 0 {
                let mut con = redis_pool.get().await.unwrap();
                let read_reply: StreamReadReply =
                    con.xread_options(&[KEY], &["0"], &opts).await.unwrap();
                for StreamKey { key: _, ids } in read_reply.keys {
                    for StreamId { id, map } in &ids {
                        log::info!("{:?} | id:{} | data:{:?}", cur_date, id, map);
                    }
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }
    });
}
