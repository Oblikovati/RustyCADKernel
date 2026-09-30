// Rust translation of IntCurveSurface_ThePolygonOfHInter.hxx/.cxx and
// IntCurveSurface_ThePolygonToolOfHInter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_ThePolygonOfHInter` and its tool, generic over the curve tool
//! (docs/geomalgo-design.md section 11).
use super::dump::general;
use super::failures::out_of_bounds;
use super::polygon_utils::{PolygonSamples, approx_param_on_curve, init_uniform, init_with_params};
use super::tools::InterCurveTool;
use crate::intf::Polygon3dTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicI32, Ordering};

/// The polygon of a curve: its sampled points, the parameter range they cover, the
/// box and the deflection estimate. `IntCurveSurface_ThePolygonOfHInter` is
/// [`ThePolygonOfHInter`](super::ThePolygonOfHInter), the instance on
/// `IntCurveSurface_TheHCurveTool`.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
/// use rustycad_tkgeomalgo::int_curve_surface::{HCurve, ThePolygonOfHInter};
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let line: Handle<dyn GeomCurve> =
///     Handle::new(GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), Direction3::new(1.0, 0.0, 0.0).unwrap())).cast();
/// let curve: HCurve = Handle::new(GeomAdaptorCurve::with_range(&line, 0.0, 4.0).unwrap()).cast();
/// let polygon = ThePolygonOfHInter::new(&curve, 3).unwrap();
/// assert_eq!(polygon.nb_segments(), 4);
/// ```
pub struct InterPolygon<C: InterCurveTool> {
    the_bnd: BoundingBox3,
    the_deflection: f64,
    nb_pnt_in: i32,
    the_pnts: Vec<Point3>,
    closed_polygon: bool,
    binf: f64,
    bsup: f64,
    my_params: Option<Array1<'static, f64>>,
    tool: PhantomData<fn() -> C>,
}

impl<C: InterCurveTool> InterPolygon<C> {
    /// Constructor (Curve, NbPnt): `max(NbPnt, 5)` points over the whole curve;
    /// `ThePolygonOfHInter::new(&curve, 10)?`.
    pub fn new(curve: &C::Curve, nb_pnt: i32) -> Result<Self, Failure> {
        let (binf, bsup) = (C::first_parameter(curve)?, C::last_parameter(curve)?);
        Self::with_range(curve, binf, bsup, nb_pnt)
    }
    /// Constructor (Curve, U1, U2, NbPnt): `max(NbPnt, 5)` points from `u1` to `u2`;
    /// `ThePolygonOfHInter::with_range(&curve, 0.0, 1.0, 10)?`.
    pub fn with_range(curve: &C::Curve, u1: f64, u2: f64, nb_pnt: i32) -> Result<Self, Failure> {
        let nb_pnt_in: i32 = nb_pnt.max(5);
        let samples: PolygonSamples = init_uniform::<C>(curve, u1, u2, nb_pnt_in)?;
        Ok(Self::built(samples, nb_pnt_in, (u1, u2), None))
    }
    /// Constructor (Curve, Upars): a point at each parameter; Standard_OutOfRange
    /// for an empty array; `ThePolygonOfHInter::with_parameters(&curve, &upars)?`.
    pub fn with_parameters(curve: &C::Curve, upars: &Array1<'_, f64>) -> Result<Self, Failure> {
        let bound = |i: i32| {
            upars
                .value(i)
                .copied()
                .map_err(|e| e.into_failure("NCollection_Array1::Value"))
        };
        let (binf, bsup) = (bound(upars.lower())?, bound(upars.upper())?);
        let nb_pnt_in: i32 = upars.length();
        let (samples, params) = init_with_params::<C>(curve, upars, nb_pnt_in)?;
        Ok(Self::built(samples, nb_pnt_in, (binf, bsup), Some(params)))
    }
    fn built(
        samples: PolygonSamples,
        nb_pnt_in: i32,
        (binf, bsup): (f64, f64),
        params: Option<Array1<'static, f64>>,
    ) -> Self {
        Self {
            the_bnd: samples.bnd,
            the_deflection: samples.deflection,
            nb_pnt_in,
            the_pnts: samples.points,
            closed_polygon: false,
            binf,
            bsup,
            my_params: params,
            tool: PhantomData,
        }
    }
    /// `Bounding()`: the box of the polygon; `polygon.bounding()`.
    pub fn bounding(&self) -> &BoundingBox3 {
        &self.the_bnd
    }
    /// `DeflectionOverEstimation()`; `polygon.deflection_over_estimation()`.
    pub fn deflection_over_estimation(&self) -> f64 {
        self.the_deflection
    }
    /// `SetDeflectionOverEstimation(x)`: stores `x` and enlarges the box by it;
    /// `polygon.set_deflection_over_estimation(0.1)`.
    pub fn set_deflection_over_estimation(&mut self, x: f64) {
        self.the_deflection = x;
        self.the_bnd.enlarge(x);
    }
    /// `Closed(flag)`; `polygon.set_closed(true)`.
    pub fn set_closed(&mut self, flag: bool) {
        self.closed_polygon = flag;
    }
    /// `Closed()`; `polygon.closed()`.
    pub fn closed(&self) -> bool {
        self.closed_polygon
    }
    /// `NbSegments()`: the point count minus one; `polygon.nb_segments()`.
    pub fn nb_segments(&self) -> i32 {
        self.nb_pnt_in - 1
    }
    /// `BeginOfSeg(Index)`: point `index`; Standard_OutOfRange outside the points;
    /// `polygon.begin_of_seg(1)?`.
    pub fn begin_of_seg(&self, index: i32) -> Result<Point3, Failure> {
        self.pnt(index, "IntCurveSurface_ThePolygonOfHInter::BeginOfSeg")
    }
    /// `EndOfSeg(Index)`: point `index + 1`; `polygon.end_of_seg(1)?`.
    pub fn end_of_seg(&self, index: i32) -> Result<Point3, Failure> {
        self.pnt(index + 1, "IntCurveSurface_ThePolygonOfHInter::EndOfSeg")
    }
    fn pnt(&self, index: i32, member: &str) -> Result<Point3, Failure> {
        let found = usize::try_from(index - 1)
            .ok()
            .and_then(|at| self.the_pnts.get(at));
        found
            .copied()
            .ok_or_else(|| out_of_bounds(member, index, 1, self.the_pnts.len() as i32))
    }
    /// `InfParameter()`: the parameter of the first point; `polygon.inf_parameter()`.
    pub fn inf_parameter(&self) -> f64 {
        self.binf
    }
    /// `SupParameter()`: the parameter of the last point; `polygon.sup_parameter()`.
    pub fn sup_parameter(&self) -> f64 {
        self.bsup
    }
    /// `ApproxParamOnCurve(Index, ParamOnLine)`: see
    /// [`approx_param_on_curve`](super::polygon_utils::approx_param_on_curve);
    /// `polygon.approx_param_on_curve(2, 0.5)?`.
    pub fn approx_param_on_curve(&self, index: i32, param_on_line: f64) -> Result<f64, Failure> {
        approx_param_on_curve(
            index,
            param_on_line,
            self.binf,
            self.bsup,
            self.nb_pnt_in,
            self.my_params.as_ref(),
        )
    }
    /// `Dump()`: writes nothing natively; `polygon.dump()`.
    pub fn dump(&self) {}
}

