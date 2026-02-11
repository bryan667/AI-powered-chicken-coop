use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use std::time::Duration;

const SENSOR_API_BASE_URL_DEFAULT: &str = "http://127.0.0.1:8080";

pub trait Sensor {
    fn read(&self) -> SensorValue;
}

fn redact_key(key: &str) -> String {
    let shown: String = key.chars().take(4).collect();
    format!("{shown}***")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorValue {
    Numeric(f32),
    Binary(bool),
}

#[derive(Deserialize)]
struct NumericResponse {
    value: f32,
}

#[derive(Deserialize)]
struct BinaryResponse {
    value: bool,
}

fn sensor_api_base_url() -> String {
    env::var("SENSOR_API_BASE_URL").unwrap_or_else(|_| SENSOR_API_BASE_URL_DEFAULT.to_string())
}

fn sensor_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("failed to build sensor http client")
}

fn fetch_numeric(path: &str, key: &str) -> Option<f32> {
    let url = format!("{}/{}", sensor_api_base_url().trim_end_matches('/'), path);
    let response = sensor_client()
        .get(url)
        .header("x-api-key", key)
        .send()
        .ok()?;
    let parsed: NumericResponse = response.error_for_status().ok()?.json().ok()?;
    Some(parsed.value)
}

fn fetch_binary(path: &str, key: &str) -> Option<bool> {
    let url = format!("{}/{}", sensor_api_base_url().trim_end_matches('/'), path);
    let response = sensor_client()
        .get(url)
        .header("x-api-key", key)
        .send()
        .ok()?;
    let parsed: BinaryResponse = response.error_for_status().ok()?.json().ok()?;
    Some(parsed.value)
}

pub struct TemperatureSensor {
    pub key: String,
}

impl TemperatureSensor {
    pub fn new(key: &str) -> Self {
        TemperatureSensor {
            key: key.to_string(),
        }
    }
}

impl Sensor for TemperatureSensor {
    fn read(&self) -> SensorValue {
        println!(
            "Reading temperature via API using key {}",
            redact_key(&self.key)
        );
        let value = fetch_numeric("sensors/temperature", &self.key).unwrap_or_else(|| {
            eprintln!("Temperature API unavailable; using 0.0 fallback");
            0.0
        });
        SensorValue::Numeric(value)
    }
}

pub struct HumiditySensor {
    pub key: String,
}

impl HumiditySensor {
    pub fn new(key: &str) -> Self {
        HumiditySensor {
            key: key.to_string(),
        }
    }
}

impl Sensor for HumiditySensor {
    fn read(&self) -> SensorValue {
        println!(
            "Reading humidity via API using key {}",
            redact_key(&self.key)
        );
        let value = fetch_numeric("sensors/humidity", &self.key).unwrap_or_else(|| {
            eprintln!("Humidity API unavailable; using 0.0 fallback");
            0.0
        });
        SensorValue::Numeric(value)
    }
}

pub struct MotionSensor {
    pub key: String,
}

impl MotionSensor {
    pub fn new(key: &str) -> Self {
        MotionSensor {
            key: key.to_string(),
        }
    }
}

impl Sensor for MotionSensor {
    fn read(&self) -> SensorValue {
        println!("Reading motion via API using key {}", redact_key(&self.key));
        let value = fetch_binary("sensors/motion", &self.key).unwrap_or_else(|| {
            eprintln!("Motion API unavailable; using false fallback");
            false
        });
        SensorValue::Binary(value)
    }
}

pub struct EggPresenceSensor {
    pub key: String,
}

impl EggPresenceSensor {
    pub fn new(key: &str) -> Self {
        EggPresenceSensor {
            key: key.to_string(),
        }
    }
}

impl Sensor for EggPresenceSensor {
    fn read(&self) -> SensorValue {
        println!(
            "Reading egg presence via API using key {}",
            redact_key(&self.key)
        );
        let value = fetch_binary("sensors/eggs", &self.key).unwrap_or_else(|| {
            eprintln!("Egg presence API unavailable; using false fallback");
            false
        });
        SensorValue::Binary(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EggPresenceSensor, HumiditySensor, MotionSensor, Sensor, SensorValue, TemperatureSensor,
    };
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, OnceLock};
    use std::thread;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn spawn_json_server(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        thread::spawn(move || {
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().expect("accept");
                let mut buffer = [0_u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).expect("write");
                stream.flush().expect("flush");
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn sensors_read_from_http_gateway() {
        let _guard = env_lock().lock().expect("env lock");
        let url = spawn_json_server(r#"{"value":21.5}"#);
        std::env::set_var("SENSOR_API_BASE_URL", url);

        let temp = TemperatureSensor::new("TEMP");
        let humidity = HumiditySensor::new("HUM");
        assert_eq!(temp.read(), SensorValue::Numeric(21.5));
        assert_eq!(humidity.read(), SensorValue::Numeric(21.5));

        std::env::remove_var("SENSOR_API_BASE_URL");
    }

    #[test]
    fn binary_sensors_read_from_http_gateway() {
        let _guard = env_lock().lock().expect("env lock");
        let url = spawn_json_server(r#"{"value":true}"#);
        std::env::set_var("SENSOR_API_BASE_URL", url);

        let motion = MotionSensor::new("MOTION");
        let eggs = EggPresenceSensor::new("EGG");
        assert_eq!(motion.read(), SensorValue::Binary(true));
        assert_eq!(eggs.read(), SensorValue::Binary(true));

        std::env::remove_var("SENSOR_API_BASE_URL");
    }
}
