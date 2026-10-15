//! `PrsDim_ConcentricRelation`: circles (or a circle and a vertex) sharing a center, drawn as
//! two small concentric circles with a cross.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_owner};
use super::*;
use crate::dsg_prs::DsgPrsConcentricPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveCircle, Select3DSensitiveSegment};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomConic, GeomCurve, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};
use std::f64::consts::FRAC_PI_2;

/// The center, radius, normal and cross point of `PrsDim_ConcentricRelation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimConcentricFields {
    /// `myCenter`.
    pub center: Point3,
    /// `myRad`.
    pub rad: f64,
    /// `myDir`.
    pub dir: Direction3,
    /// `myPnt`.
    pub pnt: Point3,
}

prs_dim_relation_class!(
    PrsDimConcentricRelation,
    "PrsDim_ConcentricRelation",
    PrsDimConcentricFields,
    "`PrsDim_ConcentricRelation`; `PrsDimConcentricRelation::new(&s1, &s2, &plane)?`."
);

fn circle_of(curve: &Handle<dyn GeomCurve>) -> Result<Handle<GeomCircle>, Failure> {
    let circle = curve.cast::<GeomCircle>();
    if circle.is_null() {
        return Err(null_object());
    }
    Ok(circle)
}

impl PrsDimConcentricRelation {
    /// `PrsDim_ConcentricRelation(shape1, shape2, plane)`: the plane normal as the direction.
    /// Example: `PrsDimConcentricRelation::new(&s1, &s2, &plane)?`.
    pub fn new(
        first: &Shape,
        second: &Shape,
        plane: &Handle<GeomPlane>,
    ) -> Result<Handle<Self>, Failure> {
        let dir = plane
            .get()
            .ok_or_else(null_object)?
            .pln()
            .axis()
            .direction();
        let relation = Self::alloc(PrsDimConcentricFields {
            dir,
            ..PrsDimConcentricFields::default()
        });
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            f.plane = plane.clone();
        }
        Ok(relation)
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let (first, second) = (self.first_shape(), self.second_shape());
        let second_type = second.shape_type()?;
        prs.get().ok_or_else(null_object)?.set_infinite_state(true);
        match first.shape_type()? {
            ShapeEnum::Edge if second_type == ShapeEnum::Edge => {
                self.compute_two_edges_concentric(prs)
            }
            ShapeEnum::Edge if second_type == ShapeEnum::Vertex => {
                self.compute_edge_vertex_concentric(prs)
            }
            ShapeEnum::Vertex if second_type == ShapeEnum::Vertex => {
                self.compute_two_vertices_concentric(prs)
            }
            ShapeEnum::Vertex if second_type == ShapeEnum::Edge => {
                self.compute_edge_vertex_concentric(prs)
            }
            _ => Ok(()),
        }
    }

    /// The cross point at the radius from the center toward a point, then the presentation.
    fn add_concentric(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        center: Point3,
        rad: f64,
        toward: Direction3,
    ) -> Result<(), Failure> {
        let mut f = *self.fields.read();
        f.center = center;
        f.rad = rad;
        f.pnt = center.translated(Vector3::from(toward) * rad);
        *self.fields.write() = f;
        DsgPrsConcentricPresentation::add(prs, &self.attributes(), &f.center, f.rad, &f.dir, &f.pnt)
    }

    /// `ComputeEdgeVertexConcentric`; `relation.compute_edge_vertex_concentric(&prs)?`.
    pub fn compute_edge_vertex_concentric(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let (first, second) = (self.first_shape(), self.second_shape());
        let (edge, vertex): (Edge, Vertex) = if first.shape_type()? == ShapeEnum::Edge {
            (topods::edge(&first)?, topods::vertex(&second)?)
        } else {
            (topods::edge(&second)?, topods::vertex(&first)?)
        };
        let plane = self.plane();
        let Some(projected) = PrsDim::compute_edge_geometry_on_plane(&edge, &plane)? else {
            return Ok(());
        };
        let (point, vertex_on_plane) = PrsDim::compute_vertex_geometry(&vertex, &plane)?;
        let circle = circle_of(&projected.curve)?;
        let value = circle.get().ok_or_else(null_object)?;
        let center = value.location();
        let rad = (value.radius() / 5.0).min(15.0);
        let toward = Direction3::from_xyz(projected.ends.0.xyz() - center.xyz())?;
        self.add_concentric(prs, center, rad, toward)?;
        let drawer = self.attributes();
        let style = PrsDimProjectionStyle::default();
        if !projected.on_plane {
            PrsDim::compute_proj_edge_presentation(
                prs,
                &drawer,
                &edge,
                &circle.cast(),
                projected.ends,
                &style,
            )?;
        }
        if !vertex_on_plane {
            PrsDim::compute_proj_vertex_presentation(prs, &drawer, &vertex, &point, &style)?;
        }
        Ok(())
    }

    /// `ComputeTwoVerticesConcentric` (both vertices read from the first shape, as natively).
    /// Example: `relation.compute_two_vertices_concentric(&prs)?`.
    pub fn compute_two_vertices_concentric(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::vertex(&self.first_shape())?;
        let second = topods::vertex(&self.first_shape())?;
        let plane = self.plane();
        let (p1, on_plane1) = PrsDim::compute_vertex_geometry(&first, &plane)?;
        let (p2, on_plane2) = PrsDim::compute_vertex_geometry(&second, &plane)?;
        let x = plane
            .get()
            .ok_or_else(null_object)?
            .pln()
            .position()
            .x_direction();
        self.add_concentric(prs, p1, 15.0, x)?;
        let drawer = self.attributes();
        let style = PrsDimProjectionStyle::default();
        if !on_plane1 {
            PrsDim::compute_proj_vertex_presentation(prs, &drawer, &first, &p1, &style)?;
        }
        if !on_plane2 {
            PrsDim::compute_proj_vertex_presentation(prs, &drawer, &second, &p2, &style)?;
        }
        Ok(())
    }

    /// `ComputeTwoEdgesConcentric`; `relation.compute_two_edges_concentric(&prs)?`.
    pub fn compute_two_edges_concentric(
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
        let circle1 = circle_of(&edges.first_curve)?;
        let circle2 = circle_of(&edges.second_curve)?;
        let (c1, c2) = (
            circle1.get().ok_or_else(null_object)?,
            circle2.get().ok_or_else(null_object)?,
        );
        let center = c1.location();
        let (r1, r2) = (c1.radius(), c2.radius());
        let mut rad = if r1 > r2 { r2 } else { r1 };
        rad /= 5.0;
        if rad > 15.0 {
            rad = 15.0;
        }
        let toward = Direction3::from_xyz(edges.first_ends.0.xyz() - center.xyz())?;
        self.add_concentric(prs, center, rad, toward)?;
        let ext = self.ext_shape();
        if ext == 0 || edges.ext_curve.is_null() {
            return Ok(());
        }
        let style = PrsDimProjectionStyle::default();
        let unset = (Point3::default(), Point3::default());
        if ext == 1 {
            let ends = if edges.first_infinite {
                unset
            } else {
                edges.first_ends
            };
            self.compute_proj_edge_presentation(
                prs,
                &first,
                &circle1.cast(),
                ends.0,
                ends.1,
                &style,
            )
        } else {
            let ends = if edges.second_infinite {
                unset
            } else {
                edges.second_ends
            };
            self.compute_proj_edge_presentation(
                prs,
                &second,
                &circle2.cast(),
                ends.0,
                ends.1,
                &style,
            )
        }
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let owner = relation_owner(self, 7);
        let target = selection.get().ok_or_else(null_object)?;
        let f = *self.fields.read();
        let mut circle = Circle3::new(Axis2::new(f.center, f.dir)?, f.rad)?;
        target.add(&Handle::new(Select3DSensitiveCircle::new(&owner, &circle, false)).cast())?;
        circle.set_radius(f.rad / 2.0)?;
        target.add(&Handle::new(Select3DSensitiveCircle::new(&owner, &circle, false)).cast())?;
        let other = f.pnt.mirrored_point(f.center);
        target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, other, f.pnt)).cast())?;
        let axis = Axis1::new(f.center, f.dir);
        let first = f.center.rotated(axis, FRAC_PI_2)?;
        let second = f.center.rotated(axis, -FRAC_PI_2)?;
        target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, first, second)).cast())
    }
}
