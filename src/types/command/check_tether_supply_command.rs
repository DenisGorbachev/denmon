use clap::Parser;
use derive_more::Error;
use fmt_derive::Display;
use ntfy::{Payload, dispatcher};
use numfmt::Precision;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::num::ParseFloatError;
use url_macro::url;

const TRANSPARENCY_URL: &str = "https://app.tether.to/transparency.json";

// TODO: Remove this macro in favor of error_handling::handle
macro_rules! handle {
    ($expr:expr, $map:expr) => {{
        match $expr {
            Ok(value) => value,
            Err(source) => {
                return Err($map(source));
            }
        }
    }};
}

#[derive(Parser, Clone, Debug)]
pub struct CheckTetherSupplyCommand {
    #[arg(short, long, env = "DENMON_NTFY_TOPIC")]
    ntfy_topic: String,

    #[arg(short, long)]
    supply_min: f64,
}

impl CheckTetherSupplyCommand {
    pub async fn run(self) -> Result<(), CheckTetherSupplyCommandRunError> {
        use CheckTetherSupplyCommandRunError::*;

        let Self {
            ntfy_topic,
            supply_min,
        } = self;

        let supply = fetch_usdt_supply().await?;
        let mut formatter = regular_formatter();
        let supply_formatted = formatter.fmt2(supply);
        eprintln!("Current supply: {supply_formatted}");

        if supply < supply_min {
            eprintln!("Sending notification");
            let dispatcher = handle!(dispatcher::builder("https://ntfy.sh").build_async(), |source| BuildDispatcherFailed {
                source
            });
            let payload = Payload::new(ntfy_topic)
                .title("USDT supply decreased")
                .message(format!("Current supply: {supply_formatted}"))
                .markdown(true)
                .tags(["warning"])
                .click(url!("https://studio.glassnode.com/charts/supply.Current?a=USDT&category=Supply"));
            handle!(dispatcher.send(&payload).await, |source| SendNotificationFailed {
                source
            });
        }
        Ok(())
    }
}

#[derive(Error, Display, Debug)]
pub enum CheckTetherSupplyCommandRunError {
    #[display("failed to fetch transparency data: {source}")]
    FetchTransparencyFailed { source: reqwest::Error },
    #[display("transparency endpoint returned unexpected status {status}")]
    UnexpectedStatus { status: StatusCode },
    #[display("failed to parse transparency payload: {source}")]
    ParseTransparencyFailed { source: serde_json::Error },
    #[display("no supply fields found in transparency payload")]
    NoSupplyFields,
    #[display("invalid supply value for key {key}: {source}")]
    InvalidSupplyValue { key: String, source: SupplyValueParseError },
    #[display("failed to build ntfy dispatcher: {source}")]
    BuildDispatcherFailed { source: ntfy::Error },
    #[display("failed to send notification: {source}")]
    SendNotificationFailed { source: ntfy::Error },
}

#[derive(Error, Display, Debug)]
pub enum SupplyValueParseError {
    #[display("number exceeds f64 range: {value}")]
    NumberOutOfRange { value: String },
    #[display("failed to parse string as number: {value}: {source}")]
    InvalidString { value: String, source: ParseFloatError },
    #[display("unsupported value type {value_type}")]
    UnsupportedType { value_type: &'static str },
    #[display("non-finite number encountered: {value}")]
    NonFinite { value: f64 },
}

#[derive(Deserialize, Debug)]
struct TransparencyResponse {
    data: TransparencyData,
}

#[derive(Deserialize, Debug)]
struct TransparencyData {
    usdt: HashMap<String, Value>,
}

pub async fn fetch_usdt_supply() -> Result<f64, CheckTetherSupplyCommandRunError> {
    use CheckTetherSupplyCommandRunError::*;

    let response = handle!(reqwest::get(TRANSPARENCY_URL).await, |source| FetchTransparencyFailed {
        source
    });

    let status = response.status();
    if !status.is_success() {
        return Err(UnexpectedStatus {
            status,
        });
    }

    let body = handle!(response.text().await, |source| FetchTransparencyFailed {
        source
    });

    let payload: TransparencyResponse = handle!(serde_json::from_str(&body), |source| ParseTransparencyFailed {
        source
    });

    calculate_usdt_supply(&payload.data.usdt)
}

fn calculate_usdt_supply(usdt: &HashMap<String, Value>) -> Result<f64, CheckTetherSupplyCommandRunError> {
    use CheckTetherSupplyCommandRunError::*;

    let mut total = 0.0_f64;
    let mut seen = false;

    for (key, value) in usdt.iter() {
        if !key.starts_with("totaltokens") {
            continue;
        }

        seen = true;
        let numeric = handle!(value_to_f64(value), |source| InvalidSupplyValue {
            key: key.clone(),
            source,
        });
        total += numeric;
    }

    if !seen {
        return Err(NoSupplyFields);
    }

    Ok(total)
}

fn value_to_f64(value: &Value) -> Result<f64, SupplyValueParseError> {
    match value {
        Value::Number(number) => {
            let numeric = number
                .as_f64()
                .ok_or_else(|| SupplyValueParseError::NumberOutOfRange {
                    value: number.to_string(),
                })?;
            ensure_finite(numeric)
        }
        Value::String(text) => {
            let numeric = handle!(text.parse::<f64>(), |source| SupplyValueParseError::InvalidString {
                value: text.clone(),
                source,
            });
            ensure_finite(numeric)
        }
        Value::Bool(_) => Err(SupplyValueParseError::UnsupportedType {
            value_type: "bool",
        }),
        Value::Array(_) => Err(SupplyValueParseError::UnsupportedType {
            value_type: "array",
        }),
        Value::Object(_) => Err(SupplyValueParseError::UnsupportedType {
            value_type: "object",
        }),
        Value::Null => Err(SupplyValueParseError::UnsupportedType {
            value_type: "null",
        }),
    }
}

fn ensure_finite(value: f64) -> Result<f64, SupplyValueParseError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(SupplyValueParseError::NonFinite {
            value,
        })
    }
}

fn regular_formatter() -> numfmt::Formatter {
    numfmt::Formatter::new()
        .separator(',')
        .unwrap()
        .precision(Precision::Decimals(0))
}
