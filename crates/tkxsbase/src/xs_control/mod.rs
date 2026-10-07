//! XSControl: the control of the data exchange: the norm controllers, the work session
//! of a norm with its transfer reader and writer, the reader front end, the shape tools
//! and the commands of the exchange.
mod utils;
mod utils_shapes;

pub use utils::{XSControlExtTextSeq, XSControlTextSeq, XSControlTransientSeq, XSControlUtils};
pub use utils_shapes::XSControlShapeSeq;
#[cfg(test)]
mod utils_tests;
