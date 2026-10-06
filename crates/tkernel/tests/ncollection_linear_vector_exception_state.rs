//! Supplemental exception and relocation diagnostics for `LinearVector`.

#[path = "ncollection_linear_vector/exceptions.rs"]
mod exceptions;

#[path = "ncollection_linear_vector/append_order.rs"]
mod append_order;

#[path = "ncollection_linear_vector/resize_value_ref.rs"]
mod resize_value_ref;

#[path = "ncollection_linear_vector/assign_tail.rs"]
mod assign_tail;

#[path = "support/linear_unpublished_tail.rs"]
mod tail_repair;
