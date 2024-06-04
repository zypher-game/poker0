use chrono::Local;
use handler::init_prover_key;
use poker_snark::build_cs::N_CARDS;
use z4_engine::{Config, Engine};

mod errors;
mod handler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    init_prover_key(N_CARDS);
    println!("\n{}", Local::now().format("%Y/%m/%d %H:%M:%S"));

    let config = Config::from_env().unwrap();
    Engine::<handler::PokerHandler>::init(config)
        .run()
        .await
        .expect("Down");
}
