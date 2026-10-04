mod hidden {
    pub struct ResultRecord {
        pub succeeded: bool,
    }
}

pub mod api {
    pub use crate::hidden::ResultRecord;
}

impl From<&api::ResultRecord> for bool {
    fn from(value: &api::ResultRecord) -> Self {
        value.succeeded
    }
}
