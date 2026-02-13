use reqwest::blocking::Client;
use serde::Serialize;
use std::env;
use std::process::Command;
use std::time::Duration;

const ACTUATOR_API_BASE_URL_DEFAULT: &str = "http://127.0.0.1:8081";

fn redact_key(key: &str) -> String {
    let shown: String = key.chars().take(4).collect();
    format!("{shown}***")
}

fn actuator_api_base_url() -> String {
    env::var("ACTUATOR_API_BASE_URL").unwrap_or_else(|_| ACTUATOR_API_BASE_URL_DEFAULT.to_string())
}

fn actuator_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("failed to build actuator http client")
}

#[derive(Serialize)]
struct ActuatorCommand<'a> {
    device_key: &'a str,
}

fn send_command(path: &str, device_key: &str, api_key: &str) -> bool {
    let url = format!("{}/{}", actuator_api_base_url().trim_end_matches('/'), path);
    actuator_client()
        .post(url)
        .header("x-api-key", api_key)
        .json(&ActuatorCommand { device_key })
        .send()
        .and_then(|resp| resp.error_for_status())
        .is_ok()
}

pub trait ActuatorDriver: Send {
    fn feeder_activate(&mut self, device_key: &str, duration_ms: u64) -> Result<(), String>;
    fn door_open(&mut self, device_key: &str) -> Result<(), String>;
    fn door_close(&mut self, device_key: &str) -> Result<(), String>;
}

pub fn create_driver_from_env() -> Result<Box<dyn ActuatorDriver>, String> {
    let backend = env::var("ACTUATOR_BACKEND").unwrap_or_else(|_| "command".to_string());
    match backend.as_str() {
        "command" => Ok(Box::new(LocalActuatorDriver::default())),
        "rpi-gpio" => create_rpi_driver_from_env(),
        _ => Err(format!(
            "unsupported ACTUATOR_BACKEND `{backend}` (expected `command` or `rpi-gpio`)"
        )),
    }
}

#[cfg(all(feature = "pi-hw", target_os = "linux"))]
fn parse_u8_env(name: &str, default: u8) -> Result<u8, String> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u8>()
            .map_err(|_| format!("invalid value for {name}: {value}")),
        Err(_) => Ok(default),
    }
}

#[cfg(all(feature = "pi-hw", target_os = "linux"))]
fn parse_u64_env(name: &str, default: u64) -> Result<u64, String> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|_| format!("invalid value for {name}: {value}")),
        Err(_) => Ok(default),
    }
}

#[cfg(all(feature = "pi-hw", target_os = "linux"))]
fn parse_bool_env(name: &str, default: bool) -> Result<bool, String> {
    match env::var(name) {
        Ok(value) => match value.as_str() {
            "1" | "true" | "TRUE" | "yes" | "on" => Ok(true),
            "0" | "false" | "FALSE" | "no" | "off" => Ok(false),
            _ => Err(format!("invalid value for {name}: {value}")),
        },
        Err(_) => Ok(default),
    }
}

pub struct LocalActuatorDriver {
    pub door_is_open: bool,
    feeder_activate_cmd: Option<String>,
    door_open_cmd: Option<String>,
    door_close_cmd: Option<String>,
}

impl Default for LocalActuatorDriver {
    fn default() -> Self {
        LocalActuatorDriver {
            door_is_open: false,
            feeder_activate_cmd: env::var("FEEDER_ACTIVATE_CMD").ok(),
            door_open_cmd: env::var("DOOR_OPEN_CMD").ok(),
            door_close_cmd: env::var("DOOR_CLOSE_CMD").ok(),
        }
    }
}

#[cfg(all(feature = "pi-hw", target_os = "linux"))]
fn create_rpi_driver_from_env() -> Result<Box<dyn ActuatorDriver>, String> {
    let feeder_pin = parse_u8_env("FEEDER_GPIO_PIN", 17)?;
    let door_open_pin = parse_u8_env("DOOR_OPEN_GPIO_PIN", 27)?;
    let door_close_pin = parse_u8_env("DOOR_CLOSE_GPIO_PIN", 22)?;
    let active_high = parse_bool_env("ACTUATOR_ACTIVE_HIGH", true)?;
    let door_pulse_ms = parse_u64_env("DOOR_PULSE_MS", 1200)?;
    let driver = RpiGpioActuatorDriver::new(
        feeder_pin,
        door_open_pin,
        door_close_pin,
        active_high,
        door_pulse_ms,
    )?;
    Ok(Box::new(driver))
}

