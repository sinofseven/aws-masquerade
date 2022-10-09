pub mod cmd {
    pub mod configure {
        pub const NAME: &str = "configure";
        pub mod sub_command {
            pub const PATH: &str = "path";
            pub const VALIDATE: &str = "validate";
            pub const MIGRATE: &str = "migrate";
        }
    }
}

pub mod models {
    pub mod configuration {
        pub enum Version {
            V0,
            V1,
        }
    }
}
