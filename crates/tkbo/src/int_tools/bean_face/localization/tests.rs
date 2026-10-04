use super::compute;
use crate::int_tools::{BeanFaceIntersector, Context, SurfaceRangeSample};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::Handle,
};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[path = "cache_state_tests.rs"]
mod cache_state_tests;
#[path = "periodic_origin_tests.rs"]
mod periodic_origin_tests;
#[path = "public_perform_tests.rs"]
mod public_perform_tests;
#[path = "surface_family_tests.rs"]
mod surface_family_tests;

#[test]
fn localized_flat_bezier_matches_native_recursive_ranges() {
    let mut bean = bezier_bean();
    bean.range_manager.set_boundaries(0.0, 2.0, 0);

    assert!(compute(&mut bean).unwrap());

    assert_native_ranges(&bean, "flat_bezier");
}

#[test]
fn localized_cubic_bspline_matches_native_recursive_ranges() {
    let mut bean = bspline_bean();
    bean.range_manager.set_boundaries(0.0, 2.0, 0);

    assert!(compute(&mut bean).unwrap());

    assert_native_ranges(&bean, "cubic_bspline");
}

#[test]
fn localized_repeat_reuses_surface_box_cache_and_releases_grid() {
    let mut bean = bspline_bean();
    bean.range_manager.set_boundaries(0.0, 2.0, 0);
    assert!(compute(&mut bean).unwrap());
    let first = range_snapshot(&bean);
    let first_box = cached_root_surface_box(&bean);
    assert_grid_is_clear(&bean);

    bean.range_manager.set_boundaries(0.0, 2.0, 0);
    seed_obsolete_surface_exclusion(&bean);
    assert!(compute(&mut bean).unwrap());

    assert_eq!(range_snapshot(&bean), first);
    assert_same_box(&first_box, &cached_root_surface_box(&bean));
    assert_exclusion_was_cleared(&bean);
    assert_grid_is_clear(&bean);
}

fn seed_obsolete_surface_exclusion(bean: &BeanFaceIntersector) {
    let cache = super::surface_cache(bean).unwrap();
    cache.write().add_out_range(&SurfaceRangeSample::default());
}

fn assert_exclusion_was_cleared(bean: &BeanFaceIntersector) {
    let cache = super::surface_cache(bean).unwrap();
    assert!(!cache.read().is_range_out(&SurfaceRangeSample::default()));
}

fn cached_root_surface_box(bean: &BeanFaceIntersector) -> rustycad_tkmath::bnd::BoundingBox3 {
    let cache = super::surface_cache(bean).unwrap();
    let mut output = rustycad_tkmath::bnd::BoundingBox3::default();
    assert!(
        cache
            .read()
            .find_box(&SurfaceRangeSample::default(), &mut output)
    );
    output
}

fn assert_same_box(
    expected: &rustycad_tkmath::bnd::BoundingBox3,
    actual: &rustycad_tkmath::bnd::BoundingBox3,
) {
    assert_eq!(expected.get().unwrap(), actual.get().unwrap());
}

fn range_snapshot(bean: &BeanFaceIntersector) -> Vec<(i32, f64, f64)> {
    (1..=bean.range_manager.length())
        .map(|index| {
            let range = bean.range_manager.range(index).unwrap();
            (
                bean.range_manager.flag(index).unwrap(),
                range.first(),
                range.last(),
            )
        })
        .collect()
}

fn assert_grid_is_clear(bean: &BeanFaceIntersector) {
    let cache = super::surface_cache(bean).unwrap();
    let data = cache.read();
    assert_eq!((data.get_range_u_grid(), data.get_range_v_grid()), (0, 0));
    assert_eq!(data.get_grid_deflection(), 0.0);
}

fn assert_native_ranges(bean: &BeanFaceIntersector, name: &str) {
    let expected = native_ranges(name);
    assert_eq!(bean.range_manager.length() as usize, expected.len());
    for (offset, (flag, first, last)) in expected.into_iter().enumerate() {
        let index = offset as i32 + 1;
        let actual = bean.range_manager.range(index).unwrap();
        assert_eq!(bean.range_manager.flag(index).unwrap(), flag);
        assert!(
            (actual.first() - first).abs() < 1e-12,
            "range {index} first"
        );
        assert!((actual.last() - last).abs() < 1e-12, "range {index} last");
    }
}

fn native_ranges(name: &str) -> Vec<(i32, f64, f64)> {
    let fixture = include_str!("../../../../../../tools/oracle/tkbo_bean_localized.expected");
    let mut selected = false;
    fixture
        .lines()
        .filter_map(|line| {
            if line.starts_with(name) {
                selected = true;
                return None;
            }
            if selected && line.contains(" localized=") {
                selected = false;
            }
            if !selected || !line.starts_with("range ") {
                return None;
            }
            let mut fields = line.split_whitespace().skip(2);
            Some((
                fields.next()?.parse().ok()?,
                fields.next()?.parse().ok()?,
                fields.next()?.parse().ok()?,
            ))
        })
        .collect()
}

fn bezier_bean() -> BeanFaceIntersector {
    let edge = vertical_edge();
    let face = flat_bezier_face();
    let curve = BRepAdaptorCurve::with_edge(&edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(&face, true).unwrap();
    let mut bean = BeanFaceIntersector::from_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap();
    bean.set_bean_parameters(0.0, 2.0);
    bean.set_surface_parameters(0.0, 1.0, 0.0, 1.0);
    bean.set_context(&Handle::new(Context::default()));
    bean
}

fn vertical_edge() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(0.5, 0.5, -1.0), Point3::new(0.5, 0.5, 1.0))
        .unwrap()
        .edge()
        .unwrap()
}

fn flat_bezier_face() -> Face {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    poles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    poles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    poles[(2, 2)] = Point3::new(1.0, 1.0, 0.0);
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0.0, 1.0, 0.0, 1.0), 1e-7)
        .unwrap()
        .face()
        .unwrap()
}

fn bspline_bean() -> BeanFaceIntersector {
    let edge = vertical_edge();
    let face = cubic_bspline_face();
    let curve = BRepAdaptorCurve::with_edge(&edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(&face, true).unwrap();
    let mut bean = BeanFaceIntersector::from_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap();
    bean.set_bean_parameters(0.0, 2.0);
    bean.set_surface_parameters(0.0, 1.0, 0.0, 1.0);
    bean.set_context(&Handle::new(Context::default()));
    bean
}

fn cubic_bspline_face() -> Face {
    let mut poles = Array2::with_bounds(1, 6, 1, 6).unwrap();
    for u in 1..=6 {
        for v in 1..=6 {
            let x = f64::from(u - 1) / 5.0;
            let y = f64::from(v - 1) / 5.0;
            poles[(u, v)] = Point3::new(x, y, 0.08 * x * y);
        }
    }
    let (knots, mults) = cubic_knots();
    let surface = GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&knots, &mults),
        (3, 3),
        (false, false),
    )
    .unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(surface).cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0.0, 1.0, 0.0, 1.0), 1e-7)
        .unwrap()
        .face()
        .unwrap()
}

fn cubic_knots() -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut knots = Array1::with_bounds(1, 3).unwrap();
    let mut mults = Array1::with_bounds(1, 3).unwrap();
    for (index, value) in [0.0, 0.5, 1.0].into_iter().enumerate() {
        knots[(index as i32) + 1] = value;
    }
    for (index, value) in [4, 2, 4].into_iter().enumerate() {
        mults[(index as i32) + 1] = value;
    }
    (knots, mults)
}
