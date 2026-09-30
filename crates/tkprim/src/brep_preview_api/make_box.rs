// Rust translation of BRepPreviewAPI_MakeBox.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPreviewAPI_MakeBox`: the box maker that accepts flat boxes.
use crate::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeShapeBase, BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};
use std::ops::{Deref, DerefMut};

/// `BRepPreviewAPI_MakeBox`: a [`BRepPrimAPIMakeBox`] (it dereferences to it, as the
/// native class derives from it) whose `Build` makes a vertex, an edge or a planar face
/// when the box is thin (below Precision::Confusion()) in three, two or one of the
/// directions, and the solid otherwise. The corners are the location of the frame and
/// that location plus (XMax, YMax, ZMax) of the wedge: the directions of the frame are
/// ignored (docs/prim-design.md section 5).
///
/// ```
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkprim::brep_preview_api::BRepPreviewAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut preview = BRepPreviewAPIMakeBox::new();
/// preview.init_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0))?;
/// assert_eq!(preview.shape()?.shape_type()?, ShapeEnum::Face);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct BRepPreviewAPIMakeBox {
    my_box: BRepPrimAPIMakeBox,
}

impl Deref for BRepPreviewAPIMakeBox {
    type Target = BRepPrimAPIMakeBox;
    fn deref(&self) -> &BRepPrimAPIMakeBox {
        &self.my_box
    }
}

impl DerefMut for BRepPreviewAPIMakeBox {
    fn deref_mut(&mut self) -> &mut BRepPrimAPIMakeBox {
        &mut self.my_box
    }
}

/// The corners of the rectangle of a box thin in one direction, from its two corners.
fn rectangle_corners(first: Point3, second: Point3, thin: [bool; 3]) -> [Point3; 4] {
    let (f, s) = (first, second);
    if thin[0] {
        return [
            Point3::new(f.x(), f.y(), f.z()),
            Point3::new(f.x(), s.y(), f.z()),
            Point3::new(f.x(), s.y(), s.z()),
            Point3::new(f.x(), f.y(), s.z()),
        ];
    }
    if thin[1] {
        return [
            Point3::new(f.x(), f.y(), f.z()),
            Point3::new(s.x(), f.y(), f.z()),
            Point3::new(s.x(), f.y(), s.z()),
            Point3::new(f.x(), f.y(), s.z()),
        ];
    }
    [
        Point3::new(f.x(), f.y(), f.z()),
        Point3::new(s.x(), f.y(), f.z()),
        Point3::new(s.x(), s.y(), f.z()),
        Point3::new(f.x(), s.y(), f.z()),
    ]
}

/// `makeVertex`: the vertex at the point.
fn make_vertex(point: Point3) -> Result<Shape, Failure> {
    Ok(BRepBuilderAPIMakeVertex::new(point)?.vertex()?.into())
}

/// `makeEdge`: the segment between the points.
fn make_edge(point1: Point3, point2: Point3) -> Result<Shape, Failure> {
    Ok(BRepBuilderAPIMakeEdge::from_points(point1, point2)?
        .edge()?
        .into())
}

/// `makeRectangle`: the planar face bounded by the four segments between the corners.
fn make_rectangle(corners: [Point3; 4]) -> Result<Shape, Failure> {
    let [p1, p2, p3, p4] = corners;
    let edge1 = BRepBuilderAPIMakeEdge::from_points(p1, p2)?.edge()?;
    let edge2 = BRepBuilderAPIMakeEdge::from_points(p2, p3)?.edge()?;
    let edge3 = BRepBuilderAPIMakeEdge::from_points(p3, p4)?.edge()?;
    let edge4 = BRepBuilderAPIMakeEdge::from_points(p4, p1)?.edge()?;
    let wire = BRepBuilderAPIMakeWire::from_four_edges(&edge1, &edge2, &edge3, &edge4)?.wire()?;
    let mut face = BRepBuilderAPIMakeFace::from_wire(&wire, false)?;
    Ok(face.shape()?.clone())
}

impl BRepPreviewAPIMakeBox {
    /// `BRepPreviewAPI_MakeBox()`: an empty box, set by the `Init` members of the box
    /// maker; `BRepPreviewAPIMakeBox::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The preview shape of a thin box, or a null shape for a box thin in no direction.
    fn thin_shape(&self) -> Result<Shape, Failure> {
        let location = self.my_box.my_wedge.axes().location();
        let first = Point3::new(location.x(), location.y(), location.z());
        let wedge = &self.my_box.my_wedge;
        let second = Point3::new(
            location.x() + wedge.get_xmax(),
            location.y() + wedge.get_ymax(),
            location.z() + wedge.get_zmax(),
        );
        let thin = [
            (first.x() - second.x()).abs() < precision::confusion(),
            (first.y() - second.y()).abs() < precision::confusion(),
            (first.z() - second.z()).abs() < precision::confusion(),
        ];
        match thin.iter().filter(|flag| **flag).count() {
            3 => make_vertex(first),
            2 => make_edge(first, second),
            1 => make_rectangle(rectangle_corners(first, second, thin)),
            _ => Ok(Shape::default()),
        }
    }
}

impl BRepBuilderAPIMakeShape for BRepPreviewAPIMakeBox {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        self.my_box.make_shape_base()
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        self.my_box.make_shape_base_mut()
    }

    /// `Build()`: the vertex, edge or face of a thin box; otherwise the shape is kept
    /// when it is not null (only marked done), else the solid of the box is built.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        let thin = self.thin_shape()?;
        if !thin.is_null() {
            self.make_shape_base_mut().my_shape = thin;
        }
        if !self.make_shape_base().my_shape.is_null() {
            self.make_shape_base_mut().set_done();
            return Ok(());
        }
        self.my_box.solid()?;
        Ok(())
    }
}
