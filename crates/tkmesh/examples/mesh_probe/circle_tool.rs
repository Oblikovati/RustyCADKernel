use super::super::{flag, int, real};
use super::vertex_inspector::report_unit;
use rustycad_tkernel::{
    ncollection::{CellAction, List},
    standard::Handle,
};
use rustycad_tkmath::gp::{Circle2, Xy};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshCircle as Circle, BRepMeshCircleInspector as Inspector, BRepMeshCircleTool as Tool,
};
#[path = "../../tests/support/circle_random.rs"]
mod random;
fn scalar(index: i32) -> f64 {
    [-1., -0., 0., 1e-9, 0.1, 1., 2., f64::NAN, f64::INFINITY][index as usize]
}
fn point(index: i32) -> Xy {
    let x = scalar(index);
    Xy::new(x, if index % 2 == 0 { x } else { 0. })
}
fn report_circle(circle: &Circle) {
    real(circle.location().x());
    real(circle.location().y());
    real(circle.radius());
}
fn report_hits(hits: &List<i32>) {
    int(hits.extent());
    for index in hits.iter() {
        int(*index);
    }
}
pub fn inspector_case(v: &[i32]) {
    let allocator = Handle::new(Default::default());
    let mut inspector = Inspector::new(scalar(v[0]), 3, &allocator);
    let mut circle = Circle::default();
    report_circle(&circle);
    circle.set_location(point(v[1]));
    circle.set_radius(scalar(v[2]));
    report_circle(&circle);
    report_unit(inspector.bind(v[3], circle));
    int(inspector.circles().length());
    inspect_and_report(&mut inspector, v);
    println!();
}
fn inspect_and_report(inspector: &mut Inspector, v: &[i32]) {
    inspector.set_point(point(v[4]));
    for index in 0..inspector.circles().length() {
        match inspector.inspect(index) {
            Ok(action) => int(if action == CellAction::Keep { 0 } else { 1 }),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
    report_hits(inspector.get_shot_circles());
    inspector_mutations(inspector, v);
}
fn inspector_mutations(inspector: &mut Inspector, v: &[i32]) {
    inspector.get_shot_circles().append(99);
    report_hits(inspector.get_shot_circles());
    inspector.set_point(Xy::default());
    report_hits(inspector.get_shot_circles());
    if v[3] >= 0 {
        inspector.circle(v[3]).unwrap().set_radius(2.);
        report_circle(inspector.circle(v[3]).unwrap());
    }
    inspector_static(v);
}
fn inspector_static(v: &[i32]) {
    flag(Inspector::is_equal(v[3], 0));
    let shifted = Inspector::shift(&point(v[1]), scalar(v[0]));
    real(shifted.x());
    real(shifted.y());
    for axis in [-1, 0, 1, 2] {
        match Inspector::coord(axis, &point(v[1])) {
            Ok(value) => real(value),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
pub fn tool_case(v: &[i32]) {
    let allocator = Handle::new(Default::default());
    let mut tool = if v[0] == 0 {
        Tool::new(&allocator)
    } else {
        Tool::with_reserved(v[0], &allocator)
    };
    flag(tool.is_empty());
    setup(&mut tool, v[1], v[2]);
    bind_initial(&mut tool, v[3]);
    flag(tool.is_empty());
    queries(&mut tool);
    report_unit(alter(&mut tool, v[4]));
    queries(&mut tool);
    tool.init(500);
    flag(tool.is_empty());
    queries(&mut tool);
    report_unit(tool.set_cell_size(1.));
    flag(tool.is_empty());
    queries(&mut tool);
    println!();
}
fn bind_initial(tool: &mut Tool, radius: i32) {
    for index in 0..3 {
        let mut circle = Circle2::new(Default::default(), scalar(radius)).unwrap();
        circle.set_location(rustycad_tkmath::gp::Point2::new(f64::from(index) - 1., 0.));
        report_unit(tool.bind(index, &circle));
    }
}
fn setup(tool: &mut Tool, cells: i32, bounds: i32) {
    if cells < 2 {
        report_unit(tool.set_cell_size(if cells == 0 { 1. } else { -1. }));
    } else {
        report_unit(tool.set_cell_sizes(0.5, 1.));
    }
    match bounds {
        1 => tool.set_min_max_size(Xy::new(-3., -3.), Xy::new(3., 3.)),
        2 => tool.set_min_max_size(Xy::new(1., 1.), Xy::new(-1., -1.)),
        _ => (),
    }
}
fn queries(tool: &mut Tool) {
    for (x, y) in [
        (-2., 0.),
        (-1., 0.),
        (0., 0.),
        (1., 0.),
        (2., 0.),
        (0., 1.),
        (0., 2.),
    ] {
        match tool.select(&Xy::new(x, y)) {
            Ok(hits) => {
                print!("OK ");
                report_hits(hits);
                hits.append(99);
            }
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
fn alter(tool: &mut Tool, mode: i32) -> Result<(), rustycad_tkernel::standard::Failure> {
    match mode {
        0 => tool.delete(1)?,
        1 => tool.moc_bind(1)?,
        2 => tool.bind(1, &Circle2::new(Default::default(), 2.)?)?,
        3 => flag(tool.bind_points(1, &[Xy::new(-1., 0.), Xy::new(0., 1.), Xy::new(1., 0.)])?),
        4 => flag(tool.bind_points(1, &[Xy::default(); 3])?),
        5 => tool.set_cell_sizes(1., 2.)?,
        _ => (),
    }
    Ok(())
}
pub fn make_case(v: &[i32]) {
    let points = construction_points(v);
    let mut center = Xy::new(7., 8.);
    let mut radius = 9.;
    flag(Tool::make_circle(&points, &mut center, &mut radius));
    real(center.x());
    real(center.y());
    real(radius);
    println!();
}
fn construction_points(v: &[i32]) -> [Xy; 3] {
    let scales = [
        0.,
        1e-155,
        1e-10,
        1e-9,
        1e-8,
        1.,
        1e9,
        1e150,
        f64::INFINITY,
        f64::NAN,
    ];
    let shifts = [0., 1., 1e9, -1e9];
    let scale = scales[v[0] as usize];
    let shift = shifts[v[1] as usize];
    let mut points = [
        Xy::new(-scale + shift, shift),
        Xy::new(shift, scale + shift),
        Xy::new(scale + shift, shift),
    ];
    if v[2] & 1 != 0 {
        points[1] = points[0];
    }
    if v[2] & 2 != 0 {
        points.swap(0, 2);
    }
    if v[2] & 4 != 0 {
        points[1].set_y(shift + scale * 1e-12);
    }
    points
}
pub fn random_case() {
    let mut random = random::CircleRandom::seeded();
    let mut failed = 0;
    for _ in 0..100000 {
        let points = random.accepted_triangle();
        let (made, center, radius) = report_random_circle(&points);
        if made
            && points
                .iter()
                .any(|point| !random::on_circle(*point, center, radius))
        {
            failed += 1;
        }
    }
    int(failed);
    println!();
}
pub fn invalid_case(v: &[i32]) {
    let mut inspector = Inspector::new(1e-9, 4, &Handle::default());
    inspector.bind(0, Circle::default()).unwrap();
    let result = if v[0] == 0 {
        inspector.circle(v[1]).map(|_| ())
    } else {
        inspector.inspect(v[1]).map(|_| ())
    };
    report_unit(result);
    println!();
}

fn report_random_circle(points: &[Xy; 3]) -> (bool, Xy, f64) {
    for point in *points {
        real(point.x());
        real(point.y());
    }
    let mut center = Xy::default();
    let mut radius = 0.;
    let made = Tool::make_circle(points, &mut center, &mut radius);
    flag(made);
    real(center.x());
    real(center.y());
    real(radius);
    (made, center, radius)
}
