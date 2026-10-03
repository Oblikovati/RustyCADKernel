//! Behavior of the Message package members that need no process-wide state: statuses,
//! messages and catalogs, alerts and reports, printers and algorithms.
#[path = "message_api_behavior/algorithm.rs"]
mod algorithm;
#[path = "message_api_behavior/catalog.rs"]
mod catalog;
#[path = "message_api_behavior/fakes.rs"]
mod fakes;
#[path = "message_api_behavior/msg.rs"]
mod msg;
#[path = "message_api_behavior/printers.rs"]
mod printers;
#[path = "message_api_behavior/report.rs"]
mod report;
#[path = "message_api_behavior/status.rs"]
mod status;
#[path = "message_api_behavior/status_values.rs"]
mod status_values;
