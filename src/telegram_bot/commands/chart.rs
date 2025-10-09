use std::sync::Arc;

use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::{InputFile, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode}};

use crate::cache::{Cache, ICache};
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlestickResponse {
    #[serde(rename = "t")]
    pub open_time: u64,

    #[serde(rename = "T")]
    pub close_time: u64,

    #[serde(rename = "o")]
    pub open: f64,

    #[serde(rename = "h")]
    pub high: f64,

    #[serde(rename = "l")]
    pub low: f64,

    #[serde(rename = "c")]
    pub close: f64,

    #[serde(rename = "v")]
    pub volume: f64,

    #[serde(rename = "i")]
    pub interval: String,
}

pub struct Chart;

// Helper function to escape MarkdownV2 characters
fn escape_markdown_v2(text: &str) -> String {
    let reserved = r"_*[]()~`>#+-=|{}.!"; 
    let mut escaped = String::new();
    for c in text.chars() {
        if reserved.contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[async_trait::async_trait]
impl CommandProcessor for Chart {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let args = msg.text();

        // Extract arguments after the command
        let parsed_args = if let Some(text) = args {
            text.split_whitespace()
                .skip(1) // Skip the "/chart" part
                .collect::<Vec<&str>>()
        } else {
            bot.send_message(
                msg.chat.id,
                "⚠️ Usage: /chart <PAIR> <INTERVAL>\nExample: /chart APT/USD 1h",
            )
            .await?;
            return Ok(());
        };

        println!("Parsed args: {:?}", parsed_args);

        // Check if we have exactly 2 arguments
        if parsed_args.len() != 2 {
            let error_msg = if parsed_args.is_empty() {
                "⚠️ Missing arguments!\n\nUsage: /chart <PAIR> <INTERVAL>\nExample: /chart APT/USD 1h"
            } else if parsed_args.len() == 1 {
                "⚠️ Missing interval!\n\nUsage: /chart <PAIR> <INTERVAL>\nExample: /chart APT/USD 1h"
            } else {
                "⚠️ Too many arguments!\n\nUsage: /chart <PAIR> <INTERVAL>\nExample: /chart APT/USD 1h"
            };

            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }

        let pair = parsed_args[0].to_string();
        let interval = parsed_args[1].to_string();

        println!("Pair: {}, Interval: {}", pair, interval);

        // Send initial loading message
        let initial_message = bot
            .send_message(
                msg.chat.id,
                format!("Loading {} chart for {}...", interval, pair),
            )
            .await?;

        // Get market from cache
        let market = match cfg.cache.get_market(&pair).await {
            Some(m) => m,
            None => {
                bot.edit_message_text(
                    initial_message.chat.id,
                    initial_message.id,
                    "❌ Market not found",
                )
                .await?;
                return Ok(());
            }
        };

        // Fetch candlestick data
        let client = Client::new();
        let end = Utc::now().timestamp_millis();
        let start = end - 86400 * 1000; // 24 hours ago
        let url = format!(
            "{}/api/v1/candlesticks?market={}&interval={}&startTime={}&endTime={}",
            cfg.config.decibel_url, market.market_addr, interval, start, end
        );

        let response = client.get(url).send().await;

        let candles: Vec<CandlestickResponse> = match response {
            Ok(resp) => match resp.json::<Vec<CandlestickResponse>>().await {
                Ok(data) if !data.is_empty() => data,
                err => {
                    tracing::error!("{:#?}", err);
                    bot.edit_message_text(
                        initial_message.chat.id,
                        initial_message.id,
                        "❌ Failed to fetch chart data. Please try again later.",
                    )
                    .await?;
                    return Ok(());
                }
            },
            Err(err) => {
                tracing::error!("{}", err);
                bot.edit_message_text(
                    initial_message.chat.id,
                    initial_message.id,
                    "❌ Failed to fetch chart data. Please try again later.",
                )
                .await?;
                return Ok(());
            }
        };

        // Generate chart
        let path = format!("plotters/chart-{}-{}.png", interval, msg.id);
        let output_path = path.clone();
        let chart_candles = candles.clone();
        let chart_interval = interval.clone();
        let market_name = market.market_name.clone();

        tokio::task::spawn_blocking(move || {
            use plotters::prelude::*;

            let root = BitMapBackend::new(&output_path, (700, 500)).into_drawing_area();
            root.fill(&RGBColor(37, 37, 37)).unwrap();

            let (header, chart_area) = root.split_vertically(50);
            header
                .draw_text(
                    &market_name,
                    &("monospace", 24).into_font().color(&WHITE),
                    (640 - 40, 25),
                )
                .unwrap();

            let y_min = chart_candles
                .iter()
                .map(|c| c.low)
                .fold(f64::INFINITY, f64::min);
            let y_max = chart_candles
                .iter()
                .map(|c| c.high)
                .fold(f64::NEG_INFINITY, f64::max);
            let x_min = chart_candles.first().unwrap().open_time;
            let x_max = chart_candles.last().unwrap().open_time;

            let mut chart = ChartBuilder::on(&chart_area)
                .margin(20)
                .caption("", ("monospace", 20).into_font().color(&WHITE))
                .x_label_area_size(30)
                .right_y_label_area_size(40)
                .build_cartesian_2d(x_min..x_max, y_min..y_max)
                .unwrap();

            chart
                .configure_mesh()
                .x_labels(10)
                .x_label_formatter(
                    &(|ts| {
                        let ts_i64 = *ts as i64;
                        let dt = Utc.timestamp_millis_opt(ts_i64).single().unwrap();
                        match chart_interval.as_str() {
                            "1m" | "5m" | "15m" | "30m" => dt.format("%H:%M").to_string(),
                            "1h" | "2h" | "4h" | "8h" => dt.format("%H:%M").to_string(),
                            "1d" | "3d" | "1w" => dt.format("%m-%d").to_string(),
                            _ => dt.format("%H:%M").to_string(),
                        }
                    }),
                )
                .y_label_formatter(&(|v| format!("${:.2}", v)))
                .axis_style(&WHITE.mix(0.8))
                .x_label_style(("monospace", 12).into_font().color(&WHITE))
                .y_label_style(("monospace", 12).into_font().color(&WHITE))
                .set_all_tick_mark_size(3)
                .draw()
                .unwrap();

            chart
                .draw_series(chart_candles.iter().map(|candle| {
                    CandleStick::new(
                        candle.open_time,
                        candle.open,
                        candle.high,
                        candle.low,
                        candle.close,
                        RGBColor(0, 174, 0).filled(),
                        RGBColor(249, 70, 57).filled(),
                        10,
                    )
                }))
                .unwrap();

            chart_area.present().unwrap();
        })
        .await
        .unwrap_or_else(|err| tracing::error!("Task join error: {:?}", err));

        // Create inline keyboard
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback("1D".to_string(), "interval_1d".to_string()),
                InlineKeyboardButton::callback("1H".to_string(), "interval_1h".to_string()),
                InlineKeyboardButton::callback("4H".to_string(), "interval_4h".to_string()),
                InlineKeyboardButton::callback("8H".to_string(), "interval_8h".to_string()),
            ],
            vec![
                InlineKeyboardButton::callback("📈 Long".to_string(), "position_long".to_string()),
                InlineKeyboardButton::callback("📉 Short".to_string(), "position_short".to_string()),
            ],
        ]);

        // Escape pair and interval for MarkdownV2
        let caption = format!(
            "📊 *{} Chart \\({}\\)*\n\nChoose interval or position below:",
            escape_markdown_v2(&pair),
            escape_markdown_v2(&interval)
        );

        // Send chart image with buttons
        bot.send_photo(msg.chat.id, InputFile::file(&path))
            .caption(caption)
            .reply_markup(keyboard)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        // Cleanup temp file
        if let Err(e) = tokio::fs::remove_file(&path).await {
            tracing::warn!("Failed to delete chart file {}: {}", path, e);
        }

        Ok(())
    }
}
