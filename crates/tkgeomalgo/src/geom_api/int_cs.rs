// Rust translation of GeomAPI_IntCS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_IntCS`: the intersection points and segments of a 3D curve and a
//! surface.
use crate::int_curve_surface::{HCurve, HInter, IntersectionPoint};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::Point3;

/// `GeomAPI_IntCS`: IntCurveSurface_HInter on adaptors of the curve and the surface.
/// The points are the isolated intersections, the segments the parts of the curve
/// that lie on the surface (as trimmed curves of the given curve).
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::IntCS;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
/// let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
///     Point3::new(0.0, 0.0, 2.0), Direction3::new(0.0, 0.0, 1.0).unwrap())).cast();
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let inter = IntCS::with_curve(&line, &plane).unwrap();
/// assert_eq!(inter.nb_points().unwrap(), 1);
/// ```
#[derive(Default)]
pub struct IntCS {
    my_curve: Handle<dyn GeomCurve>,
    my_int_cs: HInter,
}

impl IntCS {
    /// `GeomAPI_IntCS()`: not done. `IntCS::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_IntCS(C, S)`. `IntCS::with_curve(&curve, &surface)?`.
    pub fn with_curve(
        c: &Handle<dyn GeomCurve>,
        s: &Handle<dyn GeomSurface>,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform(c, s)?;
        Ok(inter)
    }
    /// `Perform(C, S)`: the curve is kept for the segments; the adaptor exceptions
    /// (a null curve or surface) and those of IntCurveSurface_HInter are returned.
    /// `inter.perform(&curve, &surface)?`.
    pub fn perform(
        &mut self,
        c: &Handle<dyn GeomCurve>,
        s: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        self.my_curve = c.clone();
        let hc: HCurve = Handle::new(GeomAdaptorCurve::with_curve(c)?).cast();
        let hs: Adaptor3dSurfaceHandle = Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
        self.my_int_cs.perform(&hc, &hs)
    }
    /// `IsDone()`. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_int_cs.is_done()
    }
    /// `NbPoints()`: StdFail_NotDone when not done. `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.my_int_cs.nb_points()
    }
    /// `Point(Index)`: StdFail_NotDone when not done, Standard_OutOfRange outside
    /// `1..=NbPoints()`. `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.my_int_cs.point(index)?.pnt())
    }
    /// `Parameters(Index, U, V, W)`: the surface parameters and the curve parameter of
    /// the point. `let (u, v, w) = inter.parameters(1)?;`
    pub fn parameters(&self, index: i32) -> Result<(f64, f64, f64), Failure> {
        let the_point: IntersectionPoint = self.my_int_cs.point(index)?;
        Ok((the_point.u(), the_point.v(), the_point.w()))
    }
    /// `NbSegments()`: StdFail_NotDone when not done. `inter.nb_segments()?`.
    pub fn nb_segments(&self) -> Result<i32, Failure> {
        self.my_int_cs.nb_segments()
    }
    /// `Segment(Index)`: the curve trimmed between the curve parameters of the ends
    /// of the segment (the Geom_TrimmedCurve exceptions are returned).
    /// `inter.segment(1)?`.
    pub fn segment(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        let segment = self.my_int_cs.segment(index)?;
        let (first_point, last_point) = (segment.first_point(), segment.second_point());
        let the_curve =
            GeomTrimmedCurve::new(&self.my_curve, first_point.w(), last_point.w(), true, true)?;
        Ok(Handle::new(the_curve).cast())
    }
    /// `Parameters(Index, U1, V1, U2, V2)`: the surface parameters of the ends of the
    /// segment. `let (u1, v1, u2, v2) = inter.segment_parameters(1)?;`
    pub fn segment_parameters(&self, index: i32) -> Result<(f64, f64, f64, f64), Failure> {
        let segment = self.my_int_cs.segment(index)?;
        let (first_point, last_point) = (segment.first_point(), segment.second_point());
        Ok((
            first_point.u(),
            first_point.v(),
            last_point.u(),
            last_point.v(),
        ))
    }
}
