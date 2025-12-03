use clap::Parser;
use derive_more::Error;
use fmt_derive::Display;
use ntfy::{Payload, dispatcher};
use stub_macro::stub;
use url_macro::url;

#[derive(Parser, Clone, Debug)]
pub struct CheckTetherSupplyCommand {
    #[arg(short, long, env = "DENMON_NTFY_TOPIC")]
    ntfy_topic: String,

    #[arg(short, long)]
    supply_min: f64,
}

impl CheckTetherSupplyCommand {
    // TODO: Fix error handling
    pub async fn run(self) -> Result<(), CheckTetherSupplyCommandRunError> {
        let Self {
            ntfy_topic,
            supply_min,
        } = self;
        // TODO: Fetch https://app.tether.to/transparency.json
        // TODO: Get the `supply` of USDT by summing over each key in `data.usdt` that starts with "totaltokens"
        let supply = stub!(f64);
        if supply < supply_min {
            let dispatcher = dispatcher::builder("https://ntfy.sh")
                .build_async()
                .unwrap();
            let payload = Payload::new(ntfy_topic)
                .title("USDT supply decreased")
                .message(format!("* Current supply: {supply}"))
                .markdown(true)
                .tags(["warning"])
                .click(url!("https://studio.glassnode.com/charts/supply.Current?a=USDT&category=Supply"));
            dispatcher.send(&payload).await.unwrap();
        }
        Ok(())
    }
}

#[derive(Error, Display, Debug)]
pub enum CheckTetherSupplyCommandRunError {}
