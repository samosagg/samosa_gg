// @generated automatically by Diesel CLI.

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
    }
}
