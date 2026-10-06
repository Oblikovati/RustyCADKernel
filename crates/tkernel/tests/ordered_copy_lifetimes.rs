//! Native constructor allocator ownership and completed-node retention on failure.
#![allow(unsafe_code)]
#[path = "ordered_copy_lifetimes/copy.rs"]
mod copy;
#[path = "ordered_copy_lifetimes/defaults.rs"]
mod defaults;
#[path = "ordered_copy_lifetimes/fixture.rs"]
mod fixture;
