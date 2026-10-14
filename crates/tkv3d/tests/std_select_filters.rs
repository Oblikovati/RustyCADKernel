//! StdSelect_EdgeFilter, StdSelect_FaceFilter and StdSelect_ShapeTypeFilter on the BRep owners
//! of the sub-shapes of a box and a cylinder (the native IsOk/ActsOn bodies).
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis,
};
use rustycad_tkv3d::select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerSource, SelectMgrFilter};
use rustycad_tkv3d::std_select::{
    StdSelectBRepOwner, StdSelectEdgeFilter, StdSelectFaceFilter, StdSelectShapeTypeFilter,
    StdSelectTypeOfEdge, StdSelectTypeOfFace,
};

fn owner(shape: &Shape) -> Handle<dyn SelectMgrEntityOwnerSource> {
    StdSelectBRepOwner::with_shape(shape, 5, false).into_cast()
}

fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}

fn cylinder() -> Result<Shape, Failure> {
    Ok(Shape::from(BRepPrimAPIMakeCylinder::new(2.0, 3.0).solid()?))
}

fn box_shape() -> Result<Shape, Failure> {
    Ok(Shape::from(BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)?.solid()?))
}

fn accepted(filter: &dyn SelectMgrFilter, shapes: &[Shape]) -> Result<usize, Failure> {
    let mut count = 0;
    for shape in shapes {
        count += usize::from(filter.is_ok(&owner(shape))?);
    }
    Ok(count)
}

#[test]
fn edge_filter_accepts_edges_by_curve_type() -> Result<(), Failure> {
    let edges = sub_shapes(&cylinder()?, ShapeEnum::Edge)?;
    let faces = sub_shapes(&cylinder()?, ShapeEnum::Face)?;
    let filter = StdSelectEdgeFilter::new(StdSelectTypeOfEdge::AnyEdge);
    assert_eq!(accepted(&filter, &edges)?, edges.len());
    assert_eq!(accepted(&filter, &faces)?, 0);
    filter.set_type(StdSelectTypeOfEdge::Circle);
    assert_eq!(filter.type_(), StdSelectTypeOfEdge::Circle);
    // The cylinder has two circles and one seam line (the edges are shared by orientation).
    let circles = accepted(&filter, &edges)?;
    filter.set_type(StdSelectTypeOfEdge::Line);
    let lines = accepted(&filter, &edges)?;
    assert_eq!(circles + lines, edges.len());
    assert!(circles >= 2 && lines >= 1);
    assert!(filter.acts_on(ShapeEnum::Edge));
    assert!(!filter.acts_on(ShapeEnum::Face));
    Ok(())
}

#[test]
fn face_filter_accepts_faces_by_surface_type() -> Result<(), Failure> {
    let faces = sub_shapes(&cylinder()?, ShapeEnum::Face)?;
    let filter = StdSelectFaceFilter::new(StdSelectTypeOfFace::Plane);
    assert_eq!(accepted(&filter, &faces)?, 2);
    for (kind, count) in [
        (StdSelectTypeOfFace::AnyFace, 3),
        (StdSelectTypeOfFace::Cylinder, 1),
        (StdSelectTypeOfFace::Revol, 1),
        (StdSelectTypeOfFace::Sphere, 0),
        (StdSelectTypeOfFace::Torus, 0),
        (StdSelectTypeOfFace::Cone, 0),
    ] {
        filter.set_type(kind);
        assert_eq!(filter.type_(), kind);
        assert_eq!(accepted(&filter, &faces)?, count, "{kind:?}");
    }
    let box_faces = sub_shapes(&box_shape()?, ShapeEnum::Face)?;
    filter.set_type(StdSelectTypeOfFace::Plane);
    assert_eq!(accepted(&filter, &box_faces)?, 6);
    assert!(filter.acts_on(ShapeEnum::Face));
    assert!(!filter.acts_on(ShapeEnum::Edge));
    Ok(())
}

#[test]
fn filters_refuse_owners_that_are_not_brep_owners() -> Result<(), Failure> {
    let plain: Handle<dyn SelectMgrEntityOwnerSource> = SelectMgrEntityOwner::new(0).into_cast();
    assert!(!StdSelectEdgeFilter::new(StdSelectTypeOfEdge::AnyEdge).is_ok(&plain)?);
    assert!(!StdSelectFaceFilter::new(StdSelectTypeOfFace::AnyFace).is_ok(&plain)?);
    assert!(!StdSelectShapeTypeFilter::new(ShapeEnum::Face).is_ok(&plain)?);
    // A null shape is refused by the shape type filter (HasShape) and fails in the others.
    let null = owner(&Shape::default());
    assert!(!StdSelectShapeTypeFilter::new(ShapeEnum::Face).is_ok(&null)?);
    assert!(StdSelectFaceFilter::new(StdSelectTypeOfFace::AnyFace).is_ok(&null).is_err());
    Ok(())
}

#[test]
fn shape_type_filter_matches_the_shape_type() -> Result<(), Failure> {
    let solid = box_shape()?;
    let filter = StdSelectShapeTypeFilter::new(ShapeEnum::Vertex);
    assert_eq!(filter.type_(), ShapeEnum::Vertex);
    assert_eq!(accepted(&filter, &sub_shapes(&solid, ShapeEnum::Vertex)?)?, 48);
    assert_eq!(accepted(&filter, &sub_shapes(&solid, ShapeEnum::Edge)?)?, 0);
    assert!(!filter.is_ok(&owner(&solid))?);
    assert!(filter.acts_on(ShapeEnum::Vertex));
    assert!(!filter.acts_on(ShapeEnum::Edge));
    Ok(())
}
