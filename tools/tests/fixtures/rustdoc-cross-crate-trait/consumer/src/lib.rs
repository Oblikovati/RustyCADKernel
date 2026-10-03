pub use trait_provider::{Operation as PublicOperation, Value};

pub fn apply(value: &Value) -> u8 {
    <Value as PublicOperation>::apply(value)
}
