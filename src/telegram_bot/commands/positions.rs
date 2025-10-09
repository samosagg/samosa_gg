use std::sync::Arc;
use chrono::Utc;
use reqwest::Client;
use serde::{ Deserialize, Serialize, Deserializer };
use teloxide::{ prelude::*, types::{ InputFile, MessageEntityKind } };
use plotters::prelude::*;

use crate::cache::{ Cache, ICache };
use crate::telegram_bot::{ TelegramBot, commands::CommandProcessor };

// ============================================================================
// ---------------------- Custom Deserializers --------------------------------
// ============================================================================

// Handles values that can be either a string or a number, returning them as `Option<String>`
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

// Handles numbers that might appear as int, float, or string (useful for inconsistent APIs)
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

// ============================================================================
// ---------------------- Helper Functions ------------------------------------
// ============================================================================

// Helper function to safely extract text from entities using UTF-16 encoding
fn extract_entity_text(text: &str, offset: usize, length: usize) -> Option<String> {
    // Convert byte string to UTF-16 for proper indexing (Telegram uses UTF-16)
    let utf16: Vec<u16> = text.encode_utf16().collect();
    
    if offset + length <= utf16.len() {
        let slice = &utf16[offset..offset + length];
        String::from_utf16(slice).ok()
    } else {
        None
    }
}

// Extract tagged user IDs and usernames from message
fn extract_tagged_users(msg: &Message) -> Vec<(i64, Option<String>, String)> {
    let mut tagged_users = Vec::new();
    
    if let Some(entities) = msg.entities() {
        for entity in entities {
            match &entity.kind {
                MessageEntityKind::Mention => {
                    // @username mention - extract username from text
                    if let Some(text) = msg.text() {
                        if let Some(username) = extract_entity_text(text, entity.offset, entity.length) {
                            tracing::info!("Found @mention: {}", username);
                            // Store with ID=0 as placeholder (can't get real ID from @mentions)
                            tagged_users.push((0, Some(username.clone()), username));
                        }
                    }
                }
                MessageEntityKind::TextMention { user } => {
                    // Direct user tag - we have the full user object with ID
                    let user_id = user.id.0 as i64;
                    let username = user.username.clone();
                    let display_name = user.full_name();
                    
                    tagged_users.push((user_id, username.clone(), display_name.clone()));
                    tracing::info!(
                        "Found tagged user - ID: {}, username: {:?}, name: {}", 
                        user_id, &username, &display_name
                    );
                }
                _ => {}
            }
        }
    }
    
    tagged_users
}

// ============================================================================
// ---------------------- User Position Struct --------------------------------
// ============================================================================

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

