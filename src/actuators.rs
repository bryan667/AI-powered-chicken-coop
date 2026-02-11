use reqwest::blocking::Client;
use serde::Serialize;
use std::env;
use std::time::Duration;

const ACTUATOR_API_BASE_URL_DEFAULT: &str = "http://127.0.0.1:8080";

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
    use super::{CoopDoor, FeederMotor};
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
}
