/// 获取连接时设置时区，通常在创建时使用
#[macro_export]
macro_rules! conn {
    ( $x:expr ) => {{
        let mut conn = $x.get()?;
        diesel::sql_query("SET time_zone = '+8:00';").execute(&mut conn)?;
        conn
    }};
}

/// api返回的json结构
#[macro_export]
macro_rules! res_ok {
    ( $data:expr ) => {
        Ok::<actix_web::HttpResponse, actix_web::Error>(actix_web::HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": $data,
        })))
    };

    // 分页查询
    ( $data:expr, $total:expr/*,  $page:expr, $limit:expr*/ ) => {
        Ok::<actix_web::HttpResponse, actix_web::Error>(actix_web::HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": $data,
            "total": $total,
            // "current": $page,
            // "pageSize": $limit,
        })))
    };

    () => {
        Ok::<actix_web::HttpResponse, actix_web::Error>(actix_web::HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": null,
        })))
    };
}

#[macro_export]
macro_rules! res_err {
    ( $err:expr ) => {
        Err(ej($err).actix())
    };
}

/// 从$x中计算出分页数据
/// 查询参数page从1开始，但是数据库从0开始
#[macro_export]
macro_rules! get_limit_offset {
    ( $x:expr ) => {{
        let page = $x.page.unwrap();
        let limit = $x.limit.unwrap();
        let offset = (page - 1) * limit;
        (limit, offset)
    }};
}

#[macro_export]
macro_rules! dbg_sql {
    ( $x:expr ) => {{
        let dq = diesel::debug_query::<diesel::mysql::Mysql, _>(&$x).to_string();
        dbg!(dq);
    }};
}

/// 获取环境变量并解析类型，开发人员需要保证类型的正确性
#[macro_export]
macro_rules! get_env {
    ( $name:expr ) => {
        dotenv!($name)
    };
    ( $name:expr, $t:tt ) => {
        dotenv!($name).parse::<$t>().expect($name)
    };
}

/// 将u8与enum作比较
#[macro_export]
macro_rules! enum_eq {
    ( $v:expr, $enum:expr ) => {{
        ($enum as u8) == ($v as u8)
    }};
}

#[macro_export]
macro_rules! user {
    ( $db_pool:expr, $user_id:expr ) => {{
        let c_db_pool = $db_pool.clone();
        let res: models::user::User = web::block(move || -> anyhow::Result<_> {
            let mut conn = c_db_pool.get()?;
            Ok(schema::users::table.find($user_id).first(&mut conn)?)
        })
        .await?
        .map_err(ej)?;
        res
    }};
}

/// 手机 4位数字 验证码
#[macro_export]
macro_rules! make_phone_code {
    ( $len:expr ) => {{
        let mut rng = thread_rng();
        let distr = rand::distributions::Uniform::new_inclusive(48, 57); // ascii 数字 0-9
        let mut number_arr = [0u8; $len];
        for x in &mut number_arr {
            *x = rng.sample(distr);
        }
        std::str::from_utf8(&number_arr).unwrap().to_string()
    }};
}

/// 随机字符串
#[macro_export]
macro_rules! make_random_string {
    ( $len:expr ) => {{
        thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take($len)
            .map(char::from)
            .collect()
    }};
}

/// 加密密码，返回加密后的密码
#[macro_export]
macro_rules! pwd_encode {
    ( $password:expr ) => {{
        let secret = get_env!("PWD_SECRET");
        let config = argon2::Config::default();
        argon2::hash_encoded($password.as_ref(), secret.as_ref(), &config).unwrap()
    }};
}

/// 验证密码，返回 bool
#[macro_export]
macro_rules! pwd_decode {
    ( $encode_pwd:expr, $password:expr ) => {{
        let r = argon2::verify_encoded($encode_pwd, $password);
        match r {
            Ok(r) => r,
            _ => false,
        }
    }};
}

/// https://github.com/mitsuhiko/redis-rs/issues/353
macro_rules! redis_async_transaction {
    ($conn:expr, $keys:expr, $body:expr) => {
        loop {
            redis::cmd("WATCH")
                .arg($keys)
                .query_async($conn)
                .await
                .map_err(ej)?;

            if let Some(response) = $body {
                redis::cmd("UNWATCH").query_async($conn).await.map_err(ej)?;
                break response;
            }
        }
    };
}

/// 生成订单 64位无符号位
#[macro_export]
macro_rules! get_order_id {
    ( $con:expr, $key_prefix:expr ) => {{
        // 1. 生成时间戳(秒)
        let now = Local::now();
        let timestamp: u64 = now.timestamp() as u64;

        // 2. 生成序列号
        let key_prefix = $key_prefix; // 每个订单业务对应单独的key
        let date = &now.format("%Y:%m:%d").to_string(); // 每天都生成不同的key
        let key = &format!("{}:order_no:{}:{}", get_env!("APP_NAME"), key_prefix, date);

        let count: u64 = $con.incr(key, 1).await.map_err(ej)?;
        if count > 0xFF_FFFF {
            log::error!("创建订单失败，今日({})订单量超过 3字节限制", date);
        }

        // 高位存放时间戳，低位存放子增长计数
        // 4位unix时间戳最高存到 2106/2/7
        // 4位订单计数最大到 42亿订单
        let order: u64 = (timestamp << (4 * 8)) | count;

        // 在前面拼接时间
        let order_no = format!("{}{}", date, order);
        order_no
    }};
}

