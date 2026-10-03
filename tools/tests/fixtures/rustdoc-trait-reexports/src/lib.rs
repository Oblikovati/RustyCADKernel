mod hidden {
    pub trait Operation {
        fn apply(&self) -> u8;
    }

    #[derive(Clone)]
    pub struct Value;

    impl Operation for Value {
        fn apply(&self) -> u8 {
            7
        }
    }
}

pub use hidden::{Operation, Value};
pub use hidden::Operation as Alias;
