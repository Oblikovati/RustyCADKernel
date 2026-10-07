//! XSControl: the control of the data exchange: the norm controllers, the work session
//! of a norm with its transfer reader and writer, the reader front end, the shape tools
//! and the commands of the exchange.
mod controller;
mod controller_customise;
mod package;
mod reader;
mod reader_transfer;
mod selections;
mod sign_transfer_status;
mod transfer_reader;
mod transfer_reader_checks;
mod transfer_reader_results;
mod transfer_reader_stats;
mod transfer_reader_transfer;
mod transfer_writer;
mod utils;
mod utils_shapes;
mod vars;
mod work_session;
mod work_session_status;
mod work_session_transfer;

pub use controller::{XSControlController, XSControlControllerBase};
pub use package::XSControl;
pub use reader::{XSControlReader, XSControlTransferStats};
pub use reader_transfer::XSControlReaderOps;
pub use selections::{XSControlConnectedShapes, XSControlSelectForTransfer};
pub use sign_transfer_status::XSControlSignTransferStatus;
pub use transfer_reader::{XSControlContext, XSControlTransferReader};
pub use transfer_writer::XSControlTransferWriter;
pub use utils::{XSControlExtTextSeq, XSControlTextSeq, XSControlTransientSeq, XSControlUtils};
pub use utils_shapes::XSControlShapeSeq;
pub use vars::{XSControlVars, XSControlVarsOps};
pub use work_session::XSControlWorkSession;
#[cfg(test)]
mod controller_tests;
#[cfg(test)]
mod reader_tests;
#[cfg(test)]
mod test_norm;
#[cfg(test)]
mod transfer_tests;
#[cfg(test)]
mod utils_tests;
#[cfg(test)]
mod work_session_tests;
