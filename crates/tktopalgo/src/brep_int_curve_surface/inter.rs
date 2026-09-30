// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepIntCurveSurface/BRepIntCurveSurface_Inter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepIntCurveSurface_Inter`: the fields, the loading members and the accessors of the
//! current point; the exploration (`Find`, `FindPoint`) is in `find`
//! (docs/topalgo-design.md section 12).
use crate::brep_top_adaptor::BRepTopAdaptorTopolTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::int_curve_surface::{HInter, IntersectionPoint, TransitionOnCurve};
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Line3, Point3};

/// StdFail_NotDone: an accessor of the current point without a current point.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!(
            "BRepIntCurveSurface_Inter::{member}: no current point (index 0); expected More() with a point"
        ),
    )
}

/// Standard_OutOfRange: `Face()` with a face index outside the loaded faces.
fn face_out_of_range(index: i32, length: usize) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!("BRepIntCurveSurface_Inter::Face: face index {index} outside 1..={length}"),
    )
}

/// `BRepIntCurveSurface_Inter`: the intersections of a curve with the faces of a shape,
/// visited face after face, each point classified IN or ON its face.
///
/// ```
/// use rustycad_tktopalgo::brep_int_curve_surface::BRepIntCurveSurfaceInter;
/// let inter = BRepIntCurveSurfaceInter::new();
/// // A fresh object reports a current face (0 <= 0 faces), as natively.
/// assert!(inter.more());
/// assert!(inter.point().is_err());
/// ```
pub struct BRepIntCurveSurfaceInter {
    pub(super) tolerance: f64,
    pub(super) curve: Handle<GeomAdaptorCurve>,
    pub(super) intcs: HInter,
    pub(super) current_index: i32,
    pub(super) current_nb_points: i32,
    pub(super) fast_class: BRepTopAdaptorTopolTool,
    pub(super) current_state: State,
    pub(super) current_u: f64,
    pub(super) current_v: f64,
    pub(super) curve_box: BoundingBox3,
    pub(super) ind_face: i32,
    pub(super) faces: Vec<Face>,
    pub(super) face_boxes: Option<Vec<BoundingBox3>>,
}

impl Default for BRepIntCurveSurfaceInter {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepIntCurveSurfaceInter {
    /// `BRepIntCurveSurface_Inter()`: no shape, no curve; the tolerance, which the native
    /// constructor leaves uninitialized, is 0 until `Load`;
    /// `BRepIntCurveSurfaceInter::new()`.
    pub fn new() -> Self {
        let mut inter = Self {
            tolerance: 0.0,
            curve: Handle::default(),
            intcs: HInter::new(),
            current_index: 0,
            current_nb_points: 0,
            fast_class: BRepTopAdaptorTopolTool::new(),
            current_state: State::Unknown,
            current_u: 0.0,
            current_v: 0.0,
            curve_box: BoundingBox3::default(),
            ind_face: 0,
            faces: Vec::new(),
            face_boxes: None,
        };
        inter.clear();
        inter
    }

    /// `Init(theShape, theCurve, theTol)`: `Load`, then `Init(theCurve)`;
    /// `inter.init_curve(&shape, &curve, 1e-7)?`.
    pub fn init_curve(
        &mut self,
        shape: &topods::Shape,
        curve: &GeomAdaptorCurve,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.load(shape, tolerance)?;
        self.init(curve)
    }

    /// `Init(theShape, theLine, theTol)`: the curve is a `Geom_Line` on `line`;
    /// `inter.init_line(&shape, line, 1e-7)?`.
    pub fn init_line(
        &mut self,
        shape: &topods::Shape,
        line: Line3,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let geomline: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(line)).cast();
        let curve = GeomAdaptorCurve::with_curve(&geomline)?;
        self.load(shape, tolerance)?;
        self.init(&curve)
    }

    /// `Clear()` (protected natively): no current point and no current face.
    pub(super) fn clear(&mut self) {
        self.current_index = 0;
        self.current_nb_points = 0;
        self.ind_face = 0;
        self.current_state = State::Unknown;
        self.current_u = 0.0;
        self.current_v = 0.0;
    }

