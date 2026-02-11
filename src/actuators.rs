pub struct FeederMotor {
    pub key: String,
}

impl FeederMotor {
    pub fn new(key: &str) -> Self {
        FeederMotor {
            key: key.to_string(),
        }
    }

    pub fn activate(&self) {
        println!("Feeder activated using key {}", self.key);
    }
}

pub struct CoopDoor {
    pub key: String,
}

impl CoopDoor {
    pub fn new(key: &str) -> Self {
        CoopDoor {
            key: key.to_string(),
        }
    }

    pub fn open(&self) {
        println!("Coop door opened using key {}", self.key);
    }

    pub fn close(&self) {
        println!("Coop door closed using key {}", self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::{CoopDoor, FeederMotor};

    #[test]
    fn actuators_can_be_constructed() {
        let feeder = FeederMotor::new("FEEDER");
        let door = CoopDoor::new("DOOR");

        feeder.activate();
        door.open();
        door.close();
    }
}
