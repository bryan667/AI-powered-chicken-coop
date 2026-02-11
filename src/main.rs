mod actuators;
mod ai;
mod alerts;
mod cache;
mod cli;
mod scheduler;
mod sensors;

use clap::Parser;
use cli::{Cli, Commands, FeedCommands, RunCommands};
use dotenvy::dotenv;
use sensors::{
    EggPresenceSensor, HumiditySensor, MotionSensor, Sensor, SensorValue, TemperatureSensor,
};
use std::env;
use tokio::time::Duration;

fn format_sensor_value(value: SensorValue) -> String {
    match value {
        SensorValue::Numeric(v) => format!("{v:.1}"),
        SensorValue::Binary(v) => v.to_string(),
    }
}

fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| {
        eprintln!("Missing required environment variable: {name}");
        std::process::exit(2);
    })
}

fn env_or_default(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args = Cli::parse();

    let temp_sensor_key = required_env("TEMP_SENSOR_KEY");
    let humidity_sensor_key = required_env("HUMIDITY_SENSOR_KEY");
    let motion_sensor_key = required_env("MOTION_SENSOR_KEY");
    let egg_sensor_key = required_env("EGG_SENSOR_KEY");
    let cache_key = required_env("CACHE_KEY");
    let actuator_api_key = required_env("ACTUATOR_API_KEY");
    let feeder_key = required_env("FEEDER_KEY");
    let door_key = required_env("DOOR_KEY");
    let ai_key = required_env("AI_KEY");
    let egg_model_path = env_or_default("EGG_MODEL_PATH", "/models/egg_detector.pt");
    let predator_model_path = env_or_default("PREDATOR_MODEL_PATH", "/models/predator_detector.pt");

    match args.command {
        Some(Commands::Status) => {
            let temp_sensor = TemperatureSensor::new(&temp_sensor_key);
            let humidity_sensor = HumiditySensor::new(&humidity_sensor_key);
            let motion_sensor = MotionSensor::new(&motion_sensor_key);
            let egg_sensor = EggPresenceSensor::new(&egg_sensor_key);
            let mut data_cache = cache::DataCache::new(&cache_key);

            let temp = format_sensor_value(temp_sensor.read());
            let humidity = format_sensor_value(humidity_sensor.read());
            let motion = format_sensor_value(motion_sensor.read());
            let eggs = format_sensor_value(egg_sensor.read());

            data_cache.store("last_temp", &temp);
            data_cache.store("last_humidity", &humidity);
            data_cache.store("last_motion", &motion);
            data_cache.store("last_egg_presence", &eggs);

            scheduler::run_scheduled_tasks(1, Duration::from_millis(5)).await;

            println!("Chicken Coop Status");
            println!("Egg presence: {eggs}");
            println!("Temperature: {temp}C");
            println!("Humidity: {humidity}%");
            println!("Motion detected: {motion}");
            println!("Cached temp: {:?}", data_cache.retrieve("last_temp"));
        }
        Some(Commands::Feed {
            action: FeedCommands::Now,
        }) => {
            let feeder = actuators::FeederMotor::new(&feeder_key, &actuator_api_key);
            let door = actuators::CoopDoor::new(&door_key, &actuator_api_key);
            println!("Activating feeder now...");
            feeder.activate();
            door.open();
            door.close();
        }
        Some(Commands::Run {
            action: RunCommands::AiVision,
        }) => {
            println!("Running AI vision modules...");
            let egg_detector = ai::AiVision::load_model(&egg_model_path, &ai_key);
            let predator_detector = ai::AiVision::load_model(&predator_model_path, &ai_key);
            if egg_detector.detect() {
                println!("Egg detected!");
            }
            if predator_detector.detect() {
                let alert = alerts::Alert::new("Predator detected");
                alert.send();
            } else {
                println!("No predators detected.");
            }
        }
        None => {
            println!("Welcome to AI Chicken Coop! Use --help for commands.");
            println!("Try: `coop status`, `coop feed now`, `coop run ai-vision`");
        }
    }
}
