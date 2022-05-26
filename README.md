# actix-web4 example

## 安装 diesel_cli

https://diesel.rs/

需要提前安装好`mysql`

```sh
cargo install diesel_cli --no-default-features --features mysql

如果找不到 DATABASE_URL 手动添加环境变量

export DATABASE_URL=mysql://ab_db:123@localhost:3306/ab_db
```

## diesel

创建我们的数据库（如果它不存在的话）
```sh
diesel setup
```


```sh
# 创建一个迁移，使用它来管理我们的 schema
diesel migration generate init

# 执行迁移
diesel migration run

# 回滚迁移
diesel migration revert
```