// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_CSIntersector.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_CSIntersector`: the intersections of a set of lines, circles or curves with
//! the faces of a shape, sorted by parameter along each element.
use super::PntFace;
use super::localize::{Localized, loc_after, loc_after_index, loc_before};
use super::pnt_face::insert_sorted;
use crate::errors::{construction_error, not_done, out_of_range};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::int_curve_surface::{HCurve, TransitionOnCurve};
use rustycad_tkmath::gp::{Circle3, Line3};
use rustycad_tktopalgo::int_curves_face::IntCurvesFaceIntersector;

/// `LocOpe_CSIntersector`: the points of each element on the faces of the shape.
///
/// ```
/// use rustycad_tkfeat::loc_ope::CSIntersector;
/// let inter = CSIntersector::new();
/// assert!(!inter.is_done());
/// ```
#[derive(Clone, Debug, Default)]
pub struct CSIntersector {
    done: bool,
    shape: Shape,
    /// `myPoints`: one sorted point sequence per element (`myNbelem` of them).
    points: Vec<Vec<PntFace>>,
}

/// The orientation of an intersection point by the transition of the curve: FORWARD
/// entering, REVERSED leaving, EXTERNAL tangent (the face orientation is not used).
fn transition_orientation(transition: TransitionOnCurve) -> Orientation {
    match transition {
        TransitionOnCurve::In => Orientation::Forward,
        TransitionOnCurve::Out => Orientation::Reversed,
        TransitionOnCurve::Tangent => Orientation::External,
    }
}

/// `AddPoints`: the points of `inter` on `face`, inserted into `seq` by parameter.
fn add_points(
    inter: &IntCurvesFaceIntersector,
    seq: &mut Vec<PntFace>,
    face: &Face,
) -> Result<(), Failure> {
    for j in 1..=inter.nb_pnt() {
        let theor = transition_orientation(inter.transition(j)?);
        let point = PntFace::new(
            inter.pnt(j)?,
            face.clone(),
            theor,
            inter.w_parameter(j)?,
            inter.u_parameter(j)?,
            inter.v_parameter(j)?,
        );
        insert_sorted(seq, point);
    }
    Ok(())
}

/// How one face intersector meets one element.
enum Element<'a> {
    Line(&'a Line3),
    Curve(HCurve, f64, f64),
}

impl CSIntersector {
    /// `LocOpe_CSIntersector()`: no shape, not done; `CSIntersector::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_CSIntersector(S)`: the shape, not done yet;
    /// `CSIntersector::with_shape(&shape)`.
    pub fn with_shape(s: &Shape) -> Self {
        Self {
            done: false,
            shape: s.clone(),
            points: Vec::new(),
        }
    }

    /// `Init(S)`: a new shape, the points forgotten; `inter.init(&shape)`.
    pub fn init(&mut self, s: &Shape) {
        self.done = false;
        self.shape = s.clone();
        self.points.clear();
    }

    /// Starts a Perform on `count` elements: Standard_ConstructionError without shape or
    /// element.
    fn start(&mut self, count: usize) -> Result<(), Failure> {
        if self.shape.is_null() || count == 0 {
            return Err(construction_error(
                "LocOpe_CSIntersector::Perform",
                "null shape or no element; expected a shape and at least one element",
            ));
        }
        self.done = false;
        self.points = vec![Vec::new(); count];
        Ok(())
    }

