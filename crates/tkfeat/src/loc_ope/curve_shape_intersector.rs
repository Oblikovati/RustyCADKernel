// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_CurveShapeIntersector.hxx/.lxx/
// .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_CurveShapeIntersector`: the intersections of an axis or a circle with the
//! faces of a shape, sorted by parameter.
use super::PntFace;
use super::localize::{Localized, loc_after, loc_after_index, loc_before, loc_before_index};
use super::pnt_face::insert_sorted;
use crate::errors::{not_done, out_of_range};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_curve_surface::TransitionOnCurve;
use rustycad_tkmath::gp::{Axis1, Circle3, Line3};
use rustycad_tktopalgo::brep_int_curve_surface::BRepIntCurveSurfaceInter;

/// `LocOpe_CurveShapeIntersector`: the sorted points of one axis or circle on a shape.
///
/// ```
/// use rustycad_tkfeat::loc_ope::CurveShapeIntersector;
/// let inter = CurveShapeIntersector::new();
/// assert!(!inter.is_done());
/// ```
#[derive(Clone, Debug, Default)]
pub struct CurveShapeIntersector {
    done: bool,
    points: Vec<PntFace>,
}

/// The orientation of a point by the transition of the curve and the orientation of
/// the face: kept on a FORWARD face, swapped on a REVERSED one, EXTERNAL otherwise.
fn point_orientation(transition: TransitionOnCurve, orface: Orientation) -> Orientation {
    let (entering, leaving) = match orface {
        Orientation::Forward => (Orientation::Forward, Orientation::Reversed),
        Orientation::Reversed => (Orientation::Reversed, Orientation::Forward),
        _ => (Orientation::External, Orientation::External),
    };
    match transition {
        TransitionOnCurve::In => entering,
        TransitionOnCurve::Out => leaving,
        TransitionOnCurve::Tangent => Orientation::External,
    }
}

/// The static `Perform`: every point of `inter`, inserted by parameter.
fn collect_points(
    inter: &mut BRepIntCurveSurfaceInter,
    points: &mut Vec<PntFace>,
) -> Result<(), Failure> {
    while inter.more() {
        let face = inter.face()?;
        let theor = point_orientation(inter.transition()?, face.orientation());
        let point = PntFace::new(
            inter.pnt()?,
            face,
            theor,
            inter.w()?,
            inter.u()?,
            inter.v()?,
        );
        insert_sorted(points, point);
        inter.next()?;
    }
    Ok(())
}

impl CurveShapeIntersector {
    /// `LocOpe_CurveShapeIntersector()`: not done; `CurveShapeIntersector::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_CurveShapeIntersector(Axis, S)`: performed on the axis;
    /// `CurveShapeIntersector::with_axis(&axis, &shape)?`.
    pub fn with_axis(axis: &Axis1, s: &Shape) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.init_axis(axis, s)?;
        Ok(inter)
    }

    /// `LocOpe_CurveShapeIntersector(C, S)`: performed on the circle;
    /// `CurveShapeIntersector::with_circle(&circle, &shape)?`.
    pub fn with_circle(c: &Circle3, s: &Shape) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.init_circle(c, s)?;
        Ok(inter)
    }

    /// `Init(Axis, S)`: the points of the line of `axis` on the faces of `s`
    /// (Confusion tolerance); nothing done on a null shape; `inter.init_axis(&axis, &s)?`.
    pub fn init_axis(&mut self, axis: &Axis1, s: &Shape) -> Result<(), Failure> {
        self.done = false;
        self.points.clear();
        if s.is_null() {
            return Ok(());
        }
        let mut the_int = BRepIntCurveSurfaceInter::new();
        the_int.init_line(s, Line3::from_axis(*axis), precision::confusion())?;
        collect_points(&mut the_int, &mut self.points)?;
        self.done = true;
        Ok(())
    }

    /// `Init(C, S)`: the points of the full circle `c` on the faces of `s`;
    /// `inter.init_circle(&circle, &s)?`.
    pub fn init_circle(&mut self, c: &Circle3, s: &Shape) -> Result<(), Failure> {
        self.done = false;
        self.points.clear();
        if s.is_null() {
            return Ok(());
        }
        let gc: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(*c)).cast();
        let ac = GeomAdaptorCurve::with_range(&gc, 0., 2. * std::f64::consts::PI)?;
        let mut the_int = BRepIntCurveSurfaceInter::new();
        the_int.init_curve(s, &ac, precision::confusion())?;
        collect_points(&mut the_int, &mut self.points)?;
        self.done = true;
        Ok(())
    }

    /// `IsDone()`; `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    fn done_points(&self, member: &str) -> Result<&[PntFace], Failure> {
        if !self.done {
            return Err(not_done(member));
        }
        Ok(&self.points)
    }

    /// `NbPoints()`; StdFail_NotDone before Init; `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        Ok(self
            .done_points("LocOpe_CurveShapeIntersector::NbPoints")?
            .len() as i32)
    }

    /// `Point(Index)`: the point of 1-based `index`; `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<&PntFace, Failure> {
        let member = "LocOpe_CurveShapeIntersector::Point";
        let points = self.done_points(member)?;
        if index <= 0 || index as usize > points.len() {
            return Err(out_of_range(member, index, points.len() as i32));
        }
        Ok(&points[(index - 1) as usize])
    }

    /// `LocalizeAfter(From, Or, IndFrom, IndTo)`: the first franc point after the
    /// parameter `from` (Confusion as equality tolerance);
    /// `inter.localize_after(0.0, &mut or, &mut from, &mut to)?`.
    pub fn localize_after(
        &self,
        from: f64,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.done_points("LocOpe_CurveShapeIntersector::LocalizeAfter")?;
        Ok(loc_after(
            points,
            from,
            precision::confusion(),
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `LocalizeBefore(From, Or, IndFrom, IndTo)`: the last franc point before the
    /// parameter `from`; `inter.localize_before(0.0, &mut or, &mut from, &mut to)?`.
    pub fn localize_before(
        &self,
        from: f64,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.done_points("LocOpe_CurveShapeIntersector::LocalizeBefore")?;
        Ok(loc_before(
            points,
            from,
            precision::confusion(),
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `LocalizeAfter(FromInd, Or, IndFrom, IndTo)`: the first franc point after the
    /// index `from_ind`; `inter.localize_after_index(0, &mut or, &mut from, &mut to)?`.
    pub fn localize_after_index(
        &self,
        from_ind: i32,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.done_points("LocOpe_CurveShapeIntersector::LocalizeAfter")?;
        Ok(loc_after_index(
            points,
            from_ind,
            precision::confusion(),
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `LocalizeBefore(FromInd, Or, IndFrom, IndTo)`: the last franc point before the
    /// index `from_ind`; `inter.localize_before_index(3, &mut or, &mut from, &mut to)?`.
    pub fn localize_before_index(
        &self,
        from_ind: i32,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.done_points("LocOpe_CurveShapeIntersector::LocalizeBefore")?;
        Ok(loc_before_index(
            points,
            from_ind,
            precision::confusion(),
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }
}
