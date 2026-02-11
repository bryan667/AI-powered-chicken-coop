pub trait Sensor {
    fn read(&self) -> f32;
}

pub struct TemperatureSensor {
    pub key: String,
}

impl TemperatureSensor {
    pub fn new(key: &str) -> Self {
        TemperatureSensor { key: key.to_string() }
    }
}

impl Sensor for TemperatureSensor {
    fn read(&self) -> f32 {
        println!("Reading temperature using key {}", self.key);
        32.0
    }
}

pub struct HumiditySensor {
    pub key: String,
}

impl HumiditySensor {
    pub fn new(key: &str) -> Self {
        HumiditySensor { key: key.to_string() }
    }
}

impl Sensor for HumiditySensor {
    fn read(&self) -> f32 {
        println!("Reading humidity using key {}", self.key);
        75.0
    }
}