#[cfg(not(all(feature = "pi-hw", target_os = "linux")))]
fn create_rpi_driver_from_env() -> Result<Box<dyn ActuatorDriver>, String> {
    Err("ACTUATOR_BACKEND=rpi-gpio requires Linux and cargo feature `pi-hw`".to_string())
}

fn run_hardware_command(
    command_template: &str,
    device_key: &str,
    duration_ms: Option<u64>,
) -> Result<(), String> {
    let rendered = command_template
        .replace("{device_key}", device_key)
        .replace("{duration_ms}", &duration_ms.unwrap_or(0).to_string());

    #[cfg(target_os = "windows")]
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &rendered])
        .status();

    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh").args(["-c", &rendered]).status();

    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(exit) => Err(format!("command failed with status: {exit}")),
        Err(err) => Err(format!("failed to execute command: {err}")),
    }
}

impl ActuatorDriver for LocalActuatorDriver {
    fn feeder_activate(&mut self, device_key: &str, duration_ms: u64) -> Result<(), String> {
        println!(
            "Feeder relay activated for {}ms using device {}",
            duration_ms,
            redact_key(device_key)
        );
        if let Some(cmd) = &self.feeder_activate_cmd {
            run_hardware_command(cmd, device_key, Some(duration_ms))?;
        }
        Ok(())
    }

    fn door_open(&mut self, device_key: &str) -> Result<(), String> {
        self.door_is_open = true;
        println!(
            "Door motor set to OPEN using device {}",
            redact_key(device_key)
        );
        if let Some(cmd) = &self.door_open_cmd {
            run_hardware_command(cmd, device_key, None)?;
        }
        Ok(())
    }

    fn door_close(&mut self, device_key: &str) -> Result<(), String> {
        self.door_is_open = false;
        println!(
            "Door motor set to CLOSE using device {}",
            redact_key(device_key)
        );
        if let Some(cmd) = &self.door_close_cmd {
            run_hardware_command(cmd, device_key, None)?;
        }
        Ok(())
    }
}

#[cfg(all(feature = "pi-hw", target_os = "linux"))]
mod rpi_gpio {
    use super::ActuatorDriver;
    use rppal::gpio::{Gpio, Level, OutputPin};
    use std::thread::sleep;
    use std::time::Duration;

    pub struct RpiGpioActuatorDriver {
        feeder_pin: OutputPin,
        door_open_pin: OutputPin,
        door_close_pin: OutputPin,
        active_level: Level,
        inactive_level: Level,
        door_pulse_ms: u64,
    }

    impl RpiGpioActuatorDriver {
        pub fn new(
            feeder_pin: u8,
            door_open_pin: u8,
            door_close_pin: u8,
            active_high: bool,
            door_pulse_ms: u64,
        ) -> Result<Self, String> {
            let gpio = Gpio::new().map_err(|e| format!("gpio init failed: {e}"))?;
            let active_level = if active_high { Level::High } else { Level::Low };
            let inactive_level = if active_high { Level::Low } else { Level::High };

            let mut feeder = gpio
                .get(feeder_pin)
                .map_err(|e| format!("gpio pin {feeder_pin} unavailable: {e}"))?
                .into_output();
            feeder.write(inactive_level);

            let mut door_open = gpio
                .get(door_open_pin)
                .map_err(|e| format!("gpio pin {door_open_pin} unavailable: {e}"))?
                .into_output();
            door_open.write(inactive_level);

            let mut door_close = gpio
                .get(door_close_pin)
                .map_err(|e| format!("gpio pin {door_close_pin} unavailable: {e}"))?
                .into_output();
            door_close.write(inactive_level);

            Ok(Self {
                feeder_pin: feeder,
                door_open_pin: door_open,
                door_close_pin: door_close,
                active_level,
                inactive_level,
                door_pulse_ms,
            })
        }

        fn pulse(pin: &mut OutputPin, active: Level, inactive: Level, ms: u64) {
            pin.write(active);
            sleep(Duration::from_millis(ms));
            pin.write(inactive);
        }
    }

