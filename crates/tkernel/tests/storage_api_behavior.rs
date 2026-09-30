//! Behavior of the Storage package: the iostream model, the data sections, callbacks,
//! buckets and the schema. File formats are covered by `fsd_api_behavior`; bit-exact
//! agreement with OCCT by the `stor_*` requests of the differential probe.
#[path = "storage_api_behavior/buckets.rs"]
mod buckets;
#[path = "storage_api_behavior/data.rs"]
mod data;
#[path = "storage_api_behavior/fakes.rs"]
mod fakes;
#[path = "storage_api_behavior/schema.rs"]
mod schema;
#[path = "storage_api_behavior/sections.rs"]
mod sections;
#[path = "storage_api_behavior/stream.rs"]
mod stream;
