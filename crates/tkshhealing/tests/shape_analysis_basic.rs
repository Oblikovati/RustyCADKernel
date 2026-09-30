//! Behavior tests of ShapeAnalysis_Geom, ShapeAnalysis_ShapeTolerance, ShapeAnalysis_Shell,
//! ShapeAnalysis_WireOrder and ShapeAnalysis_FreeBoundData.
mod healing_support;

use healing_support::{compound, edges_of, polygon, shell, square_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{Array1, HArray2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Xy, Xyz};
use rustycad_tkshhealing::shape_analysis::{
    ShapeAnalysisFreeBoundData, ShapeAnalysisGeom, ShapeAnalysisShapeTolerance, ShapeAnalysisShell,
    ShapeAnalysisWireOrder,
};

fn points(list: &[(f64, f64, f64)]) -> Array1<'static, Point3> {
    let mut array = Array1::with_bounds(1, list.len() as i32).unwrap();
    for (i, &(x, y, z)) in list.iter().enumerate() {
        array[i as i32 + 1] = Point3::new(x, y, z);
    }
    array
}

#[test]
fn geom_nearest_plane_fits_flat_clouds_and_refuses_cubes() {
    let flat = points(&[(0., 0., 0.), (2., 0., 0.1), (2., 1., -0.1), (0., 1., 0.)]);
    let (plane, dmax) = ShapeAnalysisGeom::nearest_plane(&flat).unwrap().unwrap();
    assert!(dmax > 0.0 && dmax < 0.1);
    assert!(plane.axis().direction().z().abs() > 0.99);
    let corners: Vec<(f64, f64, f64)> = (0..8)
        .map(|k| {
            (
                f64::from(k & 1),
                f64::from((k >> 1) & 1),
                f64::from((k >> 2) & 1),
            )
        })
        .collect();
    assert!(
        ShapeAnalysisGeom::nearest_plane(&points(&corners))
            .unwrap()
            .is_none()
    );
}

fn matrix(rows: [[f64; 4]; 3]) -> Handle<HArray2<f64>> {
    let m = HArray2::with_bounds(1, 3, 1, 4).unwrap();
    for (i, row) in rows.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            *m.change_array2()
                .change_value(i as i32 + 1, j as i32 + 1)
                .unwrap() = *v;
        }
    }
    Handle::new(m)
}

#[test]
fn geom_position_trsf_reads_similarities_only() {
    let (ok, trsf) = ShapeAnalysisGeom::position_trsf(
        &matrix([[1., 0., 0., 2.], [0., 1., 0., 3.], [0., 0., 1., 4.]]),
        10.0,
        1e-6,
    )
    .unwrap();
    assert!(ok);
    assert_eq!(trsf.value(1, 4).unwrap(), 20.0);
    let skew = matrix([[1., 0.5, 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.]]);
    assert!(
        !ShapeAnalysisGeom::position_trsf(&skew, 1.0, 1e-6)
            .unwrap()
            .0
    );
    assert!(
        ShapeAnalysisGeom::position_trsf(&Handle::default(), 1.0, 1e-6)
            .unwrap()
            .0
    );
}

#[test]
fn shape_tolerance_cumulates_min_average_max() {
    let face = square_face(0.0);
    let edge = edges_of(&face)[0].clone();
    BRepBuilder::default()
        .update_edge_tolerance(&edge, 1e-3)
        .unwrap();
    let mut tool = ShapeAnalysisShapeTolerance::new();
    assert_eq!(tool.tolerance(&face, 1, ShapeEnum::Edge).unwrap(), 1e-3);
    assert_eq!(tool.tolerance(&face, -1, ShapeEnum::Face).unwrap(), 1e-7);
    let over = tool.over_tolerance(&face, 1e-4, ShapeEnum::Edge).unwrap();
    assert_eq!(over.sequence(|s| s.length()), 1);
    tool.init_tolerance();
    assert_eq!(tool.global_tolerance(0), 0.0);
}

#[test]
fn shell_reports_free_connected_and_bad_edges() {
    let (f0, f1) = (square_face(0.0), square_face(0.0));
    let mut tool = ShapeAnalysisShell::new();
    let single: Shape = shell(&[f0.clone().into_shape()]).into_shape();
    assert!(!tool.check_oriented_shells(&single, true, false).unwrap());
    assert!(tool.has_free_edges() && !tool.has_connected_edges());
    let twice: Shape = shell(&[f0.clone().into_shape(), f0.into_shape()]).into_shape();
    let mut tool = ShapeAnalysisShell::new();
    assert!(tool.check_oriented_shells(&twice, false, false).unwrap());
    assert!(tool.has_bad_edges() && tool.is_loaded(&twice));
    assert!(tool.loaded(2).is_err());
    let both = compound(&[twice, shell(&[f1.into_shape()]).into_shape()]).into_shape();
    tool.clear();
    tool.load_shells(&both).unwrap();
    assert_eq!(tool.nb_loaded(), 2);
}

#[test]
fn wire_order_reorders_and_reverses_edges() {
    let mut order = ShapeAnalysisWireOrder::new();
    let p = |x: f64, y: f64| Xyz::new(x, y, 0.0);
    order.add_3d(p(1., 0.), p(1., 1.));
    order.add_3d(p(0., 0.), p(1., 0.));
    order.add_3d(p(0., 1.), p(1., 1.));
    order.perform(true).unwrap();
    assert!(order.is_done());
    let mut ranks: Vec<i32> = (1..=3).map(|i| order.ordered(i).unwrap().abs()).collect();
    ranks.sort();
    assert_eq!(ranks, vec![1, 2, 3]);
    assert_ne!(order.status(), 0);
    assert!((1..=3).any(|i| order.ordered(i).unwrap() < 0));
    order.set_chains(1e-7).unwrap();
    assert!(order.nb_chains() >= 1);
    assert_eq!(order.nb_couples(), 0);
    let mut flat = ShapeAnalysisWireOrder::with_mode(false, 1e-7, false);
    flat.add_3d(p(0., 0.), p(1., 0.));
    flat.add_2d(Xy::new(0., 0.), Xy::new(1., 0.));
    assert_eq!(flat.nb_edges(), 1);
    assert!(flat.xyz(0).is_err());
}

#[test]
fn free_bound_data_keeps_one_width_per_notch() {
    let w0 = polygon(&[(0., 0., 0.), (1., 0., 0.), (1., 1., 0.)]);
    let w1 = polygon(&[(0., 0., 1.), (1., 0., 1.), (1., 1., 1.)]);
    let data = ShapeAnalysisFreeBoundData::with_bound(&w0);
    data.add_notch(&w1, 0.5);
    data.add_notch(&w1, 0.7);
    assert_eq!(data.nb_notches(), 1);
    assert_eq!(data.notch_width(1).unwrap(), 0.5);
    assert!(data.notch_width_of(&w0).is_err());
    let kept = data.notches();
    data.clear();
    assert_eq!(kept.sequence(|s| s.length()), 0);
    assert!(data.free_bound().is_same(&w0));
}
