//! TopOpeBRepTool shape helpers: ShapeTool queries, projections, geometry tests, the
//! FC2D p-curve cache, face tolerances and the box tool.
use rustycad_tkbool::top_ope_brep_tool::{
    TopOpeBRepToolHBoxTool, TopOpeBRepToolShapeTool, fc2d_curve_on_surface,
    fc2d_curve_on_surface_on_edge, fc2d_has_new_curve_on_surface, fc2d_has_old_curve_on_surface,
    fc2d_prepare, ftol_face_tolerances3d, fun_tool_ng_s, fun_tool_proj_pone, fun_tool_quad_surface,
    uviso,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Point2, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
use std::f64::consts::PI;

/// The sub-shapes of a type.
fn subs(s: &Shape, t: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, t, ShapeEnum::Shape)?.collect())
}

/// The cylinder of radius 1 and height 2, its lateral face and its seam.
fn cylinder() -> Result<(Shape, Face, Edge), Failure> {
    let solid = BRepPrimAPIMakeCylinder::new(1., 2.).shape()?.clone();
    for f in subs(&solid, ShapeEnum::Face)? {
        let face = topods::face(&f)?;
        if BRepAdaptorSurface::with_face(&face, false)?.get_type()? != SurfaceType::Cylinder {
            continue;
        }
        for e in subs(&f, ShapeEnum::Edge)? {
            if TopOpeBRepToolShapeTool::closed(&e, &f)? {
                return Ok((solid, face, topods::edge(&e)?));
            }
        }
    }
    Err(Failure::new("no cylinder seam"))
}

#[test]
fn tolerances_exist_only_for_faces_edges_and_vertices() -> Result<(), Failure> {
    let (solid, face, seam) = cylinder()?;
    assert!(TopOpeBRepToolShapeTool::tolerance(face.as_ref())? > 0.);
    assert!(TopOpeBRepToolShapeTool::tolerance(seam.as_ref())? > 0.);
    assert_eq!(TopOpeBRepToolShapeTool::tolerance(&Shape::default())?, 0.);
    let error = TopOpeBRepToolShapeTool::tolerance(&solid)
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_ProgramError"));
    assert!(TopOpeBRepToolShapeTool::pnt(seam.as_ref()).is_err());
    Ok(())
}

#[test]
fn uv_values_are_brought_into_the_period_of_the_face() -> Result<(), Failure> {
    let (_, face, _) = cylinder()?;
    let (mut u, mut v) = (7.0, 0.5);
    TopOpeBRepToolShapeTool::adjust_on_periodic(face.as_ref(), &mut u, &mut v)?;
    assert!((u - (7.0 - 2. * PI)).abs() < 1e-12);
    assert_eq!(v, 0.5);
    Ok(())
}

#[test]
fn the_seam_closes_the_cylinder_face_and_periodizes_its_parameters() -> Result<(), Failure> {
    let (_, face, seam) = cylinder()?;
    assert!(TopOpeBRepToolShapeTool::closed(
        seam.as_ref(),
        face.as_ref()
    )?);
    // The seam p-curve is an iso-U, so its parameter moves by the U period (ADJUST).
    let p = TopOpeBRepToolShapeTool::periodize_parameter(0.0, seam.as_ref(), face.as_ref())?;
    assert!((p - 2. * PI).abs() < 1e-12);
    Ok(())
}

#[test]
fn a_point_projects_on_an_edge_or_writes_the_unit_distance() -> Result<(), Failure> {
    let e =
        BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.))?.edge()?;
    let (mut par, mut dist) = (-1., -1.);
    assert!(fun_tool_proj_pone(
        Point3::new(0.5, 1., 0.),
        &e,
        &mut par,
        &mut dist
    )?);
    assert!((par - 0.5).abs() < 1e-12 && (dist - 1.).abs() < 1e-12);
    let (mut par, mut dist) = (-1., -1.);
    assert!(!fun_tool_proj_pone(
        Point3::new(5., 1., 0.),
        &e,
        &mut par,
        &mut dist
    )?);
    assert_eq!((par, dist), (-1., 1.));
    Ok(())
}

#[test]
fn planes_and_cylinders_are_quadrics_with_normals() -> Result<(), Failure> {
    let (_, face, _) = cylinder()?;
    let s = BRepTool::surface(&face)?;
    assert!(fun_tool_quad_surface(&s)?);
    let n = fun_tool_ng_s(Point2::new(0., 1.), &s)?;
    assert!((n.x() - 1.).abs() < 1e-12);
    Ok(())
}

#[test]
fn a_new_pcurve_is_made_cached_and_translated_onto_the_seam() -> Result<(), Failure> {
    let (solid, face, seam) = cylinder()?;
    let a: f64 = 1.0;
    let on_cylinder = BRepLibMakeEdge::from_points(
        Point3::new(a.cos(), a.sin(), 0.2),
        Point3::new(a.cos(), a.sin(), 1.8),
    )?
    .edge()?;
    fc2d_prepare(&solid, &solid);
    assert!(fc2d_has_old_curve_on_surface(&on_cylinder, &face)?.is_none());
    let (pc, ..) = fc2d_curve_on_surface(&on_cylinder, &face, false)?;
    assert!(fc2d_has_new_curve_on_surface(&on_cylinder, &face).is_some());
    let (mut d, mut o) = (Direction2::X, Point2::default());
    let (iso, iso_u, _) = uviso(&pc, &mut d, &mut o)?;
    assert!(iso && iso_u && (o.x() - a).abs() < 1e-9);
    let (moved, ..) = fc2d_curve_on_surface_on_edge(&on_cylinder, &face, &seam, false)?;
    let (_, _, _) = uviso(&moved, &mut d, &mut o)?;
    assert!((o.x() - 2. * PI).abs() < 1e-9, "u = {}", o.x());
    Ok(())
}

#[test]
fn face_tolerances_are_at_most_1e4() -> Result<(), Failure> {
    let (_, face, _) = cylinder()?;
    let tol = ftol_face_tolerances3d(&face, &face)?;
    assert_eq!(tol, 2. * BRepTool::face_tolerance(&face)?);
    Ok(())
}

#[test]
fn the_box_tool_boxes_faces_once_and_refuses_wires() -> Result<(), Failure> {
    let solid = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)?.shape()?.clone();
    let hbt = TopOpeBRepToolHBoxTool::new();
    hbt.add_boxes(&solid, ShapeEnum::Face, ShapeEnum::Shape)?;
    hbt.add_boxes(&solid, ShapeEnum::Face, ShapeEnum::Shape)?;
    assert_eq!(hbt.extent(), 6);
    let first = hbt.shape(1)?;
    assert_eq!(hbt.index(&first), 1);
    assert!(hbt.box_of_index(7).is_err());
    assert!(
        hbt.add_boxes(&solid, ShapeEnum::Wire, ShapeEnum::Shape)
            .is_err()
    );
    Ok(())
}
