mod actuators;
mod ai;
mod alerts;
mod cache;
mod cli;
mod scheduler;
mod sensors;

use clap::Parser;
use cli::{Cli, Commands, FeedCommands, RunCommands};
use sensors::{
    EggPresenceSensor, HumiditySensor, MotionSensor, Sensor, SensorValue, TemperatureSensor,
};
use tokio::time::Duration;

fn format_sensor_value(value: SensorValue) -> String {
    match value {
        SensorValue::Numeric(v) => format!("{v:.1}"),
        SensorValue::Binary(v) => v.to_string(),
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Status) => {
            let temp_sensor = TemperatureSensor::new("TEMP_KEY");
            let humidity_sensor = HumiditySensor::new("HUMIDITY_KEY");
            let motion_sensor = MotionSensor::new("MOTION_KEY");
            let egg_sensor = EggPresenceSensor::new("EGG_KEY");
            let mut data_cache = cache::DataCache::new("CACHE_KEY");

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
            let feeder = actuators::FeederMotor::new("FEEDER_KEY");
            let door = actuators::CoopDoor::new("DOOR_KEY");
            println!("Activating feeder now...");
            feeder.activate();
            door.open();
            door.close();
        }
        Some(Commands::Run {
            action: RunCommands::AiVision,
        }) => {
            println!("Running AI vision modules...");
            let egg_detector =
                ai::AiVision::load_model("/models/egg_detector.pt", "AI_KEY");
            let predator_detector =
                ai::AiVision::load_model("/models/predator_detector.pt", "AI_KEY");
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
