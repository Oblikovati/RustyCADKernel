//! `PrsDim_IdenticRelation`: two identical shapes (coincident vertices, a vertex on an edge,
//! or edges sharing their geometry), drawn as an identity mark.
use super::dimension_impls::prs_dim_relation_class;
use super::identic_conics::compute_attach;
use super::relation_support::{null_object, relation_owner};
use super::*;
use crate::ais::{AisShape, AisShapeParameters};
use crate::dsg_prs::DsgPrsIdenticPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveCurve, Select3DSensitiveSegment};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomTrimmedCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomConic, GeomCurve, GeomEllipse, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};

/// The users, the circle flag, the attach points and the center of the relation.
#[derive(Clone, Default)]
pub struct PrsDimIdenticFields {
    /// `myUsers`.
    pub users: Vec<Handle<StandardTransient>>,
    /// `isCircle`.
    pub is_circle: bool,
    /// `myFAttach`.
    pub f_attach: Point3,
    /// `mySAttach`.
    pub s_attach: Point3,
    /// `myCenter`.
    pub center: Point3,
}

prs_dim_relation_class!(
    PrsDimIdenticRelation,
    "PrsDim_IdenticRelation",
    PrsDimIdenticFields,
    "`PrsDim_IdenticRelation`; `PrsDimIdenticRelation::new(&s1, &s2, &plane)`.",
    movable = true
);

/// The file-local `ConnectedEdges`: the two edges of a wire at a vertex.
fn connected_edges(wire: &Wire, vertex: &Vertex) -> Result<(Option<Edge>, Option<Edge>), Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        &wire.clone().into(),
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut map,
    )?;
    let target: Shape = vertex.clone().into();
    let mut key = None;
    for index in 1..=map.extent() {
        let candidate = map.find_key(index)?;
        if candidate.is_same(&target) {
            key = Some(candidate.clone());
            break;
        }
    }
    let Some(key) = key else {
        return Ok((None, None));
    };
    let ancestors: Vec<Shape> = map.find_from_key(&key)?.iter().cloned().collect();
    match ancestors.len() {
        0 => Ok((None, None)),
        1 => Ok((Some(topods::edge(&ancestors[0])?), None)),
        2 => Ok((
            Some(topods::edge(&ancestors[0])?),
            Some(topods::edge(&ancestors[1])?),
        )),
        _ => Ok((None, None)),
    }
}

