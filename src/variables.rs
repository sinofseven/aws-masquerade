pub mod cmd {
    pub mod configure {
        pub const NAME: &str = "configure";
        pub mod sub_command {
            pub const PATH: &str = "path";
            pub const VALIDATE: &str = "validate";
            pub const MIGRATE: &str = "migrate";
        }
    }

    pub mod assume {
        pub const NAME: &str = "assume";
    }

    pub mod source {
        pub const NAME: &str = "source";

        pub mod sub_command {
            pub const LIST: &str = "list";
            pub const SHOW: &str = "show";
            pub const ADD: &str = "add";
            pub const EDIT: &str = "edit";
            pub const REMOVE: &str = "remove";
        }
    }

    pub mod target {
        pub const NAME: &str = "target";

        pub mod sub_command {
            pub const LIST: &str = "list";
            pub const SHOW: &str = "show";
            pub const ADD: &str = "add";
            pub const EDIT: &str = "edit";
            pub const REMOTE: &str = "remove";
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
