// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_PntFace.hxx and
// LocOpe_Operation.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_PntFace`: an intersection point of a curve with a face, and the
//! `LocOpe_Operation` enumeration.
use rustycad_tkbrep::topods::Face;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

/// `LocOpe_Operation`: the Boolean operation of a local operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    /// `LocOpe_FUSE`.
    Fuse,
    /// `LocOpe_CUT`.
    Cut,
    /// `LocOpe_INVALID`.
    Invalid,
}

/// `LocOpe_PntFace`: the point, the face it lies on, the transition of the curve
/// (FORWARD entering, REVERSED leaving, EXTERNAL tangent), the curve parameter and the
/// (u, v) of the point on the face.
///
/// ```
/// use rustycad_tkfeat::loc_ope::PntFace;
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkmath::gp::Point3;
/// let p = PntFace::new(Point3::new(1., 2., 3.), Face::default(), Orientation::Forward, 0.5, 0.1, 0.2);
/// assert_eq!((p.parameter(), p.u_parameter(), p.v_parameter()), (0.5, 0.1, 0.2));
/// ```
#[derive(Clone, Debug)]
pub struct PntFace {
    pnt: Point3,
    face: Face,
    ori: Orientation,
    par: f64,
    u_par: f64,
    v_par: f64,
}

impl Default for PntFace {
    /// `LocOpe_PntFace()`: the origin, a null face and zero parameters. The native
    /// orientation is left uninitialized; the port starts at FORWARD.
    fn default() -> Self {
        Self {
            pnt: Point3::default(),
            face: Face::default(),
            ori: Orientation::Forward,
            par: 0.0,
            u_par: 0.0,
            v_par: 0.0,
        }
    }
}

impl PntFace {
    /// `LocOpe_PntFace(P, F, Or, Param, UPar, VPar)`;
    /// `PntFace::new(p, face, Orientation::Forward, w, u, v)`.
    pub fn new(p: Point3, f: Face, or: Orientation, param: f64, u_par: f64, v_par: f64) -> Self {
        Self {
            pnt: p,
            face: f,
            ori: or,
            par: param,
            u_par,
            v_par,
        }
    }

    /// `Pnt()`; `point.pnt()`.
    pub fn pnt(&self) -> &Point3 {
        &self.pnt
    }

    /// `Face()`; `point.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }

    /// `Orientation()`; `point.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.ori
    }

    /// `ChangeOrientation()`; `*point.change_orientation() = Orientation::Reversed`.
    pub fn change_orientation(&mut self) -> &mut Orientation {
        &mut self.ori
    }

    /// `Parameter()`: the parameter on the curve; `point.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.par
    }

    /// `UParameter()`; `point.u_parameter()`.
    pub fn u_parameter(&self) -> f64 {
        self.u_par
    }

    /// `VParameter()`; `point.v_parameter()`.
    pub fn v_parameter(&self) -> f64 {
        self.v_par
    }
}

/// Inserts `point` before the first point of `points` with a greater parameter (after
/// the points of equal parameter), keeping `points` sorted along the curve.
pub(crate) fn insert_sorted(points: &mut Vec<PntFace>, point: PntFace) {
    let at = points
        .iter()
        .position(|p| p.parameter() - point.parameter() > 0.)
        .unwrap_or(points.len());
    points.insert(at, point);
}
