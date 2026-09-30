// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Face.hxx/.lxx/.cxx
// (the loads, bounds and evaluators) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_Face`: the face and boundary-curve tool of the surface and volume
//! integrators (docs/topalgo-design.md section 9).
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// `BRepGProp_Face`: a face (loaded FORWARD, its orientation kept for the normal) and
/// its current boundary curve, with the integration orders and knots the Gauss and
/// Gauss-Kronrod integrators read.
///
/// `Clone` is the native copy of the boundary curve; the copy shares the surface
/// adaptor (docs/topalgo-design.md section 9).
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::BRepGPropFace;
/// let tool = BRepGPropFace::new(false);
/// assert!(tool.get_face().is_null());
/// ```
pub struct BRepGPropFace {
    pub(super) surface: Arc<BRepAdaptorSurface>,
    pub(super) curve: Geom2dAdaptorCurve,
    face_surface: Handle<dyn GeomSurface>,
    face_location: Location,
    is_face_context_ready: bool,
    s_reverse: bool,
    pub(super) is_use_span: bool,
}

impl Clone for BRepGPropFace {
    fn clone(&self) -> Self {
        Self {
            surface: Arc::clone(&self.surface),
            curve: copy_curve(&self.curve),
            face_surface: self.face_surface.clone(),
            face_location: self.face_location.clone(),
            is_face_context_ready: self.is_face_context_ready,
            s_reverse: self.s_reverse,
            is_use_span: self.is_use_span,
        }
    }
}

/// The copy of a `Geom2dAdaptor_Curve` (its curve and range; the evaluation cache is
/// rebuilt on demand); an unloaded adaptor copies as unloaded.
fn copy_curve(curve: &Geom2dAdaptorCurve) -> Geom2dAdaptorCurve {
    curve.shallow_adaptor().unwrap_or_default()
}

impl BRepGPropFace {
    /// `BRepGProp_Face(IsUseSpan)`: a tool without face; `is_use_span` asks for the
    /// knot spans of B-spline faces (`get_u_knots`, `get_t_knots`);
    /// `BRepGPropFace::new(false)`.
    pub fn new(is_use_span: bool) -> Self {
        Self {
            surface: Arc::new(BRepAdaptorSurface::default()),
            curve: Geom2dAdaptorCurve::default(),
            face_surface: Handle::default(),
            face_location: Location::default(),
            is_face_context_ready: false,
            s_reverse: false,
            is_use_span,
        }
    }

    /// `BRepGProp_Face(F, IsUseSpan)`: the tool loaded with `face`;
    /// `BRepGPropFace::with_face(&face, false)?`.
    pub fn with_face(face: &Face, is_use_span: bool) -> Result<Self, Failure> {
        let mut tool = Self::new(is_use_span);
        tool.load_face(face)?;
        Ok(tool)
    }

    /// `Load(F)`: the adaptor initialized on the FORWARD face (restricted to its UV
    /// bounds; a null face is ignored and a face without surface keeps the previous
    /// surface, as `BRepAdaptor_Surface::Initialize` does), the face surface and location
    /// for the pcurves, and the orientation of `face` for the normal;
    /// `tool.load_face(&face)?`.
    pub fn load_face(&mut self, face: &Face) -> Result<(), Failure> {
        let forward = Face::unchecked(face.oriented(Orientation::Forward));
        self.surface.initialize(&forward, true)?;
        let (surface, location) = BRepTool::surface_with_location(&self.surface.face())?;
        self.is_face_context_ready = !surface.is_null();
        self.face_surface = surface;
        self.face_location = location;
        self.s_reverse = face.orientation() == Orientation::Reversed;
        Ok(())
    }

    /// `NaturalRestriction()`: the natural restriction flag of the face;
    /// `tool.natural_restriction()?`.
    pub fn natural_restriction(&self) -> Result<bool, Failure> {
        BRepTool::natural_restriction(&self.surface.face())
    }

    /// `GetFace()`: the loaded face, FORWARD (the orientation of the face given to
    /// `load_face` is not kept here); `tool.get_face()`.
    pub fn get_face(&self) -> Face {
        self.surface.face()
    }

