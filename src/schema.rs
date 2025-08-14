// @generated automatically by Diesel CLI.

diesel::table! {
    settings (id) {
        id -> Integer,
        channel -> Nullable<Text>,
        user_refresh_token -> Nullable<Text>,
        tree -> Nullable<Text>,
        zoom_factor -> Nullable<Float>,
    }
}
