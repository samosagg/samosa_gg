// @generated automatically by Diesel CLI.

diesel::table! {
    subaccounts (id) {
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
        tg_id -> Nullable<Int8>,
        tg_username -> Nullable<Varchar>,
        #[max_length = 66]
        connected_wallet -> Nullable<Varchar>,
        #[max_length = 66]
        address -> Varchar,
        public_key -> Varchar,
        wallet_id -> Varchar,
        slippage -> Int8,
        degen_mode -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(subaccounts, users,);
