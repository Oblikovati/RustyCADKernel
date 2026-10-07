//! HLRBRep_Surface, HLRBRep_SurfaceTool, HLRBRep_LineTool and HLRBRep_InterCSurf.
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomSurface};
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_brep::{InterCSurf, LineTool, Surface, SurfaceTool};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Direction3, Line3, Point3, Sphere3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::sync::Arc;

fn hlr_surface(face: &Face) -> Surface {
    let mut s = Surface::new();
    s.set_projector(Some(Arc::new(Projector::new().unwrap())));
    s.set_surface(face).unwrap();
    s
}

fn sphere_face(radius: f64) -> Face {
    let sphere = Sphere3::new(Axis3::default(), radius).unwrap();
    BRepBuilderAPIMakeFace::from_sphere(sphere)
        .unwrap()
        .face()
        .unwrap()
}

#[test]
fn a_bilinear_bezier_is_a_plane_through_its_middle() {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles.set_value(1, 1, Point3::new(0., 0., 1.)).unwrap();
    poles.set_value(1, 2, Point3::new(0., 2., 1.)).unwrap();
    poles.set_value(2, 1, Point3::new(2., 0., 1.)).unwrap();
    poles.set_value(2, 2, Point3::new(2., 2., 1.)).unwrap();
    let bezier: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    let face = BRepBuilderAPIMakeFace::from_surface_bounds(&bezier, (0., 1., 0., 1.), 1e-7)
        .unwrap()
        .face()
        .unwrap();
    let s = hlr_surface(&face);
    assert_eq!(s.get_type(), SurfaceType::Plane);
    let plane = s.plane().unwrap();
    assert_eq!(plane.location(), Point3::new(1., 1., 1.));
    // Seen from the top, the face is not edge-on.
    assert!(!s.is_side(1e-7, 1e-7).unwrap());
}

#[test]
fn sample_counts_follow_the_hlr_rules() {
    let s = hlr_surface(&sphere_face(2.));
    assert_eq!(SurfaceTool::nb_samples_u(&s).unwrap(), 10);
    assert_eq!(SurfaceTool::nb_samples_v(&s).unwrap(), 15);
    // Above 10 samples the reversed span ratio truncates to -1, so 5 samples remain.
    let (v1, v2) = (
        s.first_v_parameter().unwrap(),
        s.last_v_parameter().unwrap(),
    );
    assert_eq!(SurfaceTool::nb_samples_v_in(&s, v1, v2).unwrap(), 5);
}

#[test]
fn a_sight_line_through_a_sphere_meets_it_twice() {
    let s = hlr_surface(&sphere_face(2.));
    let sight = Line3::new(
        Point3::new(0.5, 0.5, 0.),
        Direction3::new(0., 0., -1.).unwrap(),
    );
    let mut inter = InterCSurf::new();
    inter.perform(&sight, &&s).unwrap();
    assert!(inter.is_done());
    assert_eq!(inter.nb_points().unwrap(), 2);
}

#[test]
fn the_line_tool_samples_the_ends_and_their_mean() {
    let line = Line3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let pars = LineTool::sample_pars(&line, 1., 3., 0.1, 7);
    let pars = pars.array1();
    assert_eq!((pars[1], pars[2], pars[3]), (1., 2., 3.));
    assert_eq!(LineTool::nb_samples(&line, 0., 1.), 3);
    assert_eq!(LineTool::first_parameter(&line), f64::MIN);
}
