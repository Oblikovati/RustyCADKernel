//! `PrsDim_FixRelation`: a fixed vertex or edge (line or circle), drawn as an anchor symbol
//! on a segment from the shape.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_owner};
use super::*;
use crate::dsg_prs::DsgPrsFixPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::Select3DSensitiveSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomConic, GeomCurve, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Circle3, Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::f64::consts::PI;

/// The wire and attach point of `PrsDim_FixRelation`.
#[derive(Clone, Default)]
pub struct PrsDimFixFields {
    /// `myWire`.
    pub wire: Wire,
    /// `myPntAttach`.
    pub pnt_attach: Point3,
}

prs_dim_relation_class!(
    PrsDimFixRelation,
    "PrsDim_FixRelation",
    PrsDimFixFields,
    "`PrsDim_FixRelation`; `PrsDimFixRelation::new(&shape, &plane, None)`.",
    movable = true
);

/// The file-local `InDomain`: a parameter in `[first, last]`, or in a range wrapping zero
/// when `first` is negative.
fn in_domain(first: f64, last: f64, parameter: f64) -> bool {
    if first >= 0.0 {
        return parameter >= first && parameter <= last;
    }
    parameter >= first + 2.0 * PI || parameter <= last
}

/// A circle parameter outside the arc replaced by the opposite one.
fn opposite_if_outside(first: f64, last: f64, parameter: f64) -> f64 {
    if in_domain(first, last, parameter) {
        return parameter;
    }
    let mut other = parameter + PI;
    if other > 2.0 * PI {
        other -= 2.0 * PI;
    }
    other
}

impl PrsDimFixRelation {
    fn create(
        shape: &Shape,
        plane: &Handle<GeomPlane>,
        wire: Option<&Wire>,
        placed: Option<(Point3, f64)>,
    ) -> Handle<Self> {
        let relation = Self::alloc(PrsDimFixFields {
            wire: wire.cloned().unwrap_or_default(),
            ..PrsDimFixFields::default()
        });
        {
            let mut f = relation.base.fields.write();
            f.f_shape = shape.clone();
            f.plane = plane.clone();
        }
        match placed {
            None => {
                let mut f = relation.base.fields.write();
                f.automatic_position = true;
                f.arrow_size = 5.0;
            }
            Some((position, arrow_size)) => {
                relation.base.fields.write().position = position;
                relation.set_arrow_size(arrow_size);
                relation.base.fields.write().automatic_position = false;
            }
        }
        relation
    }
    /// `PrsDim_FixRelation(shape, plane[, wire])`: automatic, symbol size 5.
    /// Example: `PrsDimFixRelation::new(&shape, &plane, None)`.
    pub fn new(shape: &Shape, plane: &Handle<GeomPlane>, wire: Option<&Wire>) -> Handle<Self> {
        Self::create(shape, plane, wire, None)
    }
    /// `PrsDim_FixRelation(shape, plane[, wire], position, arrowSize)`.
    /// Example: `PrsDimFixRelation::with_position(&shape, &plane, None, &p, 3.0)`.
    pub fn with_position(
        shape: &Shape,
        plane: &Handle<GeomPlane>,
        wire: Option<&Wire>,
        position: &Point3,
        arrow_size: f64,
    ) -> Handle<Self> {
        Self::create(shape, plane, wire, Some((*position, arrow_size)))
    }
    /// `Wire`; `relation.wire()`.
    pub fn wire(&self) -> Wire {
        self.fields.read().wire.clone()
    }
    /// `SetWire`; `relation.set_wire(&wire)`.
    pub fn set_wire(&self, wire: &Wire) {
        self.fields.write().wire = wire.clone();
    }
    fn normal(&self) -> Result<Direction3, Failure> {
        Ok(self
            .plane()
            .get()
            .ok_or_else(null_object)?
            .pln()
            .axis()
            .direction())
    }
    fn set_attach(&self, point: Point3) {
        self.fields.write().pnt_attach = point;
    }
    fn attach(&self) -> Point3 {
        self.fields.read().pnt_attach
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let shape = self.first_shape();
        let mut position = Point3::default();
        match shape.shape_type()? {
            ShapeEnum::Vertex => position = self.compute_vertex(&topods::vertex(&shape)?)?,
            ShapeEnum::Edge => {
                if let Some(computed) = self.compute_edge(&topods::edge(&shape)?)? {
                    position = computed;
                }
            }
            _ => {}
        }
        let normal = self.normal()?;
        if !self.relation_base().fields.read().arrow_size_is_defined {
            self.relation_base().fields.write().arrow_size = 5.0;
        }
        DsgPrsFixPresentation::add(
            prs,
            &self.attributes(),
            &self.attach(),
            &position,
            &normal,
            self.arrow_size(),
        )
    }

