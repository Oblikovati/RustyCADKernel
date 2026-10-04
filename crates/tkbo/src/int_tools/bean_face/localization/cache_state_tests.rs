//! Recursive localization state checks against the private native oracle.
use crate::int_tools::SurfaceRangeSample;

#[test]
fn recursive_localization_clears_seeded_exclusion_and_skips_flagged_endpoint_spans() {
    let mut bean = super::bspline_bean();
    bean.range_manager.set_boundaries(0.0, 2.0, 2);
    let surface_cache = super::super::surface_cache(&bean).unwrap();
    surface_cache
        .write()
        .add_out_range(&SurfaceRangeSample::default());

    assert!(super::compute(&mut bean).unwrap());

    assert_native_recursive_ranges(&bean);
    let cache = surface_cache.read();
    assert!(!cache.is_range_out(&SurfaceRangeSample::default()));
    assert_eq!((cache.get_range_u_grid(), cache.get_range_v_grid()), (0, 0));
    assert_eq!(cache.get_grid_deflection(), 0.0);
    assert_short_curve_sampling_uses_global_range();
}

fn assert_native_recursive_ranges(bean: &crate::int_tools::BeanFaceIntersector) {
    let native = include_str!("../../../../../../tools/oracle/tkbo_bean_localized_cache.expected");
    let line = native
        .lines()
        .find(|line| line.starts_with("premarked "))
        .unwrap();
    let mut fields = line.split_whitespace();
    assert_eq!(fields.next().unwrap(), "premarked");
    assert_eq!(fields.next().unwrap(), "1");
    assert_eq!(fields.next().unwrap(), "0");
    assert_eq!((fields.next().unwrap(), fields.next().unwrap()), ("0", "0"));
    assert_eq!(fields.next().unwrap(), "0");
    let count = fields.next().unwrap().parse::<usize>().unwrap();
    assert_eq!(bean.range_manager.length() as usize, count);
    for index in 1..=bean.range_manager.length() {
        let expected_flag = fields.next().unwrap().parse::<i32>().unwrap();
        let expected_first = fields.next().unwrap().parse::<f64>().unwrap();
        let expected_last = fields.next().unwrap().parse::<f64>().unwrap();
        let actual = bean.range_manager.range(index).unwrap();
        assert_eq!(bean.range_manager.flag(index).unwrap(), expected_flag);
        assert!((actual.first() - expected_first).abs() <= 1e-12);
        assert!((actual.last() - expected_last).abs() <= 1e-12);
    }
    assert!(fields.next().is_none());
}

fn assert_short_curve_sampling_uses_global_range() {
    let mut bean = super::bspline_bean();
    bean.set_bean_parameters(1.0, 1.0000001);
    bean.range_manager.set_boundaries(1.0, 1.0000001, 0);
    assert!(super::compute(&mut bean).unwrap());
    assert_eq!(bean.range_manager.length(), 1);
    assert_eq!(bean.range_manager.flag(1).unwrap(), 0);
    let actual = bean.range_manager.range(1).unwrap();
    assert!((actual.first() - 1.0).abs() <= 1e-12);
    assert!((actual.last() - 1.0000001).abs() <= 1e-12);
    assert_native_short_range();
}

fn assert_native_short_range() {
    let line = include_str!("../../../../../../tools/oracle/tkbo_bean_localized_cache.expected")
        .lines()
        .find(|line| line.starts_with("short_range "))
        .unwrap();
    assert_eq!(line, "short_range 1 0 0 0 0 1 0 1 1.0000001000000001");
}
