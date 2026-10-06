//! Native raw constructor adoption and Rust arena lifetime protection.
#![allow(unsafe_code)]
mod buffer_raw_adoption_contracts {
    pub mod concurrency;
    pub mod lifetime;
    pub mod oracle;
}