    /// `ComputeVertex`: the vertex point, the symbol beside it along the plane X axis.
    /// Example: `relation.compute_vertex(&vertex)?`.
    pub fn compute_vertex(&self, _: &Vertex) -> Result<Point3, Failure> {
        let attach = BRepTool::pnt(&topods::vertex(&self.first_shape())?)?;
        self.set_attach(attach);
        let mut position = self.position();
        if self.automatic_position() {
            let pln = self.plane().get().ok_or_else(null_object)?.pln();
            let shift = Vector3::from(pln.x_axis().direction()) * self.arrow_size();
            position = attach.translated(shift);
            let mut f = self.relation_base().fields.write();
            f.position = position;
            f.automatic_position = true;
        }
        Ok(position)
    }

    /// `ComputePosition(curve1, curve2, ...)`: the symbol position at the attach point for
    /// two curves meeting there.
    /// Example: `relation.compute_position_two(&c1, &c2, a1, b1, a2, b2)?`.
    pub fn compute_position_two(
        &self,
        first: &Handle<dyn GeomCurve>,
        second: &Handle<dyn GeomCurve>,
        (first1, last1): (Point3, Point3),
        (first2, last2): (Point3, Point3),
    ) -> Result<Point3, Failure> {
        let attach = self.attach();
        let size = self.arrow_size();
        let circle = |curve: &Handle<dyn GeomCurve>| {
            curve
                .get()
                .and_then(|c| c.is_instance(GeomCircle::static_type()).then_some(()))
        };
        if circle(first).is_some() || circle(second).is_some() {
            let mut circ = first.cast::<GeomCircle>();
            if circ.is_null() {
                circ = second.cast::<GeomCircle>();
            }
            let center = circ.get().ok_or_else(null_object)?.location();
            let direction = Direction3::from_xyz(center.xyz() + attach.xyz())?;
            return Ok(attach.translated(Vector3::from(direction) * size));
        }
        let vec1 = Vector3::between(first1, last1);
        let vec2 = Vector3::between(first2, last2);
        if !vec1.is_parallel(vec2, precision::angular())? {
            let conf = precision::confusion();
            let xyz = if last1.is_equal(&first2, conf) || first1.is_equal(&last2, conf) {
                *vec1.xyz() - *vec2.xyz()
            } else {
                *vec1.xyz() + *vec2.xyz()
            };
            let direction = Direction3::from_xyz(xyz)?;
            return Ok(attach.translated(Vector3::from(direction) * size));
        }
        let cross = vec1.crossed(vec2);
        let direction = Direction3::from_vector(vec1.crossed(cross))?;
        Ok(attach.translated(Vector3::from(direction) * size))
    }

    /// `ComputePosition(curve, first, last)`: the symbol position at the attach point of one
    /// curve. Example: `relation.compute_position(&curve, a, b)?`.
    pub fn compute_position(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: Point3,
        last: Point3,
    ) -> Result<Point3, Failure> {
        let attach = self.attach();
        let size = self.arrow_size();
        if let Some(circle) = curve.cast::<GeomCircle>().get() {
            let direction = Direction3::from_xyz(circle.location().xyz() + attach.xyz())?;
            return Ok(attach.translated(Vector3::from(direction) * size));
        }
        let normal = self.normal()?;
        let vector = Vector3::between(first, last)
            .crossed(Vector3::from(normal))
            .normalized()?;
        let position = attach.translated(vector * size);
        position.rotated(Axis1::new(attach, normal), PI / 10.0)
    }

    /// `ComputeEdge`: the attach point and symbol position of a line or circle edge (`None`
    /// for other curves). Example: `relation.compute_edge(&edge)?`.
    pub fn compute_edge(&self, edge: &Edge) -> Result<Option<Point3>, Failure> {
        let Some((curve, (begin, end))) = PrsDim::compute_edge_geometry(edge)? else {
            return Ok(None);
        };
        if let Some(line) = curve.cast::<GeomLine>().get() {
            let lin = line.lin();
            let first = elclib::parameter(&lin, begin)?;
            let last = elclib::parameter(&lin, end)?;
            return Ok(Some(self.compute_line_position(&lin, first, last)?));
        }
        if let Some(circle) = curve.cast::<GeomCircle>().get() {
            let circ = circle.circ()?;
            let adaptor = BRepAdaptorCurve::with_edge(edge)?;
            let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
            return Ok(Some(self.compute_circle_position(&circ, first, last)?));
        }
        Ok(None)
    }

