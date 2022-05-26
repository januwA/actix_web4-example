/**
 * 读操作：有缓存读取缓存返回，否则查数据库，写入缓存后在返回数据
 *
 * 更新缓存：数据变动时，先操作数据库，然后在删除缓存(手动更新缓存工作量太大)，在下次查询时更新缓存
 *
 */
use crate::prelude::*;
use schema::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(transaction)
        .service(lua)
        .service(pipe)
        .service(hash)
        .service(cache_list)
        .service(test)
        // .service(xwxpay)
        .service(spike);
}

#[http_get("/test")]
pub async fn test(redis_pool: web::Data<RedisPool>) -> HttpResult {
    // let q = books::table.select(books::id).filter(books::id.ne(1));
    // dbg_sql!(q);

    let mut con = redis_pool.get().await.map_err(ej)?;
    con.incr("test_incr", 1).await.map_err(ej)?;

    res_ok!()
}

// #[http_get("/xwxpay")]
// pub async fn xwxpay(
//     redis_pool: web::Data<RedisPool>,
//     client: web::Data<awc::Client>,
// ) -> HttpResult {
//     wxpay::calculate_signature(&redis_pool, &client).await.map_err(ej)?;
//     res_ok!(1)
// }

/// 事务和乐观锁
#[http_get("/transaction")]
pub async fn transaction(redis_pool: web::Data<RedisPool>) -> HttpResult {
    let mut con = redis_pool.get().await.map_err(ej)?;

    let key = "transaction_key";
    let (new_val,): (isize,) = redis_async_transaction!(&mut con, &[key], {
        let old_val: isize = con.get(key).await.map_err(ej)?;
        redis::pipe()
            .atomic()
            .set(key, old_val + 1)
            .ignore()
            .get(key)
            .query_async(&mut con)
            .await
            .map_err(ej)?
    });
    res_ok!(new_val)
}

/// lua 脚本
#[http_get("/lua")]
pub async fn lua(redis_pool: web::Data<RedisPool>) -> HttpResult {
    let mut con = redis_pool.get().await.map_err(ej)?;

    let script = redis::Script::new(
        r"
        redis.call('set', KEYS[1], ARGV[1]);

        local a = redis.call('get', KEYS[1]);
        redis.call('set', KEYS[2], a + ARGV[2]);
    ",
    );
    script
        .key("hello")
        .arg(233)
        .key("hello2")
        .arg(45)
        .invoke_async(&mut con)
        .await
        .map_err(ej)?;
    res_ok!()
}

/// 管道
#[http_get("/pipe")]
pub async fn pipe(redis_pool: web::Data<RedisPool>) -> HttpResult {
    let mut con = redis_pool.get().await.map_err(ej)?;

    let (k1, k2): (i32, i32) = redis::pipe()
        .atomic()
        .set("key_1", 42)
        .ignore()
        .set("key_2", 43)
        .ignore()
        // .lpop("key_1", None).ignore()
        .get("key_1")
        .get("key_2")
        .query_async(&mut con)
        .await
        .map_err(ej)?;
    res_ok!((k1, k2))
}

#[derive(Debug, Serialize)]
struct UserInfo {
    username: String,
    age: u8,
    is_active: bool,
}

// 可以使用这个宏减少代码量: https://github.com/michaelvanstraten/redis-derive
impl redis::FromRedisValue for UserInfo {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        if let redis::Value::Bulk(bulk_data) = v {
            // log::info!("{:?}", bulk_data);
            // for values in bulk_data.chunks(2) {}
            let is_active: String = redis::from_redis_value(&bulk_data[5])?;
            return Ok(Self {
                username: redis::from_redis_value(&bulk_data[1])?,
                age: redis::from_redis_value(&bulk_data[3])?,
                is_active: is_active.parse::<bool>().unwrap_or(false),
            });
        }
        Err((redis::ErrorKind::TypeError, "missing dash").into())
    }
}
impl redis::ToRedisArgs for UserInfo {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(b"username");
        out.write_arg_fmt(&self.username);

        out.write_arg(b"age");
        out.write_arg_fmt(self.age);

        out.write_arg(b"is_active");
        out.write_arg_fmt(self.is_active);
    }
}
#[http_get("/hash")]
pub async fn hash(redis_pool: web::Data<RedisPool>) -> HttpResult {
    let mut con = redis_pool.get().await.map_err(ej)?;

    let key = &format!("user:{}", 1);

    // 有缓存直接返回
    let is_exists = con.exists(key).await.map_err(ej)?;
    if is_exists {
        let x: UserInfo = con.hgetall(key).await.map_err(ej)?;
        return res_ok!(x);
    }

    // 从数据库获取数据
    let x = UserInfo {
        username: "ajanuw".into(),
        age: 12,
        is_active: true,
    };

    // 写入缓存
    redis::cmd("HMSET")
        .arg(key)
        .arg(&x)
        .query_async(&mut con)
        .await
        .map_err(ej)?;

    res_ok!(x)
}

#[http_get("/cache_list")]
pub async fn cache_list(redis_pool: web::Data<RedisPool>) -> HttpResult {
    let mut con = redis_pool.get().await.map_err(ej)?;

    let key = format!("home:list");

    // 有缓存直接返回
    let is_exists = con.exists(&key).await.map_err(ej)?;
    if is_exists {
        let x: String = con.get(&key).await.map_err(ej)?;
        let r: serde_json::Value = serde_json::from_str(&x).map_err(ej)?;
        return res_ok!(r);
    }

    let x = vec![
        UserInfo {
            username: "ajanuw".into(),
            age: 12,
            is_active: true,
        },
        UserInfo {
            username: "admin".into(),
            age: 2,
            is_active: true,
        },
    ];

    // 数据json格式化后，在缓存
    con.set_ex(&key, serde_json::to_string(&x).unwrap(), 60usize)
        .await
        .map_err(ej)?;
    res_ok!(x)
}

#[derive(Debug)]
struct SpikeProduce {
    begin_time: chrono::DateTime<Local>,
    end_time: chrono::DateTime<Local>,
    count: u64,
}

/// 秒杀
/// 1. 库存限量
/// 2. 指定时间内才能购买
#[http_get("/spike")]
pub async fn spike(redis_pool: web::Data<RedisPool>) -> HttpResult {
    let _id = 1; // 秒杀商品id
    let mut con = redis_pool.get().await.map_err(ej)?;

    // 1. 查询商品
    let mut p = SpikeProduce {
        begin_time: Local
            .datetime_from_str("2012-10-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .map_err(ej)?,
        end_time: Local
            .datetime_from_str("2022-10-07 12:00:00", "%Y-%m-%d %H:%M:%S")
            .map_err(ej)?,
        count: 100,
    };
    log::info!("{:?}", p);
    // 2. 秒杀时间是否开始，是否结束
    let now = Local::now();
    if now < p.begin_time {
        return res_err!("尚未开始");
    }
    if now > p.end_time {
        return res_err!("已经结束");
    }

    // 3. 是否还有库存
    if p.count < 1 {
        return res_err!("库存不足");
    }
    // 4. 减少库存
    p.count -= 1;
    // 5. 创建订单，向数据库订单表写入订单数据
    let order = get_order_id!(con, "spike");
    // 6. 返回订单信息
    res_ok!(order)
}
