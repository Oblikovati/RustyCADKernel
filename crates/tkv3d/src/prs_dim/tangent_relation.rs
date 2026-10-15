//! `PrsDim_TangentRelation`: two tangent edges (lines, circles, ellipses), drawn as a short
//! tangent segment at the tangency point.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_owner};
use super::*;
use crate::dsg_prs::DsgPrsTangentPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::Select3DSensitiveSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomCircle, GeomConic, GeomCurve, GeomEllipse, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_api::ExtremaCurveCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};
use std::f64::consts::PI;

/// The tangency point, direction and length of `PrsDim_TangentRelation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimTangentFields {
    /// `myAttach`.
    pub attach: Point3,
    /// `myDir`.
    pub dir: Direction3,
    /// `myLength`.
    pub length: f64,
    /// `myExternRef`.
    pub extern_ref: i32,
}

prs_dim_relation_class!(
    PrsDimTangentRelation,
    "PrsDim_TangentRelation",
    PrsDimTangentFields,
    "`PrsDim_TangentRelation`; `PrsDimTangentRelation::new(&s1, &s2, &plane, 0)`."
);

/// The derivative of a circle or ellipse curve at the parameter of a point (null otherwise).
fn conic_d1(curve: &Handle<dyn GeomCurve>, point: Point3) -> Result<Vector3, Failure> {
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        let circ = circle.circ()?;
        return Ok(elclib::d1(elclib::parameter(&circ, point)?, &circ).1);
    }
    if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
        let elips = ellipse.elips()?;
        return Ok(elclib::d1(elclib::parameter(&elips, point)?, &elips).1);
    }
    Ok(Vector3::default())
}

/// `ComputeTangencyPoint`: the nearest extremum between two curves, stopping at a zero
/// distance with parallel tangents (a line goes on to the next extremum).
fn compute_tangency_point(
    first: &Handle<dyn GeomCurve>,
    second: &Handle<dyn GeomCurve>,
) -> Result<Point3, Failure> {
    let (c1, c2) = (
        first.get().ok_or_else(null_object)?,
        second.get().ok_or_else(null_object)?,
    );
    let extrema = ExtremaCurveCurve::with_ranges(
        first,
        second,
        (c1.first_parameter(), c1.last_parameter()),
        (c2.first_parameter(), c2.last_parameter()),
    )?;
    let mut nearest = Point3::default();
    let mut min_distance = 0.0;
    for index in 1..=extrema.nb_extrema() {
        let (p1, p2) = extrema.points(index)?;
        let distance = p1.distance(&p2);
        if index == 1 || distance < min_distance || distance < precision::confusion() {
            min_distance = distance;
            nearest = p1;
        }
        if distance < precision::confusion() {
            if c1.is_instance(GeomLine::static_type()) {
                continue;
            }
            let v1 = conic_d1(first, p1)?;
            let v2 = conic_d1(second, p2)?;
            if v1.is_parallel(v2, PI / 360.0)? {
                break;
            }
        }
    }
    Ok(nearest)
}

/// The kind code of a curve: 1 line, 2 circle, 3 ellipse, else 0.
fn kind_code(curve: &Handle<dyn GeomCurve>) -> i32 {
    let Some(value) = curve.get() else {
        return 0;
    };
    if value.is_instance(GeomLine::static_type()) {
        1
    } else if value.is_instance(GeomCircle::static_type()) {
        2
    } else if value.is_instance(GeomEllipse::static_type()) {
        3
    } else {
        0
    }
}

/// The common vertex of two edges: a vertex of the second among the first two vertices of
/// the first (the native loop also reads past its two-slot table, never a match).
fn common_vertex(first: &Shape, second: &Shape) -> Result<Option<Point3>, Failure> {
    let mut table = Vec::new();
    for vertex in Explorer::new(first, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        table.push(vertex);
    }
    for vertex in Explorer::new(second, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if table
            .iter()
            .take(2)
            .any(|candidate| vertex.is_same(candidate))
        {
            return Ok(Some(BRepTool::pnt(&topods::vertex(&vertex)?)?));
        }
    }
    Ok(None)
}

