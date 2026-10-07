// Rust port of DataExchange/TKXSBase/XSControl/XSControl (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl`: the work session and the variables of a session pilot, for the
//! commands of the exchange.
use super::{XSControlVarsOps, XSControlWorkSession};
use crate::if_select::IFSelectSessionPilot;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};

/// `XSControl` (the package class).
pub struct XSControl;

impl XSControl {
    /// `Session(pilot)`: the work session of the pilot as an XSControl session (null when
    /// it is another kind); `Standard_NullObject` for a null pilot.
    ///
    /// ```
    /// use rustycad_tkxsbase::if_select::IFSelectSessionPilot;
    /// use rustycad_tkxsbase::xs_control::{XSControl, XSControlWorkSession};
    /// let pilot = IFSelectSessionPilot::new("test>").unwrap();
    /// pilot.set_session(&XSControlWorkSession::new().cast());
    /// assert!(XSControl::session(&pilot).unwrap().is_some());
    /// ```
    pub fn session(
        pilot: &Handle<IFSelectSessionPilot>,
    ) -> Result<Handle<XSControlWorkSession>, Failure> {
        let live = pilot
            .get()
            .ok_or_else(|| exceptions::null_object("XSControl::Session: null pilot"))?;
        Ok(live.session().cast())
    }
    /// `Vars(pilot)`: the variables of the XSControl session of the pilot (null without
    /// one).
    pub fn vars(
        pilot: &Handle<IFSelectSessionPilot>,
    ) -> Result<Handle<dyn XSControlVarsOps>, Failure> {
        let ws = Self::session(pilot)?;
        Ok(ws.get().map(XSControlWorkSession::vars).unwrap_or_default())
    }
}
