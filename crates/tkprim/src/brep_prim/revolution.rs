// Rust translation of BRepPrim_Revolution.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Revolution`: the one-axis primitive of a meridian curve.
use super::revolution_derived::lateral_face_on;
use super::{BRepPrimBuilder, BRepPrimOneAxis, BRepPrimOneAxisBase};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurfaceOfRevolution};
use rustycad_tkmath::gp::{Axis2, Point2, Transform3};

/// The Standard_NullObject of a member that dereferences a null meridian (a crash
/// natively).
fn null_meridian(member: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!(
            "BRepPrim_Revolution::{member}: the meridian is null; expected a meridian set by the constructor or Meridian()"
        ),
    )
}

/// `BRepPrim_Revolution`: a body of revolution of the meridian `M` (a 3D curve in the
/// XZ plane of the axes) and of its 2d image `PM` in that plane (docs/prim-design.md
/// section 2).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Axis2, Direction2, Direction3, Point2, Point3};
/// use rustycad_tkprim::brep_prim::{BRepPrimOneAxis, BRepPrimRevolution};
/// let meridian: Handle<dyn GeomCurve> =
///     Handle::new(GeomLine::from_point(Point3::new(1.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?)).cast();
/// let pmeridian: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0)?)).cast();
/// let mut cylinder = BRepPrimRevolution::new(Axis2::default(), 0.0, 2.0, meridian, pmeridian);
/// assert!(cylinder.has_top()?);
/// assert!(!cylinder.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimRevolution {
    base: BRepPrimOneAxisBase,
    my_meridian: Handle<dyn GeomCurve>,
    my_pmeridian: Handle<dyn Geom2dCurve>,
}

impl BRepPrimRevolution {
    /// `BRepPrim_Revolution(A, VMin, VMax, M, PM)`;
    /// `BRepPrimRevolution::new(axes, 0.0, 1.0, meridian, pmeridian)`.
    pub fn new(
        axes: Axis2,
        vmin: f64,
        vmax: f64,
        meridian: Handle<dyn GeomCurve>,
        pmeridian: Handle<dyn Geom2dCurve>,
    ) -> Self {
        let mut revolution = Self::without_meridian(axes, vmin, vmax);
        revolution.set_meridian(meridian, pmeridian);
        revolution
    }

    /// `BRepPrim_Revolution(A, VMin, VMax)` (protected): no meridian yet, the derived
    /// classes set it with [`Self::set_meridian`];
    /// `BRepPrimRevolution::without_meridian(axes, 0.0, 1.0)`.
    pub fn without_meridian(axes: Axis2, vmin: f64, vmax: f64) -> Self {
        Self {
            base: BRepPrimOneAxisBase::new(BRepPrimBuilder::new(), axes, vmin, vmax),
            my_meridian: Handle::default(),
            my_pmeridian: Handle::default(),
        }
    }

    /// `Meridian(M, PM)` (protected): sets the meridian curves;
    /// `revolution.set_meridian(meridian, pmeridian)`.
    pub fn set_meridian(
        &mut self,
        meridian: Handle<dyn GeomCurve>,
        pmeridian: Handle<dyn Geom2dCurve>,
    ) {
        self.my_meridian = meridian;
        self.my_pmeridian = pmeridian;
    }

    /// The 2d meridian, or Standard_NullObject when it is not set.
    fn pmeridian(&self, member: &str) -> Result<&dyn Geom2dCurve, Failure> {
        self.my_pmeridian.get().ok_or_else(|| null_meridian(member))
    }
}

impl BRepPrimOneAxis for BRepPrimRevolution {
    fn one_axis_base(&self) -> &BRepPrimOneAxisBase {
        &self.base
    }

    fn one_axis_base_mut(&mut self) -> &mut BRepPrimOneAxisBase {
        &mut self.base
    }

    /// `MakeEmptyLateralFace()`: a face on the `Geom_SurfaceOfRevolution` of the
    /// meridian around the axis.
    fn make_empty_lateral_face(&self) -> Result<Face, Failure> {
        let surface = GeomSurfaceOfRevolution::new(&self.my_meridian, self.axes().axis())?;
        lateral_face_on(self, Handle::new(surface).cast())
    }

    /// `MakeEmptyMeridianEdge(Ang)`: an edge on a copy of the meridian rotated by
    /// `angle` around the axis.
    fn make_empty_meridian_edge(&self, angle: f64) -> Result<Edge, Failure> {
        let meridian = self
            .my_meridian
            .get()
            .ok_or_else(|| null_meridian("MakeEmptyMeridianEdge"))?;
        let copy: Handle<dyn GeomCurve> = meridian.copy()?.cast();
        let mut rotation = Transform3::default();
        rotation.set_rotation(self.axes().axis(), angle)?;
        if let Some(curve) = copy.get() {
            curve.transform(rotation)?;
        }
        let mut edge = Edge::default();
        self.base.my_builder.builder().make_edge_with_curve(
            &mut edge,
            &copy,
            precision::confusion(),
        )?;
        Ok(edge)
    }

    /// `SetMeridianPCurve(E, F)`: the 2d meridian as the p-curve of `edge` on `face`.
    fn set_meridian_pcurve(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        self.base.my_builder.builder().update_edge_pcurve_on_face(
            edge,
            &self.my_pmeridian,
            face,
            precision::confusion(),
        )
    }

    /// `MeridianValue(V)`: the 2d meridian at `v`.
    fn meridian_value(&self, v: f64) -> Result<Point2, Failure> {
        self.pmeridian("MeridianValue")?.value(v)
    }
}
