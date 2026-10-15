//! `PrsDim_EqualDistanceRelation`: two equal distances (each between two edges or vertices),
//! drawn as two intervals joined by an equality mark.
use super::dimension_impls::prs_dim_relation_class;
use super::equal_distance_length::{PrsDimEqualDistanceContext, PrsDimEqualDistancePoints};
use super::relation_support::{null_object, relation_arrow, relation_owner};
use super::*;
use crate::dsg_prs::{DsgPrsArrowSide, DsgPrsEqualDistancePresentation};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveBox, Select3DSensitivePoly, Select3DSensitiveSegment};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::f64::consts::PI;

/// The third and fourth shapes, and the attach and extreme points of the two distances.
#[derive(Clone, Default)]
pub struct PrsDimEqualDistanceFields {
    /// `myShape3`.
    pub shape3: Shape,
    /// `myShape4`.
    pub shape4: Shape,
    /// `myAttachPoint1` .. `myAttachPoint4`.
    pub attach_points: [Point3; 4],
    /// `myPoint1` .. `myPoint4`.
    pub points: [Point3; 4],
}

prs_dim_relation_class!(
    PrsDimEqualDistanceRelation,
    "PrsDim_EqualDistanceRelation",
    PrsDimEqualDistanceFields,
    "`PrsDim_EqualDistanceRelation`; `PrsDimEqualDistanceRelation::new(&s1, &s2, &s3, &s4, &plane)`."
);

