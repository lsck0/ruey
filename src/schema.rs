// @generated automatically by Diesel CLI.

diesel::table! {
    actions (id) {
        id -> Integer,
        name -> Text,
        script -> Binary,
        config -> Binary,
    }
}

diesel::table! {
    kv_store (bucket, key) {
        bucket -> Text,
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    settings (id) {
        id -> Integer,
        zoom_factor -> Nullable<Float>,
        tree -> Nullable<Text>,
        channel -> Nullable<Text>,
        user_access_token -> Nullable<Text>,
        user_refresh_token -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(actions, kv_store, settings,);
