mod hidden {
    pub struct Value {
        pub field: u8,
    }

    impl Value {
        pub fn method(&self) -> u8 {
            self.field
        }
    }

    pub fn function() -> Value {
        Value { field: 7 }
    }

    pub mod nested {
        pub const CONST: u8 = 11;
    }
}

pub mod api {
    pub use crate::hidden::*;
}

pub use api::*;