    /// Intersects every face with the elements made by `element`, each face with one
    /// intersector of tolerance `tol`.
    fn intersect_faces<'a, F>(
        &mut self,
        tol: f64,
        count: usize,
        mut element: F,
    ) -> Result<(), Failure>
    where
        F: FnMut(usize) -> Result<Option<Element<'a>>, Failure>,
    {
        for f in Explorer::new(&self.shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let theface = topods::face(&f)?;
            let mut the_int = IntCurvesFaceIntersector::new(&theface, tol, true, true)?;
            for i in 0..count {
                match element(i)? {
                    None => continue,
                    Some(Element::Line(lin)) => the_int.perform_line(lin, -f64::MAX, f64::MAX)?,
                    Some(Element::Curve(hc, binf, bsup)) => {
                        the_int.perform_curve(&hc, binf, bsup)?
                    }
                }
                if the_int.is_done() {
                    add_points(&the_int, &mut self.points[i], &theface)?;
                }
            }
        }
        self.done = true;
        Ok(())
    }

    /// `Perform(Slin)`: the points of the lines on the faces (PConfusion tolerance,
    /// unbounded lines); `inter.perform_lines(&lines)?`.
    pub fn perform_lines(&mut self, slin: &Sequence<Line3>) -> Result<(), Failure> {
        self.start(slin.len())?;
        let lines: Vec<&Line3> = slin.iter().collect();
        self.intersect_faces(precision::p_confusion(), lines.len(), |i| {
            Ok(Some(Element::Line(lines[i])))
        })
    }

    /// `Perform(Scir)`: the points of the full circles on the faces;
    /// `inter.perform_circles(&circles)?`.
    pub fn perform_circles(&mut self, scir: &Sequence<Circle3>) -> Result<(), Failure> {
        self.start(scir.len())?;
        let circles: Vec<Circle3> = scir.iter().copied().collect();
        self.intersect_faces(0., circles.len(), |i| {
            let circle: Handle<dyn GeomCurve> =
                Handle::new(GeomCircle::from_circ(circles[i])).cast();
            let hc: HCurve = Handle::new(GeomAdaptorCurve::with_curve(&circle)?).cast();
            Ok(Some(Element::Curve(hc, 0., 2. * std::f64::consts::PI)))
        })
    }

    /// `Perform(Scur)`: the points of the curves (null curves skipped) on their range;
    /// `inter.perform_curves(&curves)?`.
    pub fn perform_curves(
        &mut self,
        scur: &Sequence<Handle<dyn GeomCurve>>,
    ) -> Result<(), Failure> {
        self.start(scur.len())?;
        let curves: Vec<Handle<dyn GeomCurve>> = scur.iter().cloned().collect();
        self.intersect_faces(0., curves.len(), |i| {
            if curves[i].is_null() {
                return Ok(None);
            }
            let adaptor = GeomAdaptorCurve::with_curve(&curves[i])?;
            let (binf, bsup) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
            let hc: HCurve = Handle::new(adaptor).cast();
            Ok(Some(Element::Curve(hc, binf, bsup)))
        })
    }

    /// `IsDone()`; `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// The points of the element `i`: StdFail_NotDone before Perform,
    /// Standard_OutOfRange outside `[1, NbElem]`.
    fn element_points(&self, member: &str, i: i32) -> Result<&[PntFace], Failure> {
        if !self.done {
            return Err(not_done(member));
        }
        let nbelem = self.points.len() as i32;
        if i <= 0 || i > nbelem {
            return Err(out_of_range(member, i, nbelem));
        }
        Ok(&self.points[(i - 1) as usize])
    }

    /// `NbPoints(I)`: the number of points on the element `i`; `inter.nb_points(1)?`.
    pub fn nb_points(&self, i: i32) -> Result<i32, Failure> {
        Ok(self
            .element_points("LocOpe_CSIntersector::NbPoints", i)?
            .len() as i32)
    }

    /// `Point(I, Index)`: the point `index` on the element `i`; `inter.point(1, 1)?`.
    pub fn point(&self, i: i32, index: i32) -> Result<&PntFace, Failure> {
        let points = self.element_points("LocOpe_CSIntersector::Point", i)?;
        if index <= 0 || index as usize > points.len() {
            return Err(out_of_range(
                "LocOpe_CSIntersector::Point",
                index,
                points.len() as i32,
            ));
        }
        Ok(&points[(index - 1) as usize])
    }

    /// `LocalizeAfter(I, From, Tol, Or, IndFrom, IndTo)`: the first franc point of the
    /// element `i` at a parameter after `from`;
    /// `inter.localize_after(1, 0.0, 1e-7, &mut or, &mut from, &mut to)?`.
    pub fn localize_after(
        &self,
        i: i32,
        from: f64,
        tol: f64,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.element_points("LocOpe_CSIntersector::LocalizeAfter", i)?;
        Ok(loc_after(
            points,
            from,
            tol,
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `LocalizeBefore(I, From, Tol, Or, IndFrom, IndTo)`: the last franc point of the
    /// element `i` at a parameter before `from`;
    /// `inter.localize_before(1, 0.0, 1e-7, &mut or, &mut from, &mut to)?`.
    pub fn localize_before(
        &self,
        i: i32,
        from: f64,
        tol: f64,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.element_points("LocOpe_CSIntersector::LocalizeBefore", i)?;
        Ok(loc_before(
            points,
            from,
            tol,
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `LocalizeAfter(I, FromInd, Tol, Or, IndFrom, IndTo)`: the first franc point of the
    /// element `i` after the index `from_ind`;
    /// `inter.localize_after_index(1, 0, 1e-7, &mut or, &mut from, &mut to)?`.
    pub fn localize_after_index(
        &self,
        i: i32,
        from_ind: i32,
        tol: f64,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.element_points("LocOpe_CSIntersector::LocalizeAfter", i)?;
        Ok(loc_after_index(
            points,
            from_ind,
            tol,
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `LocalizeBefore(I, FromInd, Tol, Or, IndFrom, IndTo)`. Native quirk kept: no
    /// index overload of the static `LocBefore` exists, so the index converts to a
    /// parameter and the search is the parameter one from `FromInd`;
    /// `inter.localize_before_index(1, 3, 1e-7, &mut or, &mut from, &mut to)?`.
    pub fn localize_before_index(
        &self,
        i: i32,
        from_ind: i32,
        tol: f64,
        or: &mut Orientation,
        ind_from: &mut i32,
        ind_to: &mut i32,
    ) -> Result<bool, Failure> {
        let points = self.element_points("LocOpe_CSIntersector::LocalizeBefore", i)?;
        Ok(loc_before(
            points,
            f64::from(from_ind),
            tol,
            Localized {
                or,
                ind_from,
                ind_to,
            },
        ))
    }

    /// `Destroy()`: frees the points (the done flag stays); `inter.destroy()`.
    pub fn destroy(&mut self) {
        self.points.clear();
    }
}
