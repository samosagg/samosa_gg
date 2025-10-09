use std::sync::Arc;
use anyhow::Context;
use reqwest::Client;
use serde::{ Deserialize, Serialize, Deserializer };
use teloxide::{ prelude::*, types::{ InputFile, MessageEntityKind } };
use plotters::prelude::*;

use crate::{
    cache::{ Cache, ICache },
    db_models::users::User,
    utils::database_connection::get_db_connection,
};
use crate::telegram_bot::{ TelegramBot, commands::CommandProcessor };

// ---------------------- Custom Deserializers ------------------------
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

// ---------------------- Helper Functions ----------------------------
fn extract_entity_text(text: &str, offset: usize, length: usize) -> Option<String> {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    if offset + length <= utf16.len() {
        let slice = &utf16[offset..offset + length];
        String::from_utf16(slice).ok()
    } else {
        None
    }
}

fn extract_tagged_users(msg: &Message) -> Vec<(i64, Option<String>, String)> {
    let mut tagged_users = Vec::new();
    if let Some(entities) = msg.entities() {
        for entity in entities {
            match &entity.kind {
                MessageEntityKind::Mention => {
                    if let Some(text) = msg.text() {
                        if
                            let Some(username) = extract_entity_text(
                                text,
                                entity.offset,
                                entity.length
                            )
                        {
                            tagged_users.push((0, Some(username.clone()), username));
                        }
                    }
                }
                MessageEntityKind::TextMention { user } => {
                    let user_id = user.id.0 as i64;
                    let username = user.username.clone();
                    let display_name = user.full_name();
                    tagged_users.push((user_id, username.clone(), display_name.clone()));
                }
                _ => {}
            }
        }
    }
    tagged_users
}

// ---------------------- User Position Struct ------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    pub market: String,
    pub user: String,
    pub size: f64,
    pub user_leverage: u64,
    pub entry_price: f64,
    pub unrealized_funding: Option<f64>,
    pub estimated_liquidation_price: Option<f64>,
}

// ---------------------- /positions Command --------------------------
pub struct Positions;

