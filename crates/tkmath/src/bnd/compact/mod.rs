//! Center/half-size bounding boxes preserve scalar-type rounding at each native operation.
mod real;
pub use real::BoxReal;
mod box2;
pub use box2::CompactBox2;
mod box3;
mod relation;
mod storage;
pub use box3::CompactBox3;
mod ball;
