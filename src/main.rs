mod sensors;
mod actuators;
mod ai;
mod cache;
mod scheduler;
mod alerts;
mod cli;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Some(cli::Commands::Status) => {
            println!("Chicken Coop Status");
            println!("Eggs collected: 4");
            println!("Temperature: 32°C");
            println!("Predators: 0");
        }
        Some(cli::Commands::Feed) => {
            println!("Activating feeder...");
            actuators::FeederMotor::new("FEEDER_KEY_789").activate();
        }
        Some(cli::Commands::RunAi) => {
            println!("Running AI vision modules...");
            let egg_detector = ai::AiVision::load_model("/models/egg_detector.pt", "AI_KEY");
            if egg_detector.detect() {
                println!("Egg detected!");
            }
        }
        None => {
            println!("Welcome to AI Chicken Coop! Use --help for commands.");
        }
    }
}
