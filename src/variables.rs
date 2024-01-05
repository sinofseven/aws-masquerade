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

pub mod output {
    pub mod environment_variables {
        pub const AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
        pub const AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
        pub const AWS_SESSION_TOKEN: &str = "AWS_SESSION_TOKEN";
        pub const AWS_SECURITY_TOKEN: &str = "AWS_SECURITY_TOKEN";
        pub const AWS_DEFAULT_OUTPUT: &str = "AWS_DEFAULT_OUTPUT";
        pub const AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";
        pub const AWS_REGION: &str = "AWS_REGION";
    }

    pub mod shared_credentials {
        pub const AWS_ACCESS_KEY_ID: &str = "aws_access_key_id";
        pub const AWS_SECRET_ACCESS_KEY: &str = "aws_secret_access_key";
        pub const AWS_SESSION_TOKEN: &str = "aws_session_token";
        pub const AWS_SECURITY_TOKEN: &str = "aws_security_token";
        pub const X_SECURITY_TOKEN_EXPIRES: &str = "x_security_token_expires";
        pub const X_PRINCIPAL_ARN: &str = "x_principal_arn";
        pub const OUTPUT: &str = "output";
        pub const REGION: &str = "region";
    }
}
