// @generated automatically by Diesel CLI.

diesel::table! {
    subaccounts (id) {
        id -> Uuid,
        wallet_id -> Uuid,
        #[max_length = 66]
        address -> Varchar,
        is_primary -> Nullable<Bool>,
    }
}

diesel::table! {
    tokens (id) {
        id -> Uuid,
        #[max_length = 30]
        name -> Varchar,
        #[max_length = 30]
        symbol -> Varchar,
        #[max_length = 66]
        address -> Varchar,
        coin_addr -> Nullable<Varchar>,
        decimals -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        telegram_id -> Nullable<Int8>,
        telegram_username -> Nullable<Varchar>,
        #[max_length = 66]
        secondary_wallet_address -> Nullable<Varchar>,
        degen_mode -> Bool,
        slippage -> Int4,
        #[max_length = 30]
        token -> Varchar,
    }
}

diesel::table! {
    wallets (id) {
        id -> Uuid,
        user_id -> Uuid,
        wallet_id -> Varchar,
        #[max_length = 66]
        address -> Varchar,
        public_key -> Varchar,
        is_primary -> Bool,
    }
}

diesel::joinable!(subaccounts -> wallets (wallet_id));
diesel::joinable!(wallets -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    subaccounts,
    tokens,
    users,
    wallets,
);
