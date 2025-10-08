use std::sync::Arc;
use chrono::{ TimeZone, Utc };
use reqwest::Client;
use serde::{ Deserialize, Serialize, Deserializer };
use teloxide::{ prelude::*, types::ParseMode };

use crate::cache::{ Cache, ICache };
use crate::telegram_bot::{ TelegramBot, commands::CommandProcessor };

// Custom deserializer to handle number or string
fn string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where D: Deserializer<'de>
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(serde_json::Number),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => Ok(Some(s)),
        Some(StringOrNumber::Number(n)) => Ok(Some(n.to_string())),
        None => Ok(None),
    }
}

// Custom deserializer to handle number from number or string
fn number_from_any<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where D: Deserializer<'de>
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Num {
        Int(i64),
        Float(f64),
        Str(String),
    }

    match Option::<Num>::deserialize(deserializer)? {
        Some(Num::Int(i)) => Ok(Some(i as f64)),
        Some(Num::Float(f)) => Ok(Some(f)),
        Some(Num::Str(s)) => Ok(s.parse::<f64>().ok()),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    pub market: String,
    pub user: String,
    pub size: f64,
    pub user_leverage: u64,
    pub max_allowed_leverage: Option<u64>,
    pub entry_price: f64,
    pub is_isolated: Option<bool>,
    pub is_deleted: Option<bool>,
    pub unrealized_funding: Option<f64>,
    #[serde(deserialize_with = "string_or_number")]
    pub event_uid: Option<String>,
    pub estimated_liquidation_price: Option<f64>,
    pub transaction_version: Option<i64>,
    pub tp_order_id: Option<String>,
    #[serde(deserialize_with = "number_from_any")]
    pub tp_trigger_price: Option<f64>,
    #[serde(deserialize_with = "number_from_any")]
    pub tp_limit_price: Option<f64>,
    pub sl_order_id: Option<String>,
    #[serde(deserialize_with = "number_from_any")]
    pub sl_trigger_price: Option<f64>,
    #[serde(deserialize_with = "number_from_any")]
    pub sl_limit_price: Option<f64>,
    pub has_fixed_sized_tpsls: Option<bool>,
}

pub struct Positions;

#[async_trait::async_trait]
impl CommandProcessor for Positions {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message
    ) -> anyhow::Result<()> {
        tracing::info!("Processing /positions command");
        let args = msg.text();

        let parsed_args = if let Some(text) = args {
            tracing::debug!("Command text: {}", text);
            text.split_whitespace().skip(1).collect::<Vec<&str>>()
        } else {
            tracing::warn!("No text in message");
            bot.send_message(
                msg.chat.id,
                "⚠️ Usage: /positions <WALLET_ADDRESS>\nExample: /positions 0xabc123..."
            ).await?;
            return Ok(());
        };

        if parsed_args.len() != 1 {
            tracing::warn!("Invalid args count");
            bot
                .send_message(
                    msg.chat.id,
                    "⚠️ Please provide your wallet address only\\.\nExample: /positions 0xabc123\\.\\.\\."
                )
                .parse_mode(ParseMode::MarkdownV2).await?;
            return Ok(());
        }

        let wallet = parsed_args[0].to_string();
        let escaped_wallet = escape_markdown_v2(&wallet);

        let loading = bot
            .send_message(
                msg.chat.id,
                format!("⏳ Fetching positions for `{}`\\.\\.\\.", escaped_wallet)
            )
            .parse_mode(ParseMode::MarkdownV2).await?;

        let url = format!("{}/api/v1/user_positions?user={}", cfg.config.decibel_url, wallet);
        let client = Client::new();

        let positions: Vec<UserPosition> = match client.get(&url).send().await {
            Ok(resp) => {
                let text = resp.text().await.unwrap_or_default();
                match serde_json::from_str::<Vec<UserPosition>>(&text) {
                    Ok(data) => data,
                    Err(err) => {
                        tracing::error!("Failed to parse JSON: {}", err);
                        bot
                            .edit_message_text(
                                loading.chat.id,
                                loading.id,
                                "❌ Failed to parse API response\\."
                            )
                            .parse_mode(ParseMode::MarkdownV2).await?;
                        return Ok(());
                    }
                }
            }
            Err(err) => {
                tracing::error!("Error fetching positions: {}", err);
                bot
                    .edit_message_text(
                        loading.chat.id,
                        loading.id,
                        "❌ Failed to fetch positions\\. Try again later\\."
                    )
                    .parse_mode(ParseMode::MarkdownV2).await?;
                return Ok(());
            }
        };

        if positions.is_empty() {
            bot
                .edit_message_text(loading.chat.id, loading.id, "❌ No positions found\\.")
                .parse_mode(ParseMode::MarkdownV2).await?;
            return Ok(());
        }

        let last_positions: Vec<_> = positions.iter().rev().take(4).collect();
        let mut message = format!("*Last Positions for* `{}`\n\n", escaped_wallet);

        for (i, pos) in last_positions.iter().enumerate() {
            let market_escaped = escape_markdown_v2(&pos.market);
            message.push_str(
                &format!(
                    "📊 *Position {}*\nMarket: {}\nSize: {}\nEntry Price: ${}\nLeverage: {}x\n",
                    i + 1,
                    market_escaped,
                    escape_number(pos.size),
                    escape_number(pos.entry_price),
                    pos.user_leverage
                )
            );
            if let Some(liq_price) = pos.estimated_liquidation_price {
                message.push_str(&format!("Est\\. Liquidation: ${}\n", escape_number(liq_price)));
            }
            if let Some(funding) = pos.unrealized_funding {
                message.push_str(&format!("Unrealized Funding: ${}\n", escape_number(funding)));
            }
            message.push_str("\n");
        }

        bot
            .edit_message_text(loading.chat.id, loading.id, message)
            .parse_mode(ParseMode::MarkdownV2).await?;

        Ok(())
    }
}

/// Escape special characters for MarkdownV2
fn escape_markdown_v2(text: &str) -> String {
    let chars_to_escape = [
        '_',
        '*',
        '[',
        ']',
        '(',
        ')',
        '~',
        '`',
        '>',
        '#',
        '+',
        '-',
        '=',
        '|',
        '{',
        '}',
        '.',
        '!',
    ];
    let mut result = String::new();
    for c in text.chars() {
        if chars_to_escape.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

/// Escape numbers for MarkdownV2 (handles decimals)
fn escape_number(num: f64) -> String {
    let formatted = format!("{:.2}", num);
    escape_markdown_v2(&formatted)
}
