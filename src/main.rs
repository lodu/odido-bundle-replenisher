mod config;

mod http;
mod models;
mod odido;

use config::Config;
use odido::Odido;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config: Config = Config::from_env()?;
    let interval = config.update_interval;
    let mb_threshold = config.mb_threshold;
    let mut odido = Odido::new(config);

    loop {
        let started = Instant::now();

        match odido.mbs_left() {
            Ok(mb_left) => {
                println!("{mb_left} MB's beschikbaar");
                if mb_left < mb_threshold {
                    if let Err(e) = odido.request_bundle() {
                        eprintln!("Fout bij aanvragen van extra databundel: {e}");
                    }
                }
            }
            Err(e) => eprintln!("Fout bij opvragen hoeveel MB's beschikbaar: {e}"),
        }

        let sleep_for: Duration = interval.saturating_sub(started.elapsed());
        std::thread::sleep(sleep_for);
    }
}
