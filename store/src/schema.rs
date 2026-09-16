// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "WebsiteStatus"))]
    pub struct WebsiteStatus;
}

diesel::table! {
    Region (id) {
        id -> Text,
        name -> Text,
    }
}

diesel::table! {
    Website (id) {
        id -> Text,
        url -> Text,
        timeAdded -> Timestamp,
        user_id -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WebsiteStatus;

    WebsiteTick (id) {
        id -> Text,
        response_time_ms -> Int4,
        status -> WebsiteStatus,
        websiteId -> Text,
        regionId -> Text,
        createdAt -> Timestamp,
    }
}

diesel::table! {
    region (id) {
        id -> Text,
        name -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        email -> Text,
        password -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        password -> Text,
    }
}

diesel::table! {
    website (id) {
        id -> Text,
        url -> Text,
        time_added -> Timestamp,
        user_id -> Text,
        region_ids -> Array<Nullable<Text>>,
        poll_time -> Int8,
    }
}

diesel::table! {
    website_tick (id) {
        id -> Text,
        response_time_ms -> Int4,
        status -> Text,
        website_id -> Text,
        region_id -> Text,
        created_at -> Timestamp,
        status_code -> Nullable<Int4>,
        dns_time_ms -> Nullable<Int4>,
        tcp_time_ms -> Nullable<Int4>,
        tls_time_ms -> Nullable<Int4>,
        ttfb_ms -> Nullable<Int4>,
        response_size_bytes -> Nullable<Int8>,
        content_valid -> Nullable<Bool>,
        ssl_valid -> Nullable<Bool>,
        ssl_days_remaining -> Nullable<Int4>,
        error -> Nullable<Text>,
    }
}

diesel::joinable!(Website -> users (user_id));
diesel::joinable!(WebsiteTick -> Region (regionId));
diesel::joinable!(WebsiteTick -> Website (websiteId));
diesel::joinable!(website -> user (user_id));
diesel::joinable!(website_tick -> region (region_id));
diesel::joinable!(website_tick -> website (website_id));

diesel::allow_tables_to_appear_in_same_query!(
    Region,
    Website,
    WebsiteTick,
    region,
    user,
    users,
    website,
    website_tick,
);