impl PrsDimTangentRelation {
    /// `PrsDim_TangentRelation(shape1, shape2, plane, externRef)`: a fixed position.
    /// Example: `PrsDimTangentRelation::new(&s1, &s2, &plane, 0)`.
    pub fn new(
        first: &Shape,
        second: &Shape,
        plane: &Handle<GeomPlane>,
        extern_ref: i32,
    ) -> Handle<Self> {
        let relation = Self::alloc(PrsDimTangentFields {
            extern_ref,
            ..PrsDimTangentFields::default()
        });
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            f.plane = plane.clone();
            f.automatic_position = false;
        }
        relation
    }
    /// `ExternRef`; `relation.extern_ref()`.
    pub fn extern_ref(&self) -> i32 {
        self.fields.read().extern_ref
    }
    /// `SetExternRef`; `relation.set_extern_ref(1)`.
    pub fn set_extern_ref(&self, reference: i32) {
        self.fields.write().extern_ref = reference;
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        match self.first_shape().shape_type()? {
            ShapeEnum::Face => {
                self.compute_two_faces_tangent(prs);
                Ok(())
            }
            ShapeEnum::Edge => self.compute_two_edges_tangent(prs),
            _ => Ok(()),
        }
    }

    /// `ComputeTwoFacesTangent`: nothing, as natively.
    pub fn compute_two_faces_tangent(&self, _: &Handle<dyn Graphic3dStructureSource>) {}

    /// `ComputeTwoEdgesTangent`: the tangency point (a common vertex, else computed from the
    /// curves), its direction and a length from the radii.
    /// Example: `relation.compute_two_edges_tangent(&prs)?`.
    pub fn compute_two_edges_tangent(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::edge(&self.first_shape())?;
        let second = topods::edge(&self.second_shape())?;
        let Some((done, edges)) = PrsDim::compute_two_edges_ext(&first, &second, &self.plane())?
        else {
            return Ok(());
        };
        self.set_ext_shape(edges.ext_index);
        if !done {
            return Ok(());
        }
        let (inf1, inf2) = (edges.first_infinite, edges.second_infinite);
        prs.get()
            .ok_or_else(null_object)?
            .set_infinite_state(inf1 || inf2);
        let (copy1, copy2) = (edges.first_curve.clone(), edges.second_curve.clone());
        let (k1, k2) = (kind_code(&copy1), kind_code(&copy2));
        if k1 == 0 || k2 == 0 {
            return Ok(());
        }
        let common = common_vertex(&self.first_shape(), &self.second_shape())?;
        let found = common.is_some();
        let mut point = common.unwrap_or_default();
        let length1 = edges.first_ends.0.distance(&edges.first_ends.1);
        let length2 = edges.second_ends.0.distance(&edges.second_ends.1);
        let (direction, mut length) = match (k1, k2) {
            (1, 2) | (2, 1) => {
                let (line, circle, infinite, edge_length) = if k1 == 1 {
                    (&copy1, &copy2, inf1, length1)
                } else {
                    (&copy2, &copy1, inf2, length2)
                };
                let lin = line.cast::<GeomLine>().get().ok_or_else(null_object)?.lin();
                let circle = circle.cast::<GeomCircle>();
                let circle = circle.get().ok_or_else(null_object)?;
                if !found {
                    let at = elclib::parameter(&lin, circle.location())?;
                    point = elclib::value(at, &lin);
                }
                let mut length = circle.radius() / 5.0;
                if !infinite && edge_length < length {
                    length = edge_length / 3.0;
                }
                (lin.direction(), length)
            }
            (1, 3) | (3, 1) => {
                let (line, ellipse, infinite, edge_length) = if k1 == 1 {
                    (&copy1, &copy2, inf1, length1)
                } else {
                    (&copy2, &copy1, inf2, length2)
                };
                if !found {
                    point = compute_tangency_point(line, ellipse)?;
                }
                let lin = line.cast::<GeomLine>().get().ok_or_else(null_object)?.lin();
                let major = ellipse
                    .cast::<GeomEllipse>()
                    .get()
                    .ok_or_else(null_object)?
                    .major_radius();
                let mut length = major / 5.0;
                if !infinite && edge_length < length {
                    length = edge_length / 3.0;
                }
                (lin.direction(), length)
            }
            (a, b) if a >= 2 && b >= 2 => self.conic_conic(&copy1, &copy2, found, &mut point)?,
            _ => return Ok(()),
        };
        let mut f = *self.fields.read();
        f.attach = point;
        f.dir = direction;
        length = length.min(self.arrow_size());
        f.length = length;
        *self.fields.write() = f;
        self.relation_base().fields.write().position = point;
        DsgPrsTangentPresentation::add(prs, &self.attributes(), &f.attach, &f.dir, f.length)?;
        let ext = self.ext_shape();
        if ext == 0 || edges.ext_curve.is_null() {
            return Ok(());
        }
        let style = PrsDimProjectionStyle::default();
        let unset = (Point3::default(), Point3::default());
        if ext == 1 {
            let ends = if inf1 { unset } else { edges.first_ends };
            self.compute_proj_edge_presentation(prs, &first, &copy1, ends.0, ends.1, &style)
        } else {
            let ends = if inf2 { unset } else { edges.second_ends };
            self.compute_proj_edge_presentation(prs, &second, &copy2, ends.0, ends.1, &style)
        }
    }

    /// The tangency of two circles or ellipses: the point (when not a common vertex), the
    /// tangent direction there and the length from the larger radius.
    fn conic_conic(
        &self,
        copy1: &Handle<dyn GeomCurve>,
        copy2: &Handle<dyn GeomCurve>,
        found: bool,
        point: &mut Point3,
    ) -> Result<(Direction3, f64), Failure> {
        let radius = |curve: &Handle<dyn GeomCurve>| -> Result<f64, Failure> {
            if let Some(circle) = curve.cast::<GeomCircle>().get() {
                return Ok(circle.radius());
            }
            Ok(curve
                .cast::<GeomEllipse>()
                .get()
                .ok_or_else(null_object)?
                .major_radius())
        };
        let (r1, r2) = (radius(copy1)?, radius(copy2)?);
        let length = r1.max(r2) / 5.0;
        let both_circles = kind_code(copy1) == 2 && kind_code(copy2) == 2;
        let both_ellipses = kind_code(copy1) == 3 && kind_code(copy2) == 3;
        let vector = if found {
            // A common vertex: the tangent of the circle (the first ellipse for two ellipses).
            let circle = if kind_code(copy1) == 2 || both_ellipses {
                copy1
            } else {
                copy2
            };
            let (p, v) = conic_d1_at(circle, *point)?;
            *point = p;
            v
        } else if both_circles {
            let c1 = copy1.cast::<GeomCircle>();
            let c2 = copy2.cast::<GeomCircle>();
            let (c1, c2) = (
                c1.get().ok_or_else(null_object)?,
                c2.get().ok_or_else(null_object)?,
            );
            let (big, small) = if r1 >= r2 { (c1, c2) } else { (c2, c1) };
            let circ = big.circ()?;
            // Concentric circles keep the parameter 0, as natively.
            let at = if c1
                .location()
                .is_equal(&c2.location(), precision::confusion())
            {
                0.0
            } else {
                elclib::parameter(&circ, small.location())?
            };
            let (p, v) = elclib::d1(at, &circ);
            *point = p;
            v
        } else {
            let first_larger = if both_ellipses { r1 > r2 } else { r1 >= r2 };
            let (big, small) = if first_larger {
                (copy1, copy2)
            } else {
                (copy2, copy1)
            };
            *point = compute_tangency_point(big, small)?;
            let (p, v) = conic_d1_at(big, *point)?;
            *point = p;
            v
        };
        Ok((Direction3::from_vector(vector)?, length))
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let f = *self.fields.read();
        let position = self.position();
        let vector = Vector3::from(f.dir);
        let p1 = position.translated(vector * f.length);
        let p2 = position.translated(vector * -f.length);
        let owner = relation_owner(self, 7);
        selection
            .get()
            .ok_or_else(null_object)?
            .add(&Handle::new(Select3DSensitiveSegment::new(&owner, p1, p2)).cast())
    }
}

/// `ElCLib::D1` of a circle or ellipse at the parameter of a point.
fn conic_d1_at(curve: &Handle<dyn GeomCurve>, point: Point3) -> Result<(Point3, Vector3), Failure> {
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        let circ = circle.circ()?;
        return Ok(elclib::d1(elclib::parameter(&circ, point)?, &circ));
    }
    let ellipse = curve.cast::<GeomEllipse>();
    let elips = ellipse.get().ok_or_else(null_object)?.elips()?;
    Ok(elclib::d1(elclib::parameter(&elips, point)?, &elips))
}