#[async_trait::async_trait]
impl CommandProcessor for Positions {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message
    ) -> anyhow::Result<()> {
        let mut conn = get_db_connection(&cfg.pool).await.context("Failed to get DB connection")?;

        let tagged_users = extract_tagged_users(&msg);
        let reply_user = msg
            .reply_to_message()
            .and_then(|reply| reply.from())
            .map(|user| (user.id.0 as i64, user.username.clone(), user.full_name()));

        let args = msg.text();
        let parsed_args = if let Some(text) = args {
            text.split_whitespace().skip(1).collect::<Vec<&str>>()
        } else {
            vec![]
        };

        // ------------------ Determine wallet ------------------------
        // Determine wallet
        // ------------------ Determine wallet ------------------------
        // Determine wallet
        let wallet = if !parsed_args.is_empty() && !parsed_args[0].starts_with('@') {
            // Wallet provided as argument (and not a username)
            parsed_args[0].to_string()
        } else if let Some((_, Some(username), display_name)) = tagged_users.first() {
            // Tagged user - fetch secondary_wallet_address from DB
            let username_clean = username.trim_start_matches('@').to_string();
            tracing::info!("🔍 Searching for user with username: {}", username_clean);

            let maybe_existing_user = User::get_by_telegram_username(
                username_clean.clone(),
                &mut conn
            ).await?;

            if let Some(db_user) = maybe_existing_user {
                tracing::info!("✅ Found user in DB: {:?}", db_user);

                if let Some(wallet_addr) = db_user.secondary_wallet_address.clone() {
                    if !wallet_addr.is_empty() {
                        tracing::info!("✅ User {} has wallet: {}", username_clean, wallet_addr);
                        wallet_addr
                    } else {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                "⚠️ User {} ({}) is registered but has no wallet address set.",
                                display_name,
                                username_clean
                            )
                        ).await?;
                        return Ok(());
                    }
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "⚠️ User {} ({}) is registered but has no wallet address set.",
                            display_name,
                            username_clean
                        )
                    ).await?;
                    return Ok(());
                }
            } else {
                tracing::warn!("❌ User not found in database: {}", username_clean);
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "❌ User {} ({}) not found in database.\n\n💡 The user may need to register first.",
                        display_name,
                        username_clean
                    )
                ).await?;
                return Ok(());
            }
        } else {
            // No argument and no tagged user -> notify
            if let Some((user_id, username, display_name)) = reply_user {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "📋 Reply-to user detected:\n• User ID: {}\n• Username: {}\n• Name: {}\n\n💡 You need to look up this user's wallet in your database.",
                        user_id,
                        username.unwrap_or_default(),
                        display_name
                    )
                ).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ Please provide a wallet address, tag a user, or reply to a user's message.\nExample: /positions 0xabc123... or /positions @username"
                ).await?;
            }
            return Ok(()); // early return
        };

        // ------------------ Loading message ------------------------
        let loading = bot.send_message(
            msg.chat.id,
            format!("⏳ Fetching positions for `{}`...", wallet)
        ).await?;

        // ------------------ Fetch User Positions ------------------------
        let url = format!("{}/api/v1/user_positions?user={}", cfg.config.decibel_url, wallet);
        let client = Client::new();
        let positions: Vec<UserPosition> = match client.get(&url).send().await {
            Ok(resp) => {
                let text = resp.text().await.unwrap_or_default();
                match serde_json::from_str::<Vec<UserPosition>>(&text) {
                    Ok(data) => data,
                    Err(_) => {
                        bot.edit_message_text(
                            loading.chat.id,
                            loading.id,
                            "❌ Failed to parse response."
                        ).await?;
                        return Ok(());
                    }
                }
            }
            Err(_) => {
                bot.edit_message_text(
                    loading.chat.id,
                    loading.id,
                    "❌ Failed to fetch positions."
                ).await?;
                return Ok(());
            }
        };

        if positions.is_empty() {
            bot.edit_message_text(loading.chat.id, loading.id, "❌ No positions found.").await?;
            return Ok(());
        }

        let last_positions: Vec<UserPosition> = positions.iter().rev().take(5).cloned().collect();
        let _ = std::fs::create_dir_all("plotters");
        let path = format!("plotters/positions-{}.png", msg.id);
        let output_path = path.clone();
        let wallet_display = wallet.clone();

        // ------------------ Generate chart ------------------------
        tokio::task
            ::spawn_blocking(move || {
                fn truncate_text(s: &str, max_len: usize) -> String {
                    if s.len() > max_len {
                        format!("{}...{}", &s[..4], &s[s.len().saturating_sub(4)..])
                    } else {
                        s.to_string()
                    }
                }
                let root = BitMapBackend::new(&output_path, (
                    900,
                    140 + (last_positions.len() as u32) * 45,
                )).into_drawing_area();
                root.fill(&RGBColor(30, 30, 30)).unwrap();
                let font = ("monospace", 18).into_font().color(&WHITE);
                let wallet_short = truncate_text(&wallet_display, 16);
                root.draw_text(
                    &format!("Positions for {}", wallet_short),
                    &("monospace", 24).into_font().color(&WHITE),
                    (40, 30)
                ).unwrap();
                let headers = ["Market", "Size", "Entry", "Lev", "Liq", "Funding"];
                let mut x = 40;
                for h in headers.iter() {
                    root.draw_text(h, &font, (x, 80)).unwrap();
                    x += 140;
                }
                root.draw(&PathElement::new(vec![(40, 100), (840, 100)], &WHITE.mix(0.5))).unwrap();
                let mut y = 130;
                for pos in last_positions.iter() {
                    let row = [
                        truncate_text(&pos.market, 10),
                        truncate_text(&format!("{:.2}", pos.size), 8),
                        truncate_text(&format!("{:.2}", pos.entry_price), 8),
                        truncate_text(&format!("{}x", pos.user_leverage), 6),
                        truncate_text(
                            &pos.estimated_liquidation_price.map_or("-".to_string(), |v|
                                format!("{:.2}", v)
                            ),
                            8
                        ),
                        truncate_text(
                            &pos.unrealized_funding.map_or("-".to_string(), |v|
                                format!("{:.2}", v)
                            ),
                            8
                        ),
                    ];
                    let mut x = 40;
                    for value in row.iter() {
                        root.draw_text(value, &font, (x, y)).unwrap();
                        x += 140;
                    }
                    y += 45;
                }
                root.present().unwrap();
            }).await
            .unwrap_or_else(|err| tracing::error!("Task join error: {:?}", err));

        // ------------------ Send Image & Cleanup ------------------------
        bot.send_photo(msg.chat.id, InputFile::file(&path)).await?;
        let _ = tokio::fs::remove_file(&path).await;
        bot.delete_message(loading.chat.id, loading.id).await.ok();

        Ok(())
    }
}
