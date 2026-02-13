mod actuator_server;
mod actuators;
mod ai;
mod alerts;
mod cache;
mod camera;
mod cli;
mod scheduler;
mod sensors;

use clap::Parser;
use cli::{Cli, Commands, FeedCommands, RunCommands, ServeCommands};
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

    match args.command {
        Some(Commands::Status) => {
            let temp_sensor_key = required_env("TEMP_SENSOR_KEY");
            let humidity_sensor_key = required_env("HUMIDITY_SENSOR_KEY");
            let motion_sensor_key = required_env("MOTION_SENSOR_KEY");
            let egg_sensor_key = required_env("EGG_SENSOR_KEY");
            let cache_key = required_env("CACHE_KEY");

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
            let actuator_api_key = required_env("ACTUATOR_API_KEY");
            let feeder_key = required_env("FEEDER_KEY");
            let door_key = required_env("DOOR_KEY");
            let feeder = actuators::FeederMotor::new(&feeder_key, &actuator_api_key);
            let door = actuators::CoopDoor::new(&door_key, &actuator_api_key);
            println!("Activating feeder now...");
            feeder.activate();
            door.open();
            door.close();
        }
        Some(Commands::Run {
            action:
                RunCommands::AiVision {
                    image,
                    camera_index,
                    frames,
                },
        }) => {
            let ai_key = required_env("AI_KEY");
            let vision_model_path =
                env_or_default("VISION_MODEL_PATH", "models/mobilenetv2-7.onnx");
            let predator_threshold = env_or_default("PREDATOR_THRESHOLD", "0.30")
                .parse::<f32>()
                .unwrap_or(0.30);
            println!("Running AI vision modules...");
            let vision = ai::AiVision::load_model(&vision_model_path, &ai_key);

            if let Some(image_path) = image {
                match vision.classify_image(&image_path) {
                    Ok(result) => {
                        println!(
                            "Label: {} (confidence {:.3})",
                            result.label, result.confidence
                        );
                        println!("Chicken detected: {}", result.chicken_detected);
                        println!("Predator detected: {}", result.predator_detected);
                        if result.predator_detected && result.confidence >= predator_threshold {
                            let alert = alerts::Alert::new("Predator detected in image");
                            alert.send();
                        }
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                }
            } else {
                #[cfg(feature = "camera")]
                {
                    let warmup_frames = env_or_default("VISION_WARMUP_FRAMES", "8")
                        .parse::<u32>()
                        .unwrap_or(8);
                    let mut session = match camera::CameraSession::open(camera_index) {
                        Ok(session) => session,
                        Err(err) => {
                            eprintln!("{err}");
                            std::process::exit(1);
                        }
                    };
                    if let Err(err) = session.warm_up(warmup_frames) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }

                    for _ in 0..frames {
                        match session.capture_frame() {
                            Ok(frame) => match vision.classify_dynamic_image(frame.clone()) {
                                Ok(result) => {
                                    println!(
                                        "Webcam label: {} (confidence {:.3})",
                                        result.label, result.confidence
                                    );
                                    println!("Chicken detected: {}", result.chicken_detected);
                                    println!("Predator detected: {}", result.predator_detected);
                                    if result.chicken_detected || result.predator_detected {
                                        match camera::save_detection_frame(
                                            &frame,
                                            &result.label,
                                            result.confidence,
                                        ) {
                                            Ok(path) => println!("Saved detection frame: {path}"),
                                            Err(err) => eprintln!("{err}"),
                                        }
                                    }
                                    if result.predator_detected
                                        && result.confidence >= predator_threshold
                                    {
                                        let alert =
                                            alerts::Alert::new("Predator detected in webcam frame");
                                        alert.send();
                                    }
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    std::process::exit(1);
                                }
                            },
                            Err(err) => {
                                eprintln!("{err}");
                                std::process::exit(1);
                            }
                        }
                    }
                }
                #[cfg(not(feature = "camera"))]
                {
                    let _ = frames;
                    let _ = camera_index;
                    eprintln!("Webcam mode requires cargo feature `camera`.");
                    eprintln!("Use --image <path> or run with --features camera.");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Serve {
            action: ServeCommands::Actuators,
        }) => {
            let actuator_api_key = required_env("ACTUATOR_API_KEY");
            let bind_addr = env_or_default("ACTUATOR_BIND_ADDR", "0.0.0.0:8081");
            println!("Starting actuator receiver on {bind_addr}");
            if let Err(err) =
                actuator_server::run_actuator_server(&bind_addr, actuator_api_key).await
            {
                eprintln!("Actuator receiver exited with error: {err}");
                std::process::exit(1);
            }
        }
        None => {
            println!("Welcome to AI Chicken Coop! Use --help for commands.");
            println!(
                "Try: `coop status`, `coop feed now`, `coop run ai-vision`, `coop serve actuators`"
            );
        }
    }
}
