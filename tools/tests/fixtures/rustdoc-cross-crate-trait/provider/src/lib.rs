pub trait Operation {
    fn apply(&self) -> u8;
}

pub struct Value;

impl Operation for Value {
    fn apply(&self) -> u8 {
        7
    }
}
