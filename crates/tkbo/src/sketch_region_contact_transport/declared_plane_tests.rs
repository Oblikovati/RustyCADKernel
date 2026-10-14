// SPDX-License-Identifier: GPL-2.0-only
//! Actual declared support and source placement participate in the existing exact metric query.
use crate::sketch_region_certification::census_curved_pair_fixture;
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::{
    gp::{Direction3, Plane3, Point3, Transform3, Vector3},
    toploc::Location,
};

pub(super) fn translated_sources() -> Vec<Shape> {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(2., 3., 5.));
    census_curved_pair_fixture()
        .iter()
        .map(|source| source.located(&Location::new(transform), true).unwrap())
        .collect()
}

pub(super) fn translated_support() -> Plane3 {
    let origin = Point3::new(2., 3., 5.);
    Plane3::new(origin, Direction3::new(0., 0., 1.).unwrap()).unwrap()
}

#[test]
fn exact_translated_support_does_not_require_world_origin_plane() {
    let edges = translated_sources();
    let sources = binding_sources(&edges);
    let plane = translated_support();
    let census = super::test_fixture::source_census_on_plane(&sources, plane);
    let mapped = super::test_fixture::mapped_outputs_on_plane(&sources, plane);
    let retained = super::test_fixture::finish_fixture_request(census, mapped, &sources);
    assert_eq!(super::test_fixture::bounded_contacts(&retained).len(), 1);
    assert!(
        super::test_fixture::world_measurements(&retained)
            .into_iter()
            .all(|measurement| measurement.point[2] == 5.)
    );
    super::test_fixture::assert_private_incomplete(&retained);
}
