// use diesel::{AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable};
// use diesel_async::RunQueryDsl;
// use uuid::Uuid;

// use crate::{schema::tokens, utils::database_utils::DbPoolConnection};

// #[derive(AsChangeset, Debug, Queryable)]
// #[diesel(table_name = tokens)]
// #[diesel(primary_key(id))]
// pub struct Token {
//     pub id: Uuid,
//     pub name: String,
//     pub symbol: String,
//     pub address: String,
//     pub coin_addr: Option<String>,
//     pub decimals: i32,
// }

// impl Token {
//     pub async fn get_tokens(conn: &mut DbPoolConnection<'_>) -> diesel::QueryResult<Vec<Self>> {
//         tokens::table.load::<Self>(conn).await
//     }
//     pub async fn get_token_by_symbol(
//         symbol: String,
//         conn: &mut DbPoolConnection<'_>,
//     ) -> diesel::QueryResult<Option<Self>> {
//         tokens::table
//             .filter(tokens::symbol.eq(symbol))
//             .select(tokens::all_columns)
//             .first::<Self>(conn)
//             .await
//             .optional()
//     }
// }