    impl ActuatorDriver for RpiGpioActuatorDriver {
        fn feeder_activate(&mut self, _device_key: &str, duration_ms: u64) -> Result<(), String> {
            Self::pulse(
                &mut self.feeder_pin,
                self.active_level,
                self.inactive_level,
                duration_ms,
            );
            Ok(())
        }

        fn door_open(&mut self, _device_key: &str) -> Result<(), String> {
            self.door_close_pin.write(self.inactive_level);
            Self::pulse(
                &mut self.door_open_pin,
                self.active_level,
                self.inactive_level,
                self.door_pulse_ms,
            );
            Ok(())
        }

        fn door_close(&mut self, _device_key: &str) -> Result<(), String> {
            self.door_open_pin.write(self.inactive_level);
            Self::pulse(
                &mut self.door_close_pin,
                self.active_level,
                self.inactive_level,
                self.door_pulse_ms,
            );
            Ok(())
        }
    }
}

#[cfg(all(feature = "pi-hw", target_os = "linux"))]
use rpi_gpio::RpiGpioActuatorDriver;

pub struct FeederMotor {
    pub key: String,
    pub api_key: String,
}

impl FeederMotor {
    pub fn new(key: &str, api_key: &str) -> Self {
        FeederMotor {
            key: key.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn activate(&self) {
        println!("Sending feeder command using key {}", redact_key(&self.key));
        if !send_command("actuators/feeder/activate", &self.key, &self.api_key) {
            eprintln!("Feeder command failed.");
        }
    }
}

pub struct CoopDoor {
    pub key: String,
    pub api_key: String,
}

impl CoopDoor {
    pub fn new(key: &str, api_key: &str) -> Self {
        CoopDoor {
            key: key.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn open(&self) {
        println!(
            "Sending door open command using key {}",
            redact_key(&self.key)
        );
        if !send_command("actuators/door/open", &self.key, &self.api_key) {
            eprintln!("Door open command failed.");
        }
    }

    pub fn close(&self) {
        println!(
            "Sending door close command using key {}",
            redact_key(&self.key)
        );
        if !send_command("actuators/door/close", &self.key, &self.api_key) {
            eprintln!("Door close command failed.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{create_driver_from_env, CoopDoor, FeederMotor};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex, OnceLock};
    use std::thread;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn spawn_actuator_server(requests_to_serve: usize) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let paths = Arc::new(Mutex::new(Vec::new()));
        let paths_in_thread = Arc::clone(&paths);

        thread::spawn(move || {
            for _ in 0..requests_to_serve {
                let (mut stream, _) = listener.accept().expect("accept");
                let mut buffer = [0_u8; 2048];
                let size = stream.read(&mut buffer).expect("read");
                let req = String::from_utf8_lossy(&buffer[..size]).to_string();
                if let Some(first_line) = req.lines().next() {
                    paths_in_thread
                        .lock()
                        .expect("lock paths")
                        .push(first_line.to_string());
                }
                let response =
                    "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                stream.write_all(response.as_bytes()).expect("write");
                stream.flush().expect("flush");
            }
        });

        (format!("http://{addr}"), paths)
    }

    #[test]
    fn actuators_send_expected_commands() {
        let _guard = env_lock().lock().expect("env lock");
        let (url, paths) = spawn_actuator_server(3);
        std::env::set_var("ACTUATOR_API_BASE_URL", url);

        let feeder = FeederMotor::new("FEEDER_DEVICE", "ACTUATOR_API_TOKEN");
        let door = CoopDoor::new("DOOR_DEVICE", "ACTUATOR_API_TOKEN");

        feeder.activate();
        door.open();
        door.close();

        std::thread::sleep(std::time::Duration::from_millis(20));
        let seen = paths.lock().expect("lock paths");
        assert!(seen
            .iter()
            .any(|line| line.starts_with("POST /actuators/feeder/activate ")));
        assert!(seen
            .iter()
            .any(|line| line.starts_with("POST /actuators/door/open ")));
        assert!(seen
            .iter()
            .any(|line| line.starts_with("POST /actuators/door/close ")));

        std::env::remove_var("ACTUATOR_API_BASE_URL");
    }

    #[test]
    fn command_backend_is_default() {
        let _guard = env_lock().lock().expect("env lock");
        std::env::remove_var("ACTUATOR_BACKEND");
        assert!(create_driver_from_env().is_ok());
    }
}
