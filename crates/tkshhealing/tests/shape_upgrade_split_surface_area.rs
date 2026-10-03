//! Native OCCT split-count/parameter fixtures for ShapeUpgrade_SplitSurfaceArea.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::Axis3;
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitSurface, ShapeUpgradeSplitSurfaceArea, SplitSurface,
};

#[derive(Clone, Copy)]
struct Fixture {
    name: &'static str,
    u_last: f64,
    v_last: f64,
    parts: i32,
    squares: bool,
    fixed: Option<(i32, i32)>,
    area: f64,
    repeat: bool,
    cylinder: bool,
}

const FIXTURES: [Fixture; 12] = [
    fixture(
        "default-noop",
        (4., 2.),
        (1, false, None),
        0.,
        (false, false),
    ),
    fixture(
        "square-ratio-one",
        (4., 4.),
        (2, false, None),
        0.,
        (false, false),
    ),
    fixture(
        "ratio-long-u",
        (10., 2.),
        (4, false, None),
        0.,
        (false, false),
    ),
    fixture(
        "ratio-long-v",
        (2., 10.),
        (4, false, None),
        0.,
        (false, false),
    ),
    fixture(
        "ratio-between",
        (5., 2.),
        (4, false, None),
        0.,
        (false, false),
    ),
    fixture(
        "fixed-square-u-first",
        (10., 2.),
        (6, true, Some((2, 3))),
        0.,
        (false, false),
    ),
    fixture(
        "fixed-square-v-first",
        (2., 10.),
        (6, true, Some((2, 3))),
        0.,
        (false, false),
    ),
    fixture(
        "fixed-square-equal-aspect",
        (4., 4.),
        (6, true, Some((2, 3))),
        0.,
        (false, false),
    ),
    fixture(
        "auto-square",
        (10., 2.),
        (4, true, None),
        100.,
        (false, false),
    ),
    fixture(
        "cylinder-auto-square",
        (std::f64::consts::TAU, 4.),
        (4, true, None),
        25.,
        (false, true),
    ),
    fixture(
        "invalid-fixed-count-retains-parts",
        (5., 2.),
        (4, true, Some((2, -1))),
        100.,
        (false, false),
    ),
    fixture(
        "auto-square-second-call",
        (10., 2.),
        (4, true, None),
        100.,
        (true, false),
    ),
];

const fn fixture(
    name: &'static str,
    bounds: (f64, f64),
    config: (i32, bool, Option<(i32, i32)>),
    area: f64,
    flags: (bool, bool),
) -> Fixture {
    Fixture {
        name,
        u_last: bounds.0,
        v_last: bounds.1,
        parts: config.0,
        squares: config.1,
        fixed: config.2,
        area,
        repeat: flags.0,
        cylinder: flags.1,
    }
}

fn surface(cylinder: bool) -> Handle<dyn GeomSurface> {
    if cylinder {
        return Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.).unwrap()).cast();
    }
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

fn native_rows() -> Vec<Vec<String>> {
    include_str!("../../../tools/oracle/tkshhealing_split_surface_area.expected")
        .lines()
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect()
}

fn configure(fixture: Fixture) -> Handle<dyn SplitSurface> {
    let splitter = ShapeUpgradeSplitSurfaceArea::new();
    splitter
        .init_surface_bounds(
            &surface(fixture.cylinder),
            (0., fixture.u_last, 0., fixture.v_last),
            fixture.area,
        )
        .unwrap();
    splitter.set_nb_parts(fixture.parts);
    splitter.set_splitting_into_squares(fixture.squares);
    if let Some((u_count, v_count)) = fixture.fixed {
        splitter.set_numbers_uv_splits(u_count, v_count).unwrap();
    }
    Handle::new(splitter).cast()
}