/// 时间戳秒
#[macro_export]
macro_rules! timestamp {
    () => {
        Local::now().timestamp()
    };
}

/// 时间戳毫秒
#[macro_export]
macro_rules! timestamp_ms {
    () => {
        Local::now().timestamp_millis()
    };
}

/// 从 redis 的消息id中，提取消息的创建时间
///
/// 1652067609636-0 - 1652067609636
///
/// 提取失败返回 0
#[macro_export]
macro_rules! redis_extract_create_ms {
    ($id:expr) => {{
        $id.split('-')
            .next()
            .unwrap_or_default()
            .parse::<i64>()
            .unwrap_or_default()
    }};
}

/// 七牛云上传后的hash值计算，避免上传重复文件
///
/// https://developer.qiniu.com/kodo/1231/appendix
#[macro_export]
macro_rules! qn_etag {
    ($file_bytes:expr) => {{
        // 4M: 4 * 1024 * 1024
        // 2^22 = 4M
        let m4 = 4_194_304;

        let file_byte_len = $file_bytes.len();

        if file_byte_len <= m4 {
            let mut file_sha1 = openssl::sha::sha1($file_bytes.as_ref()).to_vec();
            file_sha1.insert(0, 0x16);
            openssl::base64::encode_block(file_sha1.as_ref())
        } else {
            let mut begin: usize = 0;
            let mut end: usize = m4;
            let mut splits: Vec<&[u8]> = Vec::new();
            loop {
                splits.push(&$file_bytes[begin..end]);
                if end >= file_byte_len {
                    break;
                }
                begin = end;
                end += m4;
                if end > file_byte_len {
                    end = file_byte_len
                }
            }

            let sha1_vec = splits
                .iter()
                .map(|x| openssl::sha::sha1(x.as_ref()))
                .collect::<Vec<_>>();

            // 对所有的 sha1 值拼接后做二次 sha1，
            let mut file_sha1 = openssl::sha::sha1(sha1_vec.concat().as_ref()).to_vec();
            // 然后在二次 sha1 值前拼上单个字节，值为0x96
            file_sha1.insert(0, 0x96);
            // 对拼接好的21字节的二进制数据做url_safe_base64计算，所得结果即为ETag值
            openssl::base64::encode_block(file_sha1.as_ref())
        }
        // URL安全的Base64编码适用于以URL方式传递Base64编码结果的场景。
        // 该编码方式的基本过程是先将内容以Base64格式编码为字符串，然后检查该结果字符串，将字符串中的加号+换成中划线-，并且将斜杠/换成下划线_
        .replace("+", "-")
        .replace("/", "_")
    }};
}

/// MD5
#[macro_export]
macro_rules! md5 {
    ($data:expr) => {
        hex::encode(openssl::hash::hash(openssl::hash::MessageDigest::md5(), $data).unwrap())
    };
}

/// sha1
#[macro_export]
macro_rules! sha1 {
    ($data:expr) => {
        hex::encode(openssl::sha::sha1($data))
    };
}

/// 手机验证码 key
///
/// ab:vc:手机号
#[macro_export]
macro_rules! rk_phone_vc {
    ($phone:expr) => {
        format!("{}:vc:{}", get_env!("APP_NAME"), $phone)
    };
}

/// 微信 access_token
#[macro_export]
macro_rules! rk_wx_access_token {
    () => {
        concat!(env!("APP_NAME"), ":wx:access_token")
    };
}

/// 微信 jsapi_ticket
#[macro_export]
macro_rules! rk_wx_jsapi_ticket {
    () => {
        concat!(env!("APP_NAME"), ":wx:jsapi_ticket")
    };
}

/// 微信 用户信息
#[macro_export]
macro_rules! rk_wx_userinfo {
    ($pk:expr) => {
        format!("{}:wx:userinfo:{}", env!("APP_NAME"), $pk)
    };
}

/// 微信小程序 access_token
#[macro_export]
macro_rules! rk_wxmp_access_token {
    () => {
        concat!(env!("APP_NAME"), ":wxmp:access_token")
    };
}

/// 上传文件的MD5
#[macro_export]
macro_rules! rk_media_md5 {
    ($md5_hash: expr) => {
        format!("{}:media_md5:{}", env!("APP_NAME"), $md5_hash)
    };
}

/// 七牛云上传文件的ETag hash值
#[macro_export]
macro_rules! rk_qn_etag {
    ($etag: expr) => {
        format!("{}:qn_etag:{}", env!("APP_NAME"), $etag)
    };
}
