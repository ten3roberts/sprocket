pub struct Application {
    name: String,
}

impl Application {
    /// Creates a new blank application with the given name
    pub fn new(name: &str) -> Application {
        Application {
            name: String::from(name),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
