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
        println!("Reading temperature using key {}", redact_key(&self.key));
        SensorValue::Numeric(32.0)
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
        println!("Reading humidity using key {}", redact_key(&self.key));
        SensorValue::Numeric(75.0)
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
        println!("Reading motion using key {}", redact_key(&self.key));
        SensorValue::Binary(false)
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
        println!("Reading egg presence using key {}", redact_key(&self.key));
        SensorValue::Binary(true)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EggPresenceSensor, HumiditySensor, MotionSensor, Sensor, SensorValue, TemperatureSensor,
    };

    #[test]
    fn sensor_reads_return_expected_types() {
        let temp = TemperatureSensor::new("TEMP");
        let humidity = HumiditySensor::new("HUM");
        let motion = MotionSensor::new("MOTION");
        let eggs = EggPresenceSensor::new("EGG");

        assert_eq!(temp.read(), SensorValue::Numeric(32.0));
        assert_eq!(humidity.read(), SensorValue::Numeric(75.0));
        assert_eq!(motion.read(), SensorValue::Binary(false));
        assert_eq!(eggs.read(), SensorValue::Binary(true));
    }
}