    /// `ComputeLinePosition`: the attach point at the middle of the segment (automatic) or
    /// the position projected on the segment, and the symbol position.
    /// Example: `relation.compute_line_position(&line, 0.0, 10.0)?`.
    pub fn compute_line_position(
        &self,
        line: &Line3,
        first: f64,
        last: f64,
    ) -> Result<Point3, Failure> {
        let normal = self.normal()?;
        let position = if self.automatic_position() {
            let attach = elclib::value((first + last) / 2.0, line);
            self.set_attach(attach);
            let side = normal.crossed(line.position().direction())?;
            self.relation_base().fields.write().automatic_position = true;
            attach.translated(Vector3::from(side) * self.arrow_size())
        } else {
            let position = self.position();
            let parameter = elclib::parameter(line, position)?;
            if parameter >= first && parameter <= last {
                self.set_attach(elclib::value(parameter, line));
                position
            } else {
                let on_line = if parameter > last { last } else { first };
                let attach = elclib::value(on_line, line);
                self.set_attach(attach);
                let side = normal.crossed(line.position().direction())?;
                let support = Line3::new(attach, side);
                elclib::value(elclib::parameter(&support, position)?, &support)
            }
        };
        self.relation_base().fields.write().position = position;
        Ok(position)
    }

    /// `ComputeCirclePosition`: the attach point at the middle of the arc (automatic) or at
    /// the position parameter (moved to the opposite one outside the arc).
    /// Example: `relation.compute_circle_position(&circle, 0.0, 3.0)?`.
    pub fn compute_circle_position(
        &self,
        circle: &Circle3,
        first: f64,
        last: f64,
    ) -> Result<Point3, Failure> {
        let (mut first, mut last) = (first, last);
        if last > 2.0 * PI {
            let turns = (last / (2.0 * PI)).floor();
            last -= turns * 2.0 * PI;
            first -= turns * 2.0 * PI;
        }
        if self.automatic_position() {
            let parameter = opposite_if_outside(first, last, (first + last) / 2.0);
            let attach = elclib::value(parameter, circle);
            self.set_attach(attach);
            let direction = Vector3::between(circle.location(), attach).normalized()?;
            let position = attach.translated(direction * self.arrow_size());
            let mut f = self.relation_base().fields.write();
            f.position = position;
            f.automatic_position = true;
            return Ok(position);
        }
        let position = self.position();
        let parameter = opposite_if_outside(first, last, elclib::parameter(circle, position)?);
        self.set_attach(elclib::value(parameter, circle));
        Ok(position)
    }

    /// `ConnectedEdges`: the two edges of a wire meeting at a vertex (`None` when the vertex is
    /// not in the wire or not shared by exactly two edges).
    /// Example: `PrsDimFixRelation::connected_edges(&wire, &vertex)?`.
    pub fn connected_edges(wire: &Wire, vertex: &Vertex) -> Result<Option<(Edge, Edge)>, Failure> {
        let mut map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &wire.clone().into(),
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut map,
        )?;
        let mut key = None;
        for index in 1..=map.extent() {
            let candidate = map.find_key(index)?;
            if candidate.is_same(&vertex.clone().into()) {
                key = Some(candidate.clone());
                break;
            }
        }
        let Some(key) = key else {
            return Ok(None);
        };
        let ancestors: Vec<Shape> = map.find_from_key(&key)?.iter().cloned().collect();
        let mut edges = Vec::new();
        for ancestor in ancestors.iter().take(2) {
            let edge = topods::edge(ancestor)?;
            BRepAdaptorCurve::with_edge(&edge)?;
            edges.push(edge);
        }
        if edges.len() < 2 || ancestors.len() > 2 {
            return Ok(None);
        }
        let second = edges.pop().ok_or_else(null_object)?;
        let first = edges.pop().ok_or_else(null_object)?;
        Ok(Some((first, second)))
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let owner = relation_owner(self, 7);
        let target = selection.get().ok_or_else(null_object)?;
        let segment = |a: Point3, b: Point3| {
            target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, a, b)).cast())
        };
        let (attach, position) = (self.attach(), self.position());
        segment(attach, position)?;
        let normal = self.normal()?;
        let mut dirac = Vector3::between(attach, position).normalized()?;
        let mut norac = dirac
            .crossed(Vector3::from(normal))
            .rotated(Axis1::new(position, normal), PI / 8.0)?;
        let size = self.arrow_size();
        norac *= size / 2.0;
        segment(
            position.translated(norac),
            position.translated(norac.reversed()),
        )?;
        norac *= 0.8;
        let p1 = position.translated(norac);
        let p2 = position.translated(norac.reversed());
        dirac *= size / 2.0;
        for start in [p1, p2] {
            segment(start, start.translated(dirac).translated(norac))?;
        }
        // The native code builds a third hatch from the middle without adding it.
        Ok(())
    }
}