impl PrsDimEqualDistanceRelation {
    /// `PrsDim_EqualDistanceRelation(s1, s2, s3, s4, plane)`: arrow size 3, both arrows.
    /// Example: `PrsDimEqualDistanceRelation::new(&s1, &s2, &s3, &s4, &plane)`.
    pub fn new(
        first: &Shape,
        second: &Shape,
        third: &Shape,
        fourth: &Shape,
        plane: &Handle<GeomPlane>,
    ) -> Handle<Self> {
        let relation = Self::alloc(PrsDimEqualDistanceFields {
            shape3: third.clone(),
            shape4: fourth.clone(),
            ..PrsDimEqualDistanceFields::default()
        });
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            f.plane = plane.clone();
            f.arrow_size = 3.0;
            f.symbol_prs = DsgPrsArrowSide::BothAr;
        }
        relation
    }
    /// `SetShape3`; `relation.set_shape3(&shape)`.
    pub fn set_shape3(&self, shape: &Shape) {
        self.fields.write().shape3 = shape.clone();
    }
    /// `Shape3`; `relation.shape3()`.
    pub fn shape3(&self) -> Shape {
        self.fields.read().shape3.clone()
    }
    /// `SetShape4`; `relation.set_shape4(&shape)`.
    pub fn set_shape4(&self, shape: &Shape) {
        self.fields.write().shape4 = shape.clone();
    }
    /// `Shape4`; `relation.shape4()`.
    pub fn shape4(&self) -> Shape {
        self.fields.read().shape4.clone()
    }

    /// The length of one pair of shapes into the points of the pair (0 or 1).
    fn pair_length(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ctx: &PrsDimEqualDistanceContext,
        (first, second): (Shape, Shape),
        pair: usize,
        position: Point3,
    ) -> Result<(), Failure> {
        let f = self.fields.read().clone();
        let mut points = PrsDimEqualDistancePoints {
            position,
            first_attach: f.attach_points[2 * pair],
            second_attach: f.attach_points[2 * pair + 1],
            first_extreme: f.points[2 * pair],
            second_extreme: f.points[2 * pair + 1],
        };
        let (k1, k2) = (first.shape_type()?, second.shape_type()?);
        let result = if k1 == ShapeEnum::Edge && k2 == ShapeEnum::Edge {
            Self::compute_two_edges_length(
                prs,
                ctx,
                &topods::edge(&first)?,
                &topods::edge(&second)?,
                &mut points,
            )
        } else if k1 == ShapeEnum::Vertex && k2 == ShapeEnum::Vertex {
            Self::compute_two_vertices_length(
                prs,
                ctx,
                &topods::vertex(&first)?,
                &topods::vertex(&second)?,
                PrsDimTypeOfDist::Unknown,
                &mut points,
            )
        } else {
            Self::compute_one_edge_one_vertex_length(prs, ctx, &first, &second, &mut points)
        };
        let mut f = self.fields.write();
        f.attach_points[2 * pair] = points.first_attach;
        f.attach_points[2 * pair + 1] = points.second_attach;
        f.points[2 * pair] = points.first_extreme;
        f.points[2 * pair + 1] = points.second_extreme;
        result
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let r = self.relation_base().fields.read().clone();
        let (mut position12, mut position34) = (r.position, r.position);
        relation_arrow(self)?
            .get()
            .ok_or_else(null_object)?
            .set_length(r.arrow_size);
        let p = self.fields.read().points;
        if !r.automatic_position {
            let middle12 = Point3::from((p[0].xyz() + p[1].xyz()) * 0.5);
            let middle34 = Point3::from((p[2].xyz() + p[3].xyz()) * 0.5);
            if r.position.distance(&middle12) > r.position.distance(&middle34) {
                position12 = middle12;
            } else {
                position34 = middle34;
            }
        }
        let ctx = PrsDimEqualDistanceContext {
            drawer: self.attributes(),
            arrow_size: r.arrow_size,
            plane: r.plane.clone(),
            automatic: r.automatic_position,
            is_set_bnd_box: r.is_set_bnd_box,
            bnd_box: r.bnd_box.clone(),
            symbol: r.symbol_prs,
        };
        let f = self.fields.read().clone();
        self.pair_length(
            prs,
            &ctx,
            (r.f_shape.clone(), r.s_shape.clone()),
            0,
            position12,
        )?;
        self.pair_length(prs, &ctx, (f.shape3, f.shape4), 1, position34)?;
        let p = self.fields.read().points;
        DsgPrsEqualDistancePresentation::add(
            prs,
            &self.attributes(),
            &p[0],
            &p[1],
            &p[2],
            &p[3],
            &r.plane,
        )
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
        let f = self.fields.read().clone();
        let (p, a) = (f.points, f.attach_points);
        segment(p[0], p[1])?;
        segment(p[2], p[3])?;
        let middle12 = Point3::from((p[0].xyz() + p[1].xyz()) * 0.5);
        let middle34 = Point3::from((p[2].xyz() + p[3].xyz()) * 0.5);
        segment(middle12, middle34)?;
        let middle = Point3::from((middle12.xyz() + middle34.xyz()) * 0.5);
        let small = 0.001;
        let bounds = Select3DSensitiveBox::from_bounds(
            &owner,
            middle.x() - small,
            middle.y() - small,
            middle.z() - small,
            middle.x() + small,
            middle.y() + small,
            middle.z() + small,
        );
        target.add(&Handle::new(bounds).cast())?;
        let shapes = [self.first_shape(), self.second_shape(), f.shape3, f.shape4];
        for (index, shape) in shapes.iter().enumerate() {
            let (attach, point) = (a[index], p[index]);
            if shape.shape_type()? != ShapeEnum::Edge {
                segment(attach, point)?;
                continue;
            }
            let curve = BRepAdaptorCurve::with_edge(&topods::edge(shape)?)?;
            match curve.get_type()? {
                CurveType::Line => segment(attach, point)?,
                CurveType::Circle => {
                    let geom = curve
                        .transformed_curve()
                        .curve()
                        .get()
                        .map(|c| c.curve())
                        .unwrap_or_default();
                    let circle = geom
                        .cast::<GeomCircle>()
                        .get()
                        .ok_or_else(null_object)?
                        .circ()?;
                    let first = elclib::parameter(&circle, attach)?;
                    let mut last = elclib::parameter(&circle, point)?;
                    if last < first {
                        last += PI * 2.0;
                    }
                    let arc =
                        Select3DSensitivePoly::circle(&owner, &circle, first, last, false, 12)?;
                    target.add(&Handle::new(arc).cast())?;
                }
                // The native third shape adds a segment for other curves.
                _ if index == 2 => segment(attach, point)?,
                _ => {}
            }
        }
        Ok(())
    }
}