fn run_fixture(fixture: Fixture) -> (i32, Vec<f64>, Vec<f64>) {
    let criterion = configure(fixture);
    let (u_values, v_values) = (criterion.u_split_values(), criterion.v_split_values());
    criterion.compute(false).unwrap();
    if fixture.repeat {
        criterion.compute(false).unwrap();
    }
    let area = criterion.cast::<ShapeUpgradeSplitSurfaceArea>();
    assert_eq!(
        area.dynamic_type().unwrap().name(),
        "ShapeUpgrade_SplitSurfaceArea"
    );
    assert_parent_sequences(&criterion, &u_values, &v_values);
    (area.nb_parts(), snapshot(&u_values), snapshot(&v_values))
}

fn assert_parent_sequences(
    criterion: &Handle<dyn SplitSurface>,
    u_values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>,
    v_values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>,
) {
    let parent = criterion.cast::<ShapeUpgradeSplitSurface>();
    assert!(parent.is_kind_name("ShapeUpgrade_SplitSurface"));
    assert_eq!(
        parent.u_split_values().object_address(),
        u_values.object_address()
    );
    assert_eq!(
        parent.v_split_values().object_address(),
        v_values.object_address()
    );
}

fn snapshot(values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>) -> Vec<f64> {
    values
        .get()
        .unwrap()
        .sequence(|items| items.iter().copied().collect())
}

fn expected_sequence(row: &[String], split: char) -> Vec<f64> {
    let marker = row.iter().position(|value| value == "|").unwrap();
    let range = if split == 'u' {
        4..marker
    } else {
        marker + 1..row.len()
    };
    row[range]
        .iter()
        .map(|value| value.parse().unwrap())
        .collect()
}

fn assert_fixture_matches(fixture: Fixture, oracle: &[Vec<String>]) {
    let row = oracle.iter().find(|row| row[0] == fixture.name).unwrap();
    let (parts, u_values, v_values) = run_fixture(fixture);
    assert_part_count(parts, row, fixture.name);
    assert_sequences((u_values, v_values), row, fixture.name);
}

fn assert_part_count(parts: i32, row: &[String], name: &str) {
    assert_eq!(parts, row[1].parse().unwrap(), "{name} part count");
}

fn assert_sequences(actual: (Vec<f64>, Vec<f64>), row: &[String], name: &str) {
    assert_eq!(actual.0, expected_sequence(row, 'u'), "{name} U values");
    assert_eq!(actual.1, expected_sequence(row, 'v'), "{name} V values");
}

#[test]
fn area_split_parameter_sequences_match_pinned_native_oracle() {
    let oracle = native_rows();
    for fixture in FIXTURES {
        assert_fixture_matches(fixture, &oracle);
    }
}

#[test]
fn shared_parent_and_virtual_criterion_keep_dynamic_area_type() {
    let splitter = Handle::new(ShapeUpgradeSplitSurfaceArea::new());
    splitter.clone().set_nb_parts(3);
    assert_eq!(splitter.nb_parts(), 3);
    assert_eq!(
        splitter.dynamic_type().unwrap().name(),
        "ShapeUpgrade_SplitSurfaceArea"
    );
    assert!(splitter.is_kind_name("ShapeUpgrade_SplitSurface"));
    assert_eq!(
        splitter
            .cast::<dyn SplitSurface>()
            .dynamic_type()
            .unwrap()
            .name(),
        "ShapeUpgrade_SplitSurfaceArea"
    );
}

#[test]
fn positive_fixed_count_product_overflow_is_reported_outside_native_defined_domain() {
    let splitter = ShapeUpgradeSplitSurfaceArea::new();
    let failure = splitter.set_numbers_uv_splits(i32::MAX, 2).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(splitter.nb_parts(), 1);
}

#[test]
fn automatic_square_counts_require_finite_native_measure_quotients() {
    let splitter = ShapeUpgradeSplitSurfaceArea::new();
    splitter
        .init_surface_bounds(&surface(false), (0., 1., 0., 1.), 0.)
        .unwrap();
    splitter.set_nb_parts(4);
    splitter.set_splitting_into_squares(true);
    let failure = splitter.compute(false).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("U square count"));
}