impl PrsDimIdenticRelation {
    /// `PrsDim_IdenticRelation(shape1, shape2, plane)`.
    /// Example: `PrsDimIdenticRelation::new(&s1, &s2, &plane)`.
    pub fn new(first: &Shape, second: &Shape, plane: &Handle<GeomPlane>) -> Handle<Self> {
        let relation = Self::alloc(PrsDimIdenticFields::default());
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            f.plane = plane.clone();
        }
        relation
    }
    /// `HasUsers`; `relation.has_users()`.
    pub fn has_users(&self) -> bool {
        !self.fields.read().users.is_empty()
    }
    /// `Users`; `relation.users()`.
    pub fn users(&self) -> Vec<Handle<StandardTransient>> {
        self.fields.read().users.clone()
    }
    /// `AddUser`; `relation.add_user(&user)`.
    pub fn add_user(&self, user: &Handle<StandardTransient>) {
        self.fields.write().users.push(user.clone());
    }
    /// `ClearUsers`; `relation.clear_users()`.
    pub fn clear_users(&self) {
        self.fields.write().users.clear();
    }
    /// `ComputeSegSize`: 1, as natively.
    pub fn compute_seg_size(&self) -> f64 {
        1.0
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let (first, second) = (
            self.first_shape().shape_type()?,
            self.second_shape().shape_type()?,
        );
        match (first, second) {
            (ShapeEnum::Vertex, ShapeEnum::Vertex) => self.compute_two_vertices_presentation(prs),
            (ShapeEnum::Vertex, ShapeEnum::Edge) | (ShapeEnum::Edge, ShapeEnum::Vertex) => {
                self.compute_one_edge_o_vertex_presentation(prs)
            }
            (ShapeEnum::Edge, ShapeEnum::Edge) => self.compute_two_edges_presentation(prs),
            _ => Ok(()),
        }
    }

    /// `ComputeTwoVerticesPresentation`: the mark beside two coincident vertices (oriented by
    /// the wire of the first user shape).
    /// Example: `relation.compute_two_vertices_presentation(&prs)?`.
    pub fn compute_two_vertices_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::vertex(&self.first_shape())?;
        let second = topods::vertex(&self.second_shape())?;
        let plane = self.plane();
        let (a1, on1) = PrsDim::compute_vertex_geometry(&first, &plane)?;
        let (a2, on2) = PrsDim::compute_vertex_geometry(&second, &plane)?;
        {
            let mut f = self.fields.write();
            f.f_attach = a1;
            f.s_attach = a2;
        }
        let ext = match (on1, on2) {
            (true, true) => 0,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => return Ok(()),
        };
        self.set_ext_shape(ext);
        let attach = BRepTool::pnt(&first)?;
        {
            let mut f = self.fields.write();
            f.f_attach = attach;
            f.s_attach = attach;
        }
        let position = if self.automatic_position() {
            let mut size = self.compute_seg_size();
            if size <= precision::confusion() {
                size = 1.0;
            }
            size *= 5.0;
            let position = match self.users().first() {
                Some(user) => {
                    let Some(direction) = self.user_direction(user, &first, &second)? else {
                        return Ok(());
                    };
                    attach.translated(Vector3::from(direction) * size)
                }
                None => attach,
            };
            let mut r = self.relation_base().fields.write();
            r.position = position;
            r.automatic_position = false;
            position
        } else {
            self.position()
        };
        let text = ExtendedString::from_utf8(" ++");
        DsgPrsIdenticPresentation::add(prs, &self.attributes(), &text, &attach, &position)?;
        let style = PrsDimProjectionStyle::default();
        match ext {
            1 => self.compute_proj_vertex_presentation(prs, &first, &attach, &style),
            2 => self.compute_proj_vertex_presentation(prs, &second, &attach, &style),
            _ => Ok(()),
        }
    }

    /// The symbol direction from the wire of a user shape at both vertices (`None` when the
    /// user is not a shape of a wire or a direction fails).
    fn user_direction(
        &self,
        user: &Handle<StandardTransient>,
        first: &Vertex,
        second: &Vertex,
    ) -> Result<Option<Direction3>, Failure> {
        let shape_user = user.clone().cast::<AisShape>();
        let Some(shape_user) = shape_user.get() else {
            return Ok(None);
        };
        let shape = shape_user.shape();
        if shape.is_null() || shape.shape_type()? != ShapeEnum::Wire {
            return Ok(None);
        }
        let wire = topods::wire(&shape)?;
        let Some(df) = self.compute_direction(&wire, first)? else {
            return Ok(None);
        };
        let Some(ds) = self.compute_direction(&wire, second)? else {
            return Ok(None);
        };
        let normal = self
            .plane()
            .get()
            .ok_or_else(null_object)?
            .pln()
            .axis()
            .direction();
        Ok(Some(if df.is_parallel(ds, precision::angular()) {
            df.crossed(normal)?
        } else {
            Direction3::from_xyz(df.xyz() + ds.xyz())?
        }))
    }

    /// `ComputeDirection`: the bisector of the edges of a wire at a vertex (`None` when not
    /// computed). Example: `relation.compute_direction(&wire, &vertex)?`.
    pub fn compute_direction(
        &self,
        wire: &Wire,
        vertex: &Vertex,
    ) -> Result<Option<Direction3>, Failure> {
        let (e1, e2) = connected_edges(wire, vertex)?;
        let edge_direction =
            |curve: &Handle<dyn GeomCurve>, first: Point3| -> Result<Option<Direction3>, Failure> {
                if let Some(circle) = curve.cast::<GeomCircle>().get() {
                    return Ok(Some(self.compute_circle_direction(circle, vertex)?));
                }
                if let Some(line) = curve.cast::<GeomLine>().get() {
                    return Ok(Some(self.compute_line_direction(line, &first)));
                }
                Ok(None)
            };
        match (e1, e2) {
            (Some(e1), Some(e2)) => {
                let Some(edges) = PrsDim::compute_two_edges_on_plane(&e1, &e2, &self.plane())?
                else {
                    return Ok(None);
                };
                let Some(d1) = edge_direction(&edges.first_curve, edges.first_ends.0)? else {
                    return Ok(None);
                };
                let Some(d2) = edge_direction(&edges.second_curve, edges.second_ends.0)? else {
                    return Ok(None);
                };
                if !d1.is_parallel(d2, precision::angular()) {
                    return Ok(Some(Direction3::from_xyz((d1.xyz() + d2.xyz()) / 2.0)?));
                }
                let normal = self
                    .plane()
                    .get()
                    .ok_or_else(null_object)?
                    .pln()
                    .axis()
                    .direction();
                Ok(Some(d1.crossed(normal)?))
            }
            (Some(edge), None) | (None, Some(edge)) => {
                let Some((curve, (first, _))) = PrsDim::compute_edge_geometry(&edge)? else {
                    return Ok(None);
                };
                edge_direction(&curve, first)
            }
            (None, None) => Ok(None),
        }
    }

    /// `ComputeLineDirection`: the line direction, reversed unless the first attach point is
    /// the first point. Example: `relation.compute_line_direction(&line, &p)`.
    pub fn compute_line_direction(&self, line: &GeomLine, first: &Point3) -> Direction3 {
        let direction = line.lin().direction();
        if !self
            .fields
            .read()
            .f_attach
            .is_equal(first, precision::confusion())
        {
            return direction.reversed();
        }
        direction
    }
    /// `ComputeCircleDirection`: from the circle center to the vertex.
    /// Example: `relation.compute_circle_direction(&circle, &vertex)?`.
    pub fn compute_circle_direction(
        &self,
        circle: &GeomCircle,
        vertex: &Vertex,
    ) -> Result<Direction3, Failure> {
        Direction3::from_vector(Vector3::between(circle.location(), BRepTool::pnt(vertex)?))
    }

    /// `ComputeOneEdgeOVertexPresentation`: the mark of a vertex on an edge.
    /// Example: `relation.compute_one_edge_o_vertex_presentation(&prs)?`.
    pub fn compute_one_edge_o_vertex_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let (first, second) = (self.first_shape(), self.second_shape());
        let (vertex, edge, edge_number) = if first.shape_type()? == ShapeEnum::Vertex {
            (topods::vertex(&first)?, topods::edge(&second)?, 2)
        } else {
            (topods::vertex(&second)?, topods::edge(&first)?, 1)
        };
        let plane = self.plane();
        let Some(projected) = PrsDim::compute_edge_geometry_on_plane(&edge, &plane)? else {
            return Ok(());
        };
        prs.get()
            .ok_or_else(null_object)?
            .set_infinite_state(projected.infinite);
        let (projected_vertex, vertex_on_plane) = PrsDim::compute_vertex_geometry(&vertex, &plane)?;
        self.fields.write().f_attach = projected_vertex;
        if !projected.on_plane && !vertex_on_plane {
            return Ok(());
        }
        if !projected.on_plane {
            self.set_ext_shape(if edge_number == 1 { 1 } else { 2 });
        } else if !vertex_on_plane {
            self.set_ext_shape(if edge_number == 1 { 2 } else { 1 });
        }
        let attach = BRepTool::pnt(&vertex)?;
        {
            let mut f = self.fields.write();
            f.f_attach = attach;
            f.s_attach = attach;
        }
        let position = if self.automatic_position() {
            let size = self.compute_seg_size() * 5.0;
            let curve = &projected.curve;
            let mut direction = Direction3::default();
            if let Some(line) = curve.cast::<GeomLine>().get() {
                let normal = plane
                    .get()
                    .ok_or_else(null_object)?
                    .pln()
                    .axis()
                    .direction();
                direction = line.lin().direction().crossed(normal)?;
            } else if let Some(circle) = curve.cast::<GeomCircle>().get() {
                direction = Direction3::from_xyz(attach.xyz() - circle.location().xyz())?;
            } else if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
                direction = Direction3::from_xyz(attach.xyz() - ellipse.location().xyz())?;
            }
            let position = attach.translated(Vector3::from(direction) * size);
            let mut r = self.relation_base().fields.write();
            r.position = position;
            r.automatic_position = true;
            position
        } else {
            self.position()
        };
        let text = ExtendedString::from_utf8(" -+-");
        DsgPrsIdenticPresentation::add(prs, &self.attributes(), &text, &attach, &position)?;
        if self.ext_shape() != 0 && !projected.ext_curve.is_null() {
            let line = projected.curve.cast::<GeomLine>();
            if line.is_null() {
                return Err(null_object());
            }
            let style = PrsDimProjectionStyle::default();
            let (a, b) = projected.ends;
            self.compute_proj_edge_presentation(prs, &edge, &line.cast(), a, b, &style)?;
        }
        Ok(())
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let owner = relation_owner(self, 7);
        let target = selection.get().ok_or_else(null_object)?;
        let f = self.fields.read().clone();
        let position = self.position();
        let confusion = precision::confusion();
        let mut attach = Point3::default();
        if f.f_attach.is_equal(&f.s_attach, confusion) {
            attach = f.f_attach;
        } else if self.first_shape().shape_type()? == ShapeEnum::Edge {
            let first = topods::edge(&self.first_shape())?;
            let second = topods::edge(&self.second_shape())?;
            let Some((done, edges)) =
                PrsDim::compute_two_edges_ext(&first, &second, &self.plane())?
            else {
                return Ok(());
            };
            self.set_ext_shape(edges.ext_index);
            if !done {
                return Ok(());
            }
            let curve = &edges.first_curve;
            let curve_entity =
                |basis: &Handle<dyn GeomCurve>, u1: f64, u2: f64| -> Result<(), Failure> {
                    let trimmed = GeomTrimmedCurve::new(basis, u1, u2, true, true)?;
                    let entity = Select3DSensitiveCurve::from_curve(&owner, &trimmed, 17)?;
                    target.add(&Handle::new(entity).cast())
                };
            if f.is_circle {
                let circ = curve
                    .cast::<GeomCircle>()
                    .get()
                    .ok_or_else(null_object)?
                    .circ()?;
                curve_entity(
                    curve,
                    elclib::parameter(&circ, f.f_attach)?,
                    elclib::parameter(&circ, f.s_attach)?,
                )?;
                attach = compute_attach(&circ, f.f_attach, f.s_attach, position)?;
            } else if let Some(ellipse) = curve
                .cast::<GeomEllipse>()
                .get()
                .filter(|_| is_exact::<GeomEllipse>(curve))
            {
                let elips = ellipse.elips()?;
                curve_entity(
                    curve,
                    elclib::parameter(&elips, f.f_attach)?,
                    elclib::parameter(&elips, f.s_attach)?,
                )?;
                attach = compute_attach(&elips, f.f_attach, f.s_attach, position)?;
            } else if is_exact::<GeomLine>(curve) {
                target.add(
                    &Handle::new(Select3DSensitiveSegment::new(
                        &owner, f.f_attach, f.s_attach,
                    ))
                    .cast(),
                )?;
                let v1 = Vector3::between(f.f_attach, f.s_attach);
                let v2 = Vector3::between(f.f_attach, position);
                attach = if v1.is_parallel(v2, precision::angular())? {
                    f.s_attach
                } else {
                    let line = Line3::new(f.f_attach, Direction3::from_vector(v1)?);
                    elclib::value(elclib::parameter(&line, position)?, &line)
                };
            } else {
                return Ok(());
            }
        }
        if !attach.is_equal(&position, confusion) {
            target.add(
                &Handle::new(Select3DSensitiveSegment::new(&owner, attach, position)).cast(),
            )?;
        }
        Ok(())
    }
}

/// Whether a curve is exactly of a type (`IsInstance`).
fn is_exact<T: rustycad_tkernel::standard::RuntimeTyped + 'static>(
    curve: &Handle<dyn GeomCurve>,
) -> bool {
    curve.get().is_some_and(|c| c.is_instance(T::static_type()))
}
