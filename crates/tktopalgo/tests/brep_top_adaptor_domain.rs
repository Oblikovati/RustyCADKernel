//! Behavior tests of BRepTopAdaptor_TopolTool as the domain of IntStart and IntPatch
//! (docs/topalgo-design.md section 8): the vertices it hands out dispatch the
//! Adaptor3d_HVertex members to the BRepTopAdaptor_HVertex overrides, the inherited
//! sample members run on the overriding counts, and IntPatch_Intersection bounds its
//! lines by the edges of BRep faces. The numeric probes (bta_sob, bta_inside,
//! bta_intersection) compare them bit for bit with OCCT.
mod adaptor_records;
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use adaptor_records::{BSPLINE, HOLED};
use face::tp_face;
use print::Reader;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::real::real_first;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchGLine, IntPatchIntersection, IntPatchIntersectionOptions,
};
use rustycad_tkgeomalgo::int_start::{TheTopolTool, TheVertex};
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_top_adaptor::{BRepTopAdaptorHVertex, BRepTopAdaptorTopolTool};

// The adaptor of a face record and its BRep domain.
fn domain(record: &[f64]) -> (Handle<dyn Adaptor3dSurface>, BRepTopAdaptorTopolTool) {
    let f = tp_face(&mut Reader::new(record)).expect("face record");
    let surface = BRepAdaptorSurface::with_face(&f.face, true).unwrap();
    let handle: Handle<dyn Adaptor3dSurface> = Handle::new(surface).cast();
    let tool = BRepTopAdaptorTopolTool::with_surface(&handle).unwrap();
    (handle, tool)
}

// The two vertices of the first arc of a domain, through the trait.
fn first_arc_vertices(d: &dyn TheTopolTool) -> (TheVertex, TheVertex) {
    d.init();
    d.initialize_arc(&d.value().unwrap()).unwrap();
    d.init_vertex_iterator().unwrap();
    let first = d.vertex().unwrap();
    d.next_vertex().unwrap();
    (first, d.vertex().unwrap())
}

#[test]
fn domain_vertices_dispatch_to_the_brep_overrides() {
    let (_, tool) = domain(&HOLED);
    let d: &dyn TheTopolTool = &tool;
    let (first, last) = first_arc_vertices(d);
    let arc = d.value().unwrap();
    // The base members would answer the default point (0, 0) and resolution 0.
    assert_eq!(first.value(), Point2::new(real_first(), real_first()));
    assert_eq!(first.parameter(&arc).unwrap(), 0.0);
    assert_eq!(last.parameter(&arc).unwrap(), 12.0);
    assert!(first.resolution(&arc).unwrap() > 0.0);
    assert_eq!(last.orientation(), Orientation::Reversed);
    assert!(d.identical(&first, &first).unwrap());
    assert!(!d.identical(&first, &last).unwrap());
    assert_eq!(d.vertex_orientation(&first).unwrap(), Orientation::Forward);
    assert_eq!(d.tol_3d_vertex(&first).unwrap(), 1e-7);
    assert!(d.edge().unwrap().is_some());
    assert_eq!(d.arc_orientation(&arc).unwrap(), Orientation::Forward);
    assert!(d.has_3d());
}

#[test]
fn domain_samples_with_the_overriding_counts() {
    let (_, tool) = domain(&HOLED);
    let d: &dyn TheTopolTool = &tool;
    let error = d.u_parameters().err().unwrap();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    d.sample_pnts(0.01, 4, 4).unwrap();
    // The BRepTopAdaptor count of a plane is 10 in each direction (the base one is 6).
    let (u, v) = (d.u_parameters().unwrap(), d.v_parameters().unwrap());
    assert_eq!((u.length(), v.length()), (10, 10));
    assert_eq!(
        (d.nb_samples_u().unwrap(), d.nb_samples_v().unwrap()),
        (10, 10)
    );
    assert!(d.is_uniform_sampling().unwrap());
    assert!(!d.domain_is_infinite().unwrap());
    let (_, spline) = domain(&BSPLINE);
    let s: &dyn TheTopolTool = &spline;
    assert!(!s.is_uniform_sampling().unwrap());
    s.sample_pnts(0.01, 4, 4).unwrap();
    let count: i32 = s.u_parameters().unwrap().length();
    assert_eq!(s.nb_samples_u().unwrap(), count);
    // BSplSamplePnts, inherited too, gives the same parameters on a B-spline surface.
    let (_, other) = domain(&BSPLINE);
    other.bspl_sample_pnts(0.01, 4, 4).unwrap();
    assert_eq!(other.u_parameters().unwrap().length(), count);
    assert_eq!(other.nb_samples_v().unwrap(), s.nb_samples_v().unwrap());
}

#[test]
fn intersection_bounds_its_lines_by_the_brep_vertices() {
    let (s1, d1) = domain(&HOLED);
    // The same face on the vertical plane x = y through (8, 8, 4): its UV rectangle
    // spans the diagonal of the first face at v = 4.
    let mut record = HOLED;
    record[1..7].copy_from_slice(&[8.0, 8.0, 4.0, 1.0, -1.0, 0.0]);
    let (s2, d2) = domain(&record);
    let mut inter = IntPatchIntersection::new();
    let options = IntPatchIntersectionOptions::default();
    inter
        .perform(&s1, &d1, &s2, &d2, (1e-7, 1e-7), options)
        .unwrap();
    assert!(inter.nb_lines().unwrap() > 0);
    let mut on_brep_vertex = 0;
    for i in 1..=inter.nb_lines().unwrap() {
        let line: Handle<IntPatchGLine> = inter.line(i).unwrap().cast();
        let line = line.get().expect("a plane-plane line is a GLine");
        for k in 1..=line.nb_vertex() {
            let point = line.vertex(k).unwrap();
            if !point.is_vertex_on_s1() {
                continue;
            }
            let vertex: Handle<BRepTopAdaptorHVertex> = point.vertex_on_s1().unwrap().cast();
            assert!(vertex.is_some());
            on_brep_vertex += 1;
        }
    }
    // The corner (0, 0, 0) of the first face is on the diagonal.
    assert!(on_brep_vertex > 0);
}
