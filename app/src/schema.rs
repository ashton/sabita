// @generated automatically by Diesel CLI.

diesel::table! {
    libraries (id) {
        id -> Text,
        name -> Text,
        external_id -> Nullable<Text>,
        folder -> Nullable<Text>,
        cover -> Nullable<Text>,
    }
}

diesel::table! {
    servers (id) {
        id -> Integer,
        name -> Text,
        server_type -> Text,
        url -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(libraries, servers,);
