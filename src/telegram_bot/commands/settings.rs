use std::sync::Arc;

use crate::{
    cache::Cache,
    models::db::users::User,
    telegram_bot::{TelegramBot, actions::UserAction, commands::CommandProcessor},
    utils::database_connection::get_db_connection,
};
use anyhow::Context;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

pub struct Settings;

#[async_trait::async_trait]
impl CommandProcessor for Settings {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let from = msg.from.context("From is missing in message")?;
        let chat_id = msg.chat.id;

        let tg_id = from.id.0 as i64;
        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Wallet not created yet. Type /start to create wallet")
            })?;

        let markup = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback(
                    "💸 Deposit to subaccount (Soon)",
                    "UserAction::DepositToSubaccount.to_string()",
                ),
                InlineKeyboardButton::callback("🏦 Withdraw to wallet (Soon)", "deposit"),
            ],
            vec![InlineKeyboardButton::callback(
                "📤 Withdraw to external wallet",
                UserAction::ExternalWithdraw.to_string(),
            )],
            vec![
                InlineKeyboardButton::callback(
                    "🔑 Export Private Key",
                    UserAction::ExportPk.to_string(),
                ),
                InlineKeyboardButton::callback("🌊 Slippage", UserAction::Slippage.to_string()),
            ],
            vec![InlineKeyboardButton::callback(
                format!(
                    "⚔️ Degen Mode [{}]",
                    if db_user.degen_mode { "ON" } else { "OFF" }
                ),
                UserAction::ChangeDegenMode {
                    user_id: db_user.id,
                    to: !db_user.degen_mode,
                }
                .to_string(),
            )],
            vec![InlineKeyboardButton::callback(
                "🔻 Close",
                UserAction::Cancel.to_string(),
            )],
        ]);
        bot.send_message(chat_id, "⚙️ Settings")
            .reply_markup(markup)
            .await?;
        Ok(())
    }
}