/// `IntCurveSurface_ThePolygonToolOfHInter`: static access to an [`InterPolygon`],
/// the `ToolPolygon3d` of the Intf interference. The instance on
/// `IntCurveSurface_TheHCurveTool` is [`ThePolygonToolOfHInter`](super::ThePolygonToolOfHInter).
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::ThePolygonToolOfHInter;
/// use rustycad_tkgeomalgo::intf::Polygon3dTool;
/// fn segments(p: &<ThePolygonToolOfHInter as Polygon3dTool>::Polygon) -> i32 {
///     ThePolygonToolOfHInter::nb_segments(p)
/// }
/// ```
pub struct InterPolygonTool<C: InterCurveTool>(PhantomData<fn() -> C>);

/// The count of `Dump` calls, the native function-level `static int num`.
static DUMP_COUNT: AtomicI32 = AtomicI32::new(0);

impl<C: InterCurveTool> InterPolygonTool<C> {
    /// `Dump(L)`: a numbered header with the deflection, then each segment start and
    /// the last point again with the last segment index (std::cout natively); the
    /// number counts the calls of the process, as the native static does;
    /// `ThePolygonToolOfHInter::dump(&polygon, &mut text)?`.
    pub fn dump(polygon: &InterPolygon<C>, out: &mut dyn fmt::Write) -> Result<(), Failure> {
        let num: i32 = DUMP_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        let nbs: i32 = polygon.nb_segments();
        let text = |e: fmt::Error| Failure::new(&format!("Dump: {e}"));
        write!(
            out,
            "\npol2d {num} {nbs}\nDeflection {}\n",
            general(polygon.deflection_over_estimation())
        )
        .map_err(text)?;
        for i in 1..=nbs {
            dump_point(out, num, i, polygon.begin_of_seg(i)?).map_err(text)?;
        }
        dump_point(out, num, nbs, polygon.end_of_seg(nbs)?).map_err(text)
    }
}

/// One `pnt num i x y z` line.
fn dump_point(out: &mut dyn fmt::Write, num: i32, i: i32, p: Point3) -> fmt::Result {
    writeln!(
        out,
        "pnt {num} {i} {} {} {}",
        general(p.x()),
        general(p.y()),
        general(p.z())
    )
}

impl<C: InterCurveTool> Polygon3dTool for InterPolygonTool<C> {
    type Polygon = InterPolygon<C>;
    fn bounding(polygon: &InterPolygon<C>) -> &BoundingBox3 {
        polygon.bounding()
    }
    fn deflection_over_estimation(polygon: &InterPolygon<C>) -> f64 {
        polygon.deflection_over_estimation()
    }
    fn closed(polygon: &InterPolygon<C>) -> bool {
        polygon.closed()
    }
    fn nb_segments(polygon: &InterPolygon<C>) -> i32 {
        polygon.nb_segments()
    }
    fn begin_of_seg(polygon: &InterPolygon<C>, index: i32) -> Result<Point3, Failure> {
        polygon.begin_of_seg(index)
    }
    fn end_of_seg(polygon: &InterPolygon<C>, index: i32) -> Result<Point3, Failure> {
        polygon.end_of_seg(index)
    }
}
