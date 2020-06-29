pub struct Application {
    name: String,
}

impl Application {
    pub fn new(name: &str) -> Application {
        Application {
            name: String::from(name),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