    /// `Value2d(U)`: the point of the boundary curve; `tool.value2d(0.5)?`.
    pub fn value2d(&self, u: f64) -> Result<Point2, Failure> {
        self.curve.value(u)
    }

    /// `D12d(U, P, V1)`: the point and first derivative of the boundary curve;
    /// `let (p, v) = tool.d12d(0.5)?;`.
    pub fn d12d(&self, u: f64) -> Result<(Point2, Vector2), Failure> {
        self.curve.d1(u)
    }

    /// `FirstParameter()` of the boundary curve; `tool.first_parameter()?`.
    pub fn first_parameter(&self) -> Result<f64, Failure> {
        self.curve.first_parameter()
    }

    /// `LastParameter()` of the boundary curve; `tool.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        self.curve.last_parameter()
    }

    /// `Bounds(U1, U2, V1, V2)`: the parametric bounds of the adaptor;
    /// `let (u1, u2, v1, v2) = tool.bounds()?;`.
    pub fn bounds(&self) -> Result<(f64, f64, f64, f64), Failure> {
        Ok((
            self.surface.first_u_parameter()?,
            self.surface.last_u_parameter()?,
            self.surface.first_v_parameter()?,
            self.surface.last_v_parameter()?,
        ))
    }

    /// `Normal(U, V, P, VNor)`: the point and `D1U ^ D1V`, reversed for a REVERSED
    /// face; `let (p, n) = tool.normal(0.5, 0.5)?;`.
    pub fn normal(&self, u: f64, v: f64) -> Result<(Point3, Vector3), Failure> {
        let d1 = self.surface.d1(u, v)?;
        let mut normal = d1.d1u.crossed(d1.d1v);
        if self.s_reverse {
            normal.reverse();
        }
        Ok((d1.point, normal))
    }

    /// `Load(E)`: the pcurve of `edge` on the face surface as the boundary curve
    /// (reversed with its range for a REVERSED edge); false without face surface or
    /// pcurve; `tool.load_edge(&edge)?`.
    pub fn load_edge(&mut self, edge: &Edge) -> Result<bool, Failure> {
        if !self.is_face_context_ready {
            return Ok(false);
        }
        let found = BRepTool::curve_on_surface(edge, &self.face_surface, &self.face_location)?;
        let Some(pcurve) = found.pcurve.get() else {
            return Ok(false);
        };
        let (mut a, mut b) = (found.first, found.last);
        let mut curve: Handle<dyn Geom2dCurve> = found.pcurve.clone();
        if edge.orientation() == Orientation::Reversed {
            let x = a;
            a = pcurve.reversed_parameter(b)?;
            b = pcurve.reversed_parameter(x)?;
            curve = pcurve.reversed()?;
        }
        self.curve.load_range(&curve, a, b)?;
        Ok(true)
    }

    /// `Load(IsFirstParam, theIsoType)`: a side of the UV rectangle as the boundary
    /// curve, a line on `[0, length]`: the left (`IsoU`, first) or right side, the
    /// bottom (`IsoV`, first) or top side, running counterclockwise; nothing for
    /// `NoneIso`; `tool.load_iso(false, IsoType::IsoU)?`.
    pub fn load_iso(&mut self, is_first_param: bool, iso_type: IsoType) -> Result<(), Failure> {
        let (u1, u2, v1, v2) = self.bounds()?;
        let (length, location, direction) = match (iso_type, is_first_param) {
            (IsoType::IsoU, true) => (v2 - v1, Point2::new(u1, v2), (0., -1.)),
            (IsoType::IsoU, false) => (v2 - v1, Point2::new(u2, v1), (0., 1.)),
            (IsoType::IsoV, true) => (u2 - u1, Point2::new(u1, v1), (1., 0.)),
            (IsoType::IsoV, false) => (u2 - u1, Point2::new(u2, v2), (-1., 0.)),
            _ => return Ok(()),
        };
        let direction = Direction2::new(direction.0, direction.1)?;
        let line: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::from_point(location, direction)).cast();
        self.curve.load_range(&line, 0., length)
    }
}