    /// `Load(theShape, theTol)`: the faces of `shape` in exploration order, their boxes
    /// dropped, and the classification tolerance; `inter.load(&shape, 1e-7)?`.
    pub fn load(&mut self, shape: &topods::Shape, tolerance: f64) -> Result<(), Failure> {
        self.clear();
        self.faces.clear();
        self.face_boxes = None;
        self.tolerance = tolerance;
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.faces.push(topods::face(&face)?);
        }
        Ok(())
    }

    /// `Init(theCurve)`: intersects a copy of `curve` with the loaded faces and moves to
    /// the first point IN or ON a face; the curve box is built for a finite curve only;
    /// `inter.init(&curve)?`.
    pub fn init(&mut self, curve: &GeomAdaptorCurve) -> Result<(), Failure> {
        self.clear();
        self.curve_box.set_void();
        let first: f64 = curve.first_parameter()?;
        let last: f64 = curve.last_parameter()?;
        // The native copy constructor shares the curve (and the evaluation cache, which
        // changes no value); the shallow copy shares the curve.
        self.curve = Handle::new(curve.shallow_adaptor()?);
        if !precision::is_infinite(first) && !precision::is_infinite(last) {
            Add3dCurve::add(&*self.curve, 0.0, &mut self.curve_box)?;
        }
        self.find()
    }

    /// `More()`: whether the face index is at most the face count (true on a fresh
    /// object); `inter.more()`.
    pub fn more(&self) -> bool {
        self.ind_face as usize <= self.faces.len()
    }

    /// `Next()`: the next point IN or ON a face, on this face or a next one;
    /// `inter.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, which can fail.
    pub fn next(&mut self) -> Result<(), Failure> {
        if self.current_nb_points != 0 {
            self.current_index += 1;
        }
        self.find()
    }

    /// The intersection point of the current index.
    fn current_point(&self, member: &str) -> Result<IntersectionPoint, Failure> {
        self.current_point_check(member)?;
        self.intcs.point(self.current_index)
    }

    /// `Point()`: the current point, with the parameters of the classified point;
    /// StdFail_NotDone without one; `inter.point()?`.
    pub fn point(&self) -> Result<IntersectionPoint, Failure> {
        let icps: IntersectionPoint = self.current_point("Point")?;
        Ok(IntersectionPoint::new(
            icps.pnt(),
            self.current_u,
            self.current_v,
            icps.w(),
            icps.transition(),
        ))
    }

    /// `Pnt()`: the current 3d point; `inter.pnt()?`.
    pub fn pnt(&self) -> Result<Point3, Failure> {
        Ok(self.current_point("Pnt")?.pnt())
    }

    /// `U()`: the U parameter of the current point on its face; `inter.u()?`.
    pub fn u(&self) -> Result<f64, Failure> {
        self.current_point_check("U")?;
        Ok(self.current_u)
    }

    /// `V()`: the V parameter of the current point on its face; `inter.v()?`.
    pub fn v(&self) -> Result<f64, Failure> {
        self.current_point_check("V")?;
        Ok(self.current_v)
    }

    /// `W()`: the parameter of the current point on the curve; `inter.w()?`.
    pub fn w(&self) -> Result<f64, Failure> {
        Ok(self.current_point("W")?.w())
    }

    /// `State()`: the state of the current point on its face (IN or ON);
    /// `inter.state()?`.
    pub fn state(&self) -> Result<State, Failure> {
        self.current_point_check("State")?;
        Ok(self.current_state)
    }

    /// `Transition()`: the transition of the curve at the current point;
    /// `inter.transition()?`.
    pub fn transition(&self) -> Result<TransitionOnCurve, Failure> {
        Ok(self.current_point("Transition")?.transition())
    }

    /// `Face()`: the current face; Standard_OutOfRange when the face index is outside
    /// the loaded faces (before the first `Init`, or after the last face);
    /// `inter.face()?`.
    pub fn face(&self) -> Result<Face, Failure> {
        let index: i32 = self.ind_face;
        if index < 1 || index as usize > self.faces.len() {
            return Err(face_out_of_range(index, self.faces.len()));
        }
        Ok(self.faces[index as usize - 1].clone())
    }

    /// StdFail_NotDone without a current point (the members that read the fields).
    fn current_point_check(&self, member: &str) -> Result<(), Failure> {
        if self.current_index == 0 {
            return Err(not_done(member));
        }
        Ok(())
    }

    /// The curve as the adaptor handle of the intersector.
    pub(super) fn curve_handle(&self) -> Handle<dyn Adaptor3dCurve> {
        self.curve.clone().cast()
    }
}
