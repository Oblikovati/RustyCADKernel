// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Params.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Params`: the display defaults of new geometric variables.
use crate::draw::{DrawColor, DrawColorKind, DrawMarkerShape, DrawSession};
use std::cell::RefCell;
use std::rc::Rc;

/// `DrawTrSurf_Params`: the colors, markers and discretisation that
/// `DrawTrSurf::Set` and the restore factories give new drawables.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawTrSurfParams {
    /// `PntColor`.
    pub pnt_color: DrawColor,
    /// `CurvColor`.
    pub curv_color: DrawColor,
    /// `BoundsColor`.
    pub bounds_color: DrawColor,
    /// `IsosColor`.
    pub isos_color: DrawColor,
    /// `PolesColor`.
    pub poles_color: DrawColor,
    /// `KnotsColor`.
    pub knots_color: DrawColor,
    /// `PntMarker`.
    pub pnt_marker: DrawMarkerShape,
    /// `KnotsMarker`.
    pub knots_marker: DrawMarkerShape,
    /// `IsShowPoles`.
    pub is_show_poles: bool,
    /// `IsShowKnots`.
    pub is_show_knots: bool,
    /// `NeedKnotsIsos`: BSpline surfaces draw their knot isos instead of `NbUIsos`.
    pub need_knots_isos: bool,
    /// `Deflection`.
    pub deflection: f64,
    /// `KnotsSize`.
    pub knots_size: i32,
    /// `Discret`.
    pub discret: i32,
    /// `DrawMode`.
    pub draw_mode: i32,
    /// `NbUIsos`.
    pub nb_u_isos: i32,
    /// `NbVIsos`.
    pub nb_v_isos: i32,
}

impl Default for DrawTrSurfParams {
    /// `DrawTrSurf_Params()`.
    fn default() -> Self {
        Self {
            pnt_color: DrawColor::new(DrawColorKind::Rouge),
            curv_color: DrawColor::new(DrawColorKind::Jaune),
            bounds_color: DrawColor::new(DrawColorKind::Vert),
            isos_color: DrawColor::new(DrawColorKind::Bleu),
            poles_color: DrawColor::new(DrawColorKind::Rouge),
            knots_color: DrawColor::new(DrawColorKind::Violet),
            pnt_marker: DrawMarkerShape::Plus,
            knots_marker: DrawMarkerShape::Losange,
            is_show_poles: true,
            is_show_knots: true,
            need_knots_isos: true,
            deflection: 0.01,
            knots_size: 5,
            discret: 30,
            draw_mode: 0,
            nb_u_isos: 10,
            nb_v_isos: 10,
        }
    }
}

/// `DrawTrSurf::Parameters()`: the parameters of `session` (a function-local static
/// in C++).
///
/// ```
/// use rustycad_tkdraw::draw::{DrawSession, DrawViewer, RecordingConsole};
/// use rustycad_tkdraw::draw_tr_surf::parameters;
/// use std::rc::Rc;
/// let session = DrawSession::new(Rc::new(RecordingConsole::default()), DrawViewer::batch());
/// parameters(&session).borrow_mut().discret = 50;
/// assert_eq!(parameters(&session).borrow().discret, 50);
/// ```
pub fn parameters(session: &DrawSession) -> Rc<RefCell<DrawTrSurfParams>> {
    session.package_state::<RefCell<DrawTrSurfParams>>()
}

/// The current parameters (a copy).
pub(crate) fn current(session: &DrawSession) -> DrawTrSurfParams {
    *parameters(session).borrow()
}
