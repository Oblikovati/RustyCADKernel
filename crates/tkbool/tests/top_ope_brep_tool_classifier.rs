//! TopOpeBRepTool classifiers: points in solids and shells, shapes against solids and
//! faces, the per-thread FSC classifier and the geometric tangent of edges.
use rustycad_tkbool::top_ope_brep_tool::{
    TopOpeBRepToolShapeClassifier, TopOpeBRepToolSolidClassifier, fsc_get_psc, fsc_state_eonface,
    tggeom_e,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The box [0, 2]^3.
fn cube() -> Result<Shape, Failure> {
    Ok(BRepPrimAPIMakeBox::from_sizes(2., 2., 2.)?.shape()?.clone())
}

/// The first sub-shape of a type.
fn first(s: &Shape, t: ShapeEnum) -> Result<Shape, Failure> {
    Explorer::new(s, t, ShapeEnum::Shape)?
        .take(1)
        .collect::<Vec<Shape>>()
        .pop()
        .ok_or_else(|| Failure::new("no sub-shape"))
}

/// The segment between two points.
fn segment(a: Point3, b: Point3) -> Result<Shape, Failure> {
    Ok(BRepLibMakeEdge::from_points(a, b)?.edge()?.into_shape())
}

#[test]
fn points_are_classified_in_the_solid_and_its_shell() -> Result<(), Failure> {
    let solid = topods::solid(&cube()?)?;
    let shell = topods::shell(&first(solid.as_ref(), ShapeEnum::Shell)?)?;
    let mut sc = TopOpeBRepToolSolidClassifier::new();
    assert_eq!(
        sc.classify_solid(&solid, Point3::new(1., 1., 1.), 1e-7)?,
        State::In
    );
    assert_eq!(
        sc.classify_solid(&solid, Point3::new(3., 1., 1.), 1e-7)?,
        State::Out
    );
    assert_eq!(
        sc.classify_shell(&shell, Point3::new(1., 1., 0.), 1e-7)?,
        State::On
    );
    assert_eq!(sc.state(), State::On);
    sc.clear();
    assert_eq!(sc.state(), State::Unknown);
    Ok(())
}

#[test]
fn edges_are_classified_by_a_point_of_them() -> Result<(), Failure> {
    let solid = cube()?;
    let inner = segment(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1., 1.2))?;
    let above = segment(Point3::new(0.5, 0.5, 3.), Point3::new(1.5, 1., 3.))?;
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    assert_eq!(sc.state_shape_shape(&inner, &solid, 0)?, State::In);
    assert!(sc.p3d().is_ok());
    assert_eq!(sc.state_shape_shape(&above, &solid, 0)?, State::Out);
    sc.clear_current();
    assert_eq!(sc.same_domain(), -1);
    assert_eq!(
        sc.p3d()
            .err()
            .map(|e| e.exception_type().to_string())
            .as_deref(),
        Some("Standard_ProgramError")
    );
    Ok(())
}

#[test]
fn an_edge_of_a_face_is_on_it() -> Result<(), Failure> {
    let solid = cube()?;
    let face = first(&solid, ShapeEnum::Face)?;
    let edge = first(&face, ShapeEnum::Edge)?;
    let state = fsc_get_psc(|psc| fsc_state_eonface(&edge, 0.5, &face, psc))?;
    assert_eq!(state, State::On);
    Ok(())
}

#[test]
fn tangents_exist_on_the_edge_range_only() -> Result<(), Failure> {
    let e = topods::edge(&segment(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.))?)?;
    let tg = tggeom_e(1., &e)?.ok_or_else(|| Failure::new("no tangent"))?;
    assert_eq!((tg.x(), tg.y(), tg.z()), (1., 0., 0.));
    assert!(tggeom_e(2., &e)?.is_some());
    assert!(tggeom_e(3., &e)?.is_none());
    Ok(())
}
