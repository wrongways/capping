#[allow(dead_code)]

pub struct BMC {
    pub hostname: String,
    pub username: String,
    pub password: String,
}

impl BMC {
    pub fn new(hostname: &str, username: &str, password: &str) -> Self {
        Self {
            hostname: String::from(hostname),
            username: String::from(username),
            password: String::from(password),
        }
    }
    pub fn read_power(&self) -> u64 {
        250
    }

    #[allow(unused_variables)]
    pub fn set_cap_power_level(&self, cap: u64) {}

    pub fn activate_power_cap(&self) {}

    pub fn deactivate_power_cap(&self) {}
}
