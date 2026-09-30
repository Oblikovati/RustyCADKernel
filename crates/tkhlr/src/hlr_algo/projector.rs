// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_Projector.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_Projector: the parallel or perspective projection of the hidden line
//! algorithm.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Axis3, Point3, Transform3, Vector2, Vector3};

/// HLRAlgo_Projector: a viewing transformation (the scaled one given by the caller and
/// the unscaled one used to project, with its inverse), a perspective focus, and three
/// 2D directions: the normals of the projected model axes. For a parallel projection
/// without translation the unscaled transformation is classified as one of four
/// standard views (top, front, axonometric, the axonometric of type 0), which
/// `project` and `project_xyz` evaluate with their own formulas.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::Projector;
/// use rustycad_tkmath::gp::{Point2, Point3};
/// let top = Projector::new().unwrap();
/// let mut projected = Point2::default();
/// top.project(Point3::new(1.0, 2.0, 3.0), &mut projected);
/// assert_eq!((projected.x(), projected.y()), (1.0, 2.0));
/// assert_eq!(top.project_xyz(Point3::new(1.0, 2.0, 3.0)), (1.0, 2.0, 3.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projector {
    kind: i32,
    persp: bool,
    focus: f64,
    scaled_trsf: Transform3,
    trsf: Transform3,
    inv_trsf: Transform3,
    d1: Vector2,
    d2: Vector2,
    d3: Vector2,
}

impl Projector {
    /// The fields of every constructor before `Scaled` classifies the transformation.
    fn raw(trsf: Transform3, persp: bool, focus: f64) -> Self {
        Self {
            kind: -1,
            persp,
            focus,
            scaled_trsf: trsf,
            trsf,
            inv_trsf: trsf,
            d1: Vector2::default(),
            d2: Vector2::default(),
            d3: Vector2::default(),
        }
    }

