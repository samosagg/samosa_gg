// @generated automatically by Diesel CLI.

diesel::table! {
    settings (id) {
        id -> Uuid,
        user_id -> Uuid,
        degen_mode -> Bool,
        notifications -> Bool,
    }
}

diesel::table! {
    sub_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 66]
        address -> Varchar,
        is_primary -> Nullable<Bool>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        wallet_id -> Varchar,
        #[max_length = 66]
        wallet_address -> Varchar,
        wallet_public_key -> Varchar,
        telegram_id -> Nullable<Int8>,
        telegram_username -> Nullable<Varchar>,
        #[max_length = 66]
        secondary_wallet_address -> Nullable<Varchar>,
    }
}

diesel::joinable!(sub_accounts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    settings,
    sub_accounts,
    users,
);