// ============================================================================
// ---------------------- /positions Command ----------------------------------
// ============================================================================

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
        
        // Log the full message for debugging
        tracing::info!("Message text: {:?}", msg.text());
        tracing::info!("Message from user: {:?}", msg.from().map(|u| (u.id, u.username.clone(), u.full_name())));
        
        // ------------------ Extract Tagged Users ------------------------
        let tagged_users = extract_tagged_users(&msg);
        
        if !tagged_users.is_empty() {
            tracing::info!("Found {} tagged users", tagged_users.len());
            for (user_id, username, display_name) in &tagged_users {
                tracing::info!(
                    "Tagged user - ID: {}, username: {:?}, name: {}", 
                    user_id, username, display_name
                );
            }
        }
        
        // Check for reply-to message
        let reply_user = msg.reply_to_message()
            .and_then(|reply| reply.from())
            .map(|user| {
                let user_id = user.id.0 as i64;
                let username = user.username.clone();
                let display_name = user.full_name();
                tracing::info!(
                    "Reply-to user - ID: {}, username: {:?}, name: {}",
                    user_id, username, display_name
                );
                (user_id, username, display_name)
            });

        // ------------------ Step 1: Parse Command Arguments -------------------
        let args = msg.text();

        // Extract wallet address from message text
        let parsed_args = if let Some(text) = args {
            text.split_whitespace().skip(1).collect::<Vec<&str>>()
        } else {
            bot.send_message(
                msg.chat.id,
                "⚠️ Usage: /positions <WALLET_ADDRESS> or tag a user\nExample: /positions 0xabc123..."
            ).await?;
            return Ok(());
        };

        // Determine wallet address source
        let wallet = if !parsed_args.is_empty() {
            // Wallet provided as argument
            parsed_args[0].to_string()
        } else if let Some((user_id, username, display_name)) = tagged_users.first() {
            // User was tagged - you need to look up their wallet from your database
            // For now, we'll show an example response
            bot.send_message(
                msg.chat.id,
                format!(
                    "📋 Tagged user detected:\n\
                    • User ID: {}\n\
                    • Username: {}\n\
                    • Name: {}\n\n\
                    💡 You need to look up this user's wallet in your database.\n\
                    Example: `cfg.cache.get_wallet_by_user_id({})`",
                    user_id,
                    username.as_ref().map(|s| s.as_str()).unwrap_or("(none)"),
                    display_name,
                    user_id
                )
            ).await?;
            return Ok(());
        } else if let Some((user_id, username, display_name)) = reply_user {
            // Reply-to message - look up wallet
            bot.send_message(
                msg.chat.id,
                format!(
                    "📋 Reply-to user detected:\n\
                    • User ID: {}\n\
                    • Username: {}\n\
                    • Name: {}\n\n\
                    💡 You need to look up this user's wallet in your database.",
                    user_id,
                    username.as_ref().map(|s| s.as_str()).unwrap_or("(none)"),
                    display_name
                )
            ).await?;
            return Ok(());
        } else {
            bot.send_message(
                msg.chat.id,
                "⚠️ Please provide a wallet address, tag a user, or reply to a user's message.\n\
                Example: /positions 0xabc123..."
            ).await?;
            return Ok(());
        };

        // ------------------ Step 2: Show Loading Message ----------------------
        let loading = bot.send_message(
            msg.chat.id,
            format!("⏳ Fetching positions for `{}`...", wallet)
        ).await?;

        // ------------------ Step 3: Fetch User Positions ----------------------
        let url = format!("{}/api/v1/user_positions?user={}", cfg.config.decibel_url, wallet);
        let client = Client::new();

        let positions: Vec<UserPosition> = match client.get(&url).send().await {
            Ok(resp) => {
                let text = resp.text().await.unwrap_or_default();

                match serde_json::from_str::<Vec<UserPosition>>(&text) {
                    Ok(data) => data,
                    Err(err) => {
                        tracing::error!("Failed to parse JSON: {}", err);
                        bot.edit_message_text(
                            loading.chat.id,
                            loading.id,
                            "❌ Failed to parse response."
                        ).await?;
                        return Ok(());
                    }
                }
            }
            Err(err) => {
                tracing::error!("Error fetching positions: {}", err);
                bot.edit_message_text(
                    loading.chat.id,
                    loading.id,
                    "❌ Failed to fetch positions."
                ).await?;
                return Ok(());
            }
        };

        // ------------------ Step 4: Handle Empty Data -------------------------
        if positions.is_empty() {
            bot.edit_message_text(loading.chat.id, loading.id, "❌ No positions found.").await?;
            return Ok(());
        }

        // ------------------ Step 5: Select Last 5 Positions -------------------
        let last_positions: Vec<UserPosition> = positions.iter().rev().take(5).cloned().collect();

        // ------------------ Step 6: Prepare Image Path ------------------------
        let _ = std::fs::create_dir_all("plotters"); // Ensure folder exists
        let path = format!("plotters/positions-{}.png", msg.id);
        let output_path = path.clone();
        let wallet_display = wallet.clone();

        // ------------------ Step 7: Generate Chart Image ----------------------
        tokio::task::spawn_blocking(move || {
            // Helper function to shorten long strings like wallet addresses
            fn truncate_text(s: &str, max_len: usize) -> String {
                if s.len() > max_len {
                    format!("{}...{}", &s[..4], &s[s.len().saturating_sub(4)..])
                } else {
                    s.to_string()
                }
            }

            // Create a drawing area
            let root = BitMapBackend::new(&output_path, (
                900,
                140 + (last_positions.len() as u32) * 45, // Adjust height dynamically
            )).into_drawing_area();
            root.fill(&RGBColor(30, 30, 30)).unwrap(); // Dark background

            let font = ("monospace", 18).into_font().color(&WHITE);

            // Header (with truncated wallet)
            let wallet_short = truncate_text(&wallet_display, 16);
            root.draw_text(
                &format!("Positions for {}", wallet_short),
                &("monospace", 24).into_font().color(&WHITE),
                (40, 30)
            ).unwrap();

            // Column Headers
            let headers = ["Market", "Size", "Entry", "Lev", "Liq", "Funding"];
            let mut x = 40;
            for h in headers.iter() {
                root.draw_text(h, &font, (x, 80)).unwrap();
                x += 140;
            }

            // Divider Line
            root.draw(&PathElement::new(vec![(40, 100), (840, 100)], &WHITE.mix(0.5))).unwrap();

            // Data Rows
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

                // Draw each column
                let mut x = 40;
                for value in row.iter() {
                    root.draw_text(value, &font, (x, y)).unwrap();
                    x += 140;
                }

                y += 45;
            }

            // Save the image
            root.present().unwrap();
        }).await
            .unwrap_or_else(|err| tracing::error!("Task join error: {:?}", err));

        // ------------------ Step 8: Send Image & Cleanup ----------------------
        bot.send_photo(msg.chat.id, InputFile::file(&path)).await?;

        // Delete temporary file after sending
        if let Err(e) = tokio::fs::remove_file(&path).await {
            tracing::warn!("Failed to delete file {}: {}", path, e);
        }

        // Remove the loading message
        bot.delete_message(loading.chat.id, loading.id).await.ok();

        Ok(())
    }
}