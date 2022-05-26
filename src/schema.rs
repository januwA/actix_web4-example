table! {
    admin (id) {
        id -> Unsigned<Bigint>,
        username -> Varchar,
        password -> Varchar,
        realname -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        remark -> Nullable<Varchar>,
        is_superadmin -> Bool,
        is_active -> Bool,
        last_login -> Nullable<Datetime>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    admin_log (id) {
        id -> Unsigned<Bigint>,
        action_flag -> Unsigned<Tinyint>,
        action_msg -> Varchar,
        desc -> Varchar,
        admin_id -> Unsigned<Bigint>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    auth_group (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        desc -> Varchar,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    auth_permission (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        desc -> Varchar,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    books (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        price -> Decimal,
        user_id -> Unsigned<Bigint>,
        #[sql_name = "type"]
        type_ -> Unsigned<Tinyint>,
        stock -> Unsigned<Integer>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    m2m_admin_group (id) {
        id -> Unsigned<Bigint>,
        admin_id -> Unsigned<Bigint>,
        group_id -> Unsigned<Bigint>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    m2m_admin_permission (id) {
        id -> Unsigned<Bigint>,
        admin_id -> Unsigned<Bigint>,
        permission_id -> Unsigned<Bigint>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    m2m_group_permission (id) {
        id -> Unsigned<Bigint>,
        group_id -> Unsigned<Bigint>,
        permission_id -> Unsigned<Bigint>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    posts (id) {
        id -> Unsigned<Bigint>,
        title -> Varchar,
        content -> Varchar,
        user_id -> Unsigned<Bigint>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

table! {
    users (id) {
        id -> Unsigned<Bigint>,
        user_type -> Unsigned<Tinyint>,
        username -> Nullable<Varchar>,
        password -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        is_active -> Bool,
        last_login -> Nullable<Datetime>,
        create_at -> Datetime,
        update_at -> Datetime,
    }
}

joinable!(admin_log -> admin (admin_id));
joinable!(books -> users (user_id));
joinable!(m2m_admin_group -> admin (admin_id));
joinable!(m2m_admin_group -> auth_group (group_id));
joinable!(m2m_admin_permission -> admin (admin_id));
joinable!(m2m_admin_permission -> auth_permission (permission_id));
joinable!(m2m_group_permission -> auth_group (group_id));
joinable!(m2m_group_permission -> auth_permission (permission_id));
joinable!(posts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    admin,
    admin_log,
    auth_group,
    auth_permission,
    books,
    m2m_admin_group,
    m2m_admin_permission,
    m2m_group_permission,
    posts,
    users,
);