    /// HLRAlgo_Projector(): the identity parallel projection (the top view); the
    /// directions stay zero, as natively (no SetDirection); `Projector::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let mut projector = Self::raw(Transform3::default(), false, 0.0);
        projector.scaled(false)?;
        Ok(projector)
    }

    /// HLRAlgo_Projector(CS): the parallel projection on the frame `cs`;
    /// `Projector::with_frame(frame)?`.
    pub fn with_frame(cs: Axis2) -> Result<Self, Failure> {
        Self::with_frame_projection(cs, false, 0.0)
    }

    /// HLRAlgo_Projector(CS, Focus): the perspective projection on the frame `cs` with
    /// the eye at `focus` on its Z axis; `Projector::with_frame_focus(frame, 10.0)?`.
    pub fn with_frame_focus(cs: Axis2, focus: f64) -> Result<Self, Failure> {
        Self::with_frame_projection(cs, true, focus)
    }

    fn with_frame_projection(cs: Axis2, persp: bool, focus: f64) -> Result<Self, Failure> {
        let mut trsf = Transform3::default();
        trsf.set_transformation_to(Axis3::from_axis2(cs));
        Self::with_transformation(trsf, persp, focus)
    }

    /// HLRAlgo_Projector(T, Persp, Focus); `Projector::with_transformation(t, false, 0.0)?`.
    pub fn with_transformation(t: Transform3, persp: bool, focus: f64) -> Result<Self, Failure> {
        let mut projector = Self::raw(t, persp, focus);
        projector.scaled(false)?;
        projector.set_direction();
        Ok(projector)
    }

    /// HLRAlgo_Projector(T, Persp, Focus, v1, v2, v3): with the given directions
    /// instead of computed ones; `Projector::with_directions(t, false, 0.0, [v1, v2, v3])?`.
    pub fn with_directions(
        t: Transform3,
        persp: bool,
        focus: f64,
        directions: [Vector2; 3],
    ) -> Result<Self, Failure> {
        let mut projector = Self::raw(t, persp, focus);
        [projector.d1, projector.d2, projector.d3] = directions;
        projector.scaled(false)?;
        Ok(projector)
    }

    /// Set(T, Persp, Focus): replaces the projection and recomputes the directions;
    /// `projector.set(t, true, 10.0)?`.
    pub fn set(&mut self, t: Transform3, persp: bool, focus: f64) -> Result<(), Failure> {
        self.persp = persp;
        self.focus = focus;
        self.scaled_trsf = t;
        self.scaled(false)?;
        self.set_direction();
        Ok(())
    }

    /// Directions: the normals of the projected X, Y and Z axes; `projector.directions()`.
    pub fn directions(&self) -> (Vector2, Vector2, Vector2) {
        (self.d1, self.d2, self.d3)
    }

    /// Scaled(On): with `on` the projection uses the scaled transformation as given;
    /// otherwise its scale is reset to 1 and, for a parallel projection, its
    /// translation to zero, and the standard view is recognized. The inverse follows;
    /// `projector.scaled(false)?`.
    pub fn scaled(&mut self, on: bool) -> Result<(), Failure> {
        self.kind = -1;
        self.trsf = self.scaled_trsf;
        if !on {
            self.trsf.set_scale_factor(1.0)?;
            if !self.persp {
                self.trsf.set_translation_part(Vector3::new(0.0, 0.0, 0.0));
                self.kind = trsf_type(&self.trsf);
            }
        }
        self.inv_trsf = self.trsf.inverted()?;
        Ok(())
    }

    /// Perspective; `projector.perspective()`.
    pub fn perspective(&self) -> bool {
        self.persp
    }

    /// Transformation: the unscaled transformation used to project;
    /// `projector.transformation()`.
    pub fn transformation(&self) -> Transform3 {
        self.trsf
    }

    /// InvertedTransformation; `projector.inverted_transformation()`.
    pub fn inverted_transformation(&self) -> Transform3 {
        self.inv_trsf
    }

    /// FullTransformation: the scaled transformation as given;
    /// `projector.full_transformation()`.
    pub fn full_transformation(&self) -> Transform3 {
        self.scaled_trsf
    }

    /// Focus: the perspective focus; Standard_NoSuchObject for a parallel projection;
    /// `projector.focus()?`.
    pub fn focus(&self) -> Result<f64, Failure> {
        if !self.persp {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "HLRAlgo_Projector::Not a Perspective",
            ));
        }
        Ok(self.focus)
    }

    /// Transform(gp_Vec&): applies the unscaled transformation to `d`;
    /// `projector.transform_vector(&mut v)`.
    pub fn transform_vector(&self, d: &mut Vector3) {
        d.transform(self.trsf);
    }

    /// Transform(gp_Pnt&): applies the unscaled transformation to `pnt`;
    /// `projector.transform_point(&mut p)`.
    pub fn transform_point(&self, pnt: &mut Point3) {
        pnt.transform(self.trsf);
    }

    /// SetDirection: the directions are the normals `(-y, x)` of the projected unit
    /// axes; an axis projected on a point (`|x| + |y|` below the angular precision)
    /// is replaced by `(1, 1)` first.
    fn set_direction(&mut self) {
        self.d1 = self.axis_normal(Vector3::new(1.0, 0.0, 0.0));
        self.d2 = self.axis_normal(Vector3::new(0.0, 1.0, 0.0));
        self.d3 = self.axis_normal(Vector3::new(0.0, 0.0, 1.0));
    }

    fn axis_normal(&self, mut axis: Vector3) -> Vector2 {
        axis.transform(self.trsf);
        if axis.x().abs() + axis.y().abs() < precision::angular() {
            axis = Vector3::new(1.0, 1.0, 0.0);
        }
        Vector2::new(-axis.y(), axis.x())
    }

    /// The perspective ratio `1 - z / focus`.
    pub(super) fn perspective_ratio(&self, z: f64) -> f64 {
        1.0 - z / self.focus
    }

    pub(super) fn kind(&self) -> i32 {
        self.kind
    }

    pub(super) fn inner_focus(&self) -> f64 {
        self.focus
    }
}

/// The standard view of an unscaled transformation without translation (the file
/// function TrsfType): 1 top, 2 front, 3 axonometric, 0 the axonometric of type 0,
/// -1 any other, each recognized on its coefficients at 1e-15.
fn trsf_type(trsf: &Transform3) -> i32 {
    let m = trsf.vectorial_part();
    let c = m.coefficients();
    let near = |value: f64, target: f64| (value - target).abs() < 1e-15;
    // FRAC_1_SQRT_2 is the native literal 0.7071067811865476, bit for bit.
    let (r2, h) = (std::f64::consts::FRAC_1_SQRT_2, 0.5);
    if near(c[0][0], 1.0) && near(c[1][1], 1.0) && near(c[2][2], 1.0) {
        return 1;
    }
    let type0 = [[r2, -h, h], [r2, h, -h], [0.0, r2, r2]];
    if (0..3).all(|i| (0..3).all(|j| near(c[i][j], type0[i][j]))) {
        return 0;
    }
    if near(c[0][0], 1.0) && near(c[1][2], 1.0) && near(c[2][1], -1.0) {
        return 2;
    }
    let axo = [[r2, r2, 0.0], [-h, h, r2], [h, -h, r2]];
    if (0..3).all(|i| (0..3).all(|j| near(c[i][j], axo[i][j]))) {
        return 3;
    }
    -1
}
