//! StdPrs_ShapeTool, the restricted face isolines, the pole presentations and the HLR shapes
//! on simple faces (the native counts; tools/tkv3d_corpus_std_prs_faces.py compares the
//! complete outputs with native OCCT).
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Plane3, Point3};
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dStructure, Graphic3dStructureMembers, Graphic3dStructureSource,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::std_prs::{
    StdPrsHlrShape, StdPrsHlrShapeI, StdPrsIsolinePolylines, StdPrsPoleCurve, StdPrsShapeTool,
    StdPrsWfDeflectionRestrictedFace, StdPrsWfRestrictedFace,
};
use rustycad_tkv3d::v3d::V3dViewer;

#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display;

fn axes() -> Axis3 {
    let z = Direction3::new(0., 0., 1.).unwrap();
    Axis3::new(Point3::new(0., 0., 0.), z, Direction3::new(1., 0., 0.).unwrap()).unwrap()
}

fn plane_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::from_position(axes()), (0., 2., 0., 1.))
        .unwrap()
        .face()
        .unwrap()
}

fn drawer() -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer
}

fn presentation() -> (Handle<V3dViewer>, Handle<Graphic3dStructure>) {
    let (driver, _trace) = display::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    (viewer, prs)
}

#[test]
fn shape_tool_walks_faces_edges_and_vertices() -> Result<(), Failure> {
    let mut tool = StdPrsShapeTool::new(&Shape::from(plane_face()), true)?;
    tool.init_face()?;
    assert!(tool.more_face());
    assert!(tool.is_planar_current_face()?);
    assert!(tool.has_surface()?);
    tool.next_face()?;
    assert!(!tool.more_face());
    tool.init_curve();
    let mut edges = 0;
    while tool.more_curve() {
        assert_eq!(tool.neighbours()?, 1);
        assert!(tool.has_curve()?);
        edges += 1;
        tool.next_curve();
    }
    assert_eq!(edges, 4);
    tool.init_vertex();
    let mut vertices = 0;
    while tool.more_vertex() {
        vertices += 1;
        tool.next_vertex();
    }
    assert_eq!(vertices, 4);
    // Without all the vertices, a face has no free vertex.
    let mut tool = StdPrsShapeTool::new(&Shape::from(plane_face()), false)?;
    tool.init_vertex();
    assert!(!tool.more_vertex());
    Ok(())
}

#[test]
fn restricted_face_isolines_follow_the_iso_numbers() -> Result<(), Failure> {
    let (_viewer, prs) = presentation();
    let prs: Handle<dyn Graphic3dStructureSource> = prs.cast();
    let face = Handle::new(BRepAdaptorSurface::with_face(&plane_face(), true)?);
    let drawer = drawer();
    let mut curves = StdPrsIsolinePolylines::default();
    StdPrsWfRestrictedFace::add_isos(&prs, &face, true, true, 2, 3, &drawer, &mut curves)?;
    assert_eq!(curves.extent(), 5);
    let mut curves = StdPrsIsolinePolylines::default();
    StdPrsWfDeflectionRestrictedFace::add_isos(&prs, &face, true, false, 0.01, 4, 3, &drawer, &mut curves)?;
    assert_eq!(curves.extent(), 4);
    // The U isos of the plane are x = const lines across y in [0, 1].
    let on_iso = Point3::new(2. * (0.998 / 3. + 0.001), 0.5, 0.);
    drawer.u_iso_aspect().set_number(2);
    assert!(StdPrsWfRestrictedFace::match_u_iso(&on_iso, 0.01, &face, &drawer)?);
    assert!(!StdPrsWfRestrictedFace::match_u_iso(&Point3::new(9., 9., 9.), 0.01, &face, &drawer)?);
    Ok(())
}

#[test]
fn pole_curve_picks_the_nearest_pole() -> Result<(), Failure> {
    let mut poles = [Point3::new(0., 0., 0.), Point3::new(1., 1., 0.), Point3::new(2., 0., 0.)];
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&Array1::from_slice(&mut poles))?).into_cast();
    let adaptor = GeomAdaptorCurve::with_curve(&curve)?;
    let drawer = drawer();
    assert_eq!(StdPrsPoleCurve::pick(1.1, 0.9, 0., 0.5, &adaptor, &drawer)?, 2);
    assert_eq!(StdPrsPoleCurve::pick(5., 5., 5., 0.5, &adaptor, &drawer)?, 0);
    assert!(StdPrsPoleCurve::match_point(2., 0.1, 0., 0.2, &adaptor, &drawer)?);
    Ok(())
}

#[test]
fn exact_hlr_draws_the_seen_edges_of_a_cylinder() -> Result<(), Failure> {
    let (_viewer, prs) = presentation();
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        Cylinder3::new(axes(), 1.)?,
        (0., std::f64::consts::PI, 0., 2.),
    )?
    .face()?;
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_eye_and_center(&Point3::new(10., 0., 1.), &Point3::new(0., 0., 1.))?;
    camera.set_up(&Direction3::new(0., 0., 1.)?);
    let source: Handle<dyn Graphic3dStructureSource> = prs.clone().cast();
    StdPrsHlrShape::default().compute_hlr(&source, &Shape::from(face), &drawer(), &camera)?;
    assert_eq!(prs.groups().len(), 1);
    Ok(())
}
