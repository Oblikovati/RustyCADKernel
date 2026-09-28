//! Behavior tests for the Standard services area: memory managers, synchronization,
//! locale, buffered readers, dump and Plugin.
#[path = "standard_services_api_behavior/array_stream.rs"]
mod array_stream;
#[path = "standard_services_api_behavior/dump.rs"]
mod dump;
#[path = "standard_services_api_behavior/locale.rs"]
mod locale;
#[path = "standard_services_api_behavior/memory.rs"]
mod memory;
#[path = "standard_services_api_behavior/memory_utils.rs"]
mod memory_utils;
#[path = "standard_services_api_behavior/mmgr.rs"]
mod mmgr;
#[path = "standard_services_api_behavior/plugin.rs"]
mod plugin;
#[path = "standard_services_api_behavior/readers.rs"]
mod readers;
#[path = "standard_services_api_behavior/sync.rs"]
mod sync;
