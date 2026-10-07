// Rust translation of Draw/TKDraw/DBRep/DBRep_Params.hxx and the parameter accessors
// of DBRep.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DBRep_Params`: the display defaults of new shape variables.
use crate::draw::DrawSession;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

/// `DBRep_Params`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DBRepParams {
    /// `NbIsos`: the number of isos in U and V.
    pub nb_isos: i32,
    /// `Size`: the length drawn for infinite edges and isos.
    pub size: f64,
    /// `Discretization`: the number of points of curves.
    pub discretization: i32,
    /// `DispTriangles`.
    pub disp_triangles: bool,
    /// `DisplayPolygons`.
    pub display_polygons: bool,
    /// `HLRAngle`: the discretisation angle of the hidden-line removal.
    pub hlr_angle: f64,
    /// `HAngMin`.
    pub h_ang_min: f64,
    /// `HAngMax`.
    pub h_ang_max: f64,
    /// `WithHLR`: hidden-line removal instead of wireframe.
    pub with_hlr: bool,
    /// `WithRg1`: the smooth (G1) edges.
    pub with_rg1: bool,
    /// `WithRgN`: the seam-like (CN) edges.
    pub with_rgn: bool,
    /// `WithHid`: the hidden lines.
    pub with_hid: bool,
}

impl Default for DBRepParams {
    /// `DBRep_Params()`.
    fn default() -> Self {
        Self {
            nb_isos: 2,
            size: 100.0,
            discretization: 30,
            disp_triangles: false,
            display_polygons: false,
            hlr_angle: 35.0 * PI / 180.0,
            h_ang_min: 1.0 * PI / 180.0,
            h_ang_max: 35.0 * PI / 180.0,
            with_hlr: false,
            with_rg1: true,
            with_rgn: false,
            with_hid: false,
        }
    }
}

/// `DBRep::Parameters()`: the parameters of `session` (a function-local static in
/// C++).
///
/// ```
/// use rustycad_tkdraw::dbrep::parameters;
/// use rustycad_tkdraw::draw::{DrawSession, DrawViewer, RecordingConsole};
/// use std::rc::Rc;
/// let session = DrawSession::new(Rc::new(RecordingConsole::default()), DrawViewer::batch());
/// assert_eq!(parameters(&session).borrow().nb_isos, 2);
/// ```
pub fn parameters(session: &DrawSession) -> Rc<RefCell<DBRepParams>> {
    session.package_state::<RefCell<DBRepParams>>()
}

/// The current parameters (a copy).
pub(crate) fn current(session: &DrawSession) -> DBRepParams {
    *parameters(session).borrow()
}

impl DrawSession {
    /// `DBRep::HLRMode`.
    pub fn dbrep_hlr_mode(&self) -> bool {
        current(self).with_hlr
    }

    /// `DBRep::Rg1Mode`.
    pub fn dbrep_rg1_mode(&self) -> bool {
        current(self).with_rg1
    }

    /// `DBRep::RgNMode`.
    pub fn dbrep_rgn_mode(&self) -> bool {
        current(self).with_rgn
    }

    /// `DBRep::HidMode`.
    pub fn dbrep_hid_mode(&self) -> bool {
        current(self).with_hid
    }

    /// `DBRep::HLRAngle`.
    pub fn dbrep_hlr_angle(&self) -> f64 {
        current(self).hlr_angle
    }

    /// `DBRep::NbIsos`.
    pub fn dbrep_nb_isos(&self) -> i32 {
        current(self).nb_isos
    }

    /// `DBRep::Discretisation`.
    pub fn dbrep_discretisation(&self) -> i32 {
        current(self).discretization
    }
}
