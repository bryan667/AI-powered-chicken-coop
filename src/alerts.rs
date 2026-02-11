pub struct Alert {
    pub name: String,
}

impl Alert {
    pub fn new(name: &str) -> Self {
        Alert {
            name: name.to_string(),
        }
    }

    pub fn send(&self) {
        println!("Sending alert: {}", self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::Alert;

    #[test]
    fn alert_can_be_sent() {
        let alert = Alert::new("Predator detected");
        alert.send();
    }
}
