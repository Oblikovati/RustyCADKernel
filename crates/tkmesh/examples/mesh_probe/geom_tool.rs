use super::super::{flag, int, point2, point3, real};
use super::curve_tessellator::refinement as curve_fixture;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    geom_abs::IsoType,
    gp::{Direction3, Point2, Point3, Xy},
};
use rustycad_tkmesh::brep_mesh::{DefaultRangeSplitter, GeomTool};
#[path = "../../tests/support/geom_intersection_fixture.rs"]
mod intersection;
use super::range_splitter::fixture as range_fixture;
pub fn intersection_case(v: &[i32]) {
    let (a, b) = intersection::segment(v[0]);
    let (c, d) = intersection::segment(v[1]);
    let mut xy = Xy::new(17.0, 19.0);
    let mut parameters = [23.0, 29.0];
    int(GeomTool::int_lin_lin(a, b, c, d, &mut xy, &mut parameters) as i32);
    real(xy.x());
    real(xy.y());
    real(parameters[0]);
    real(parameters[1]);
    let mut point = Point2::new(17.0, 19.0);
    int(GeomTool::int_seg_seg(a, b, c, d, v[2] != 0, v[3] != 0, &mut point) as i32);
    point2(point);
    let first = Point3::new(a.x(), a.y(), 0.0);
    let last = Point3::new(b.x(), b.y(), 0.0);
    real(
        GeomTool::square_deflection_of_segment(first, last, Point3::new(c.x(), c.y(), 0.3))
            .unwrap(),
    );
    println!();
}
pub fn surface_case(v: &[i32]) {
    let face = range_fixture::range_face(v[0], 1e-7);
    let mut topology = face.get_face().unwrap();
    topology.set_orientation(super::super::orientation(v[1]));
    let surface = Handle::new(BRepAdaptorSurface::with_face(&topology, true).unwrap());
    let coordinates = [(0.3, 0.7), (0.0, -2.0 / 0.3_f64.sin()), (1.0, 1.0)];
    let (u, vv) = coordinates[v[2] as usize];
    let mut point = Point3::new(11.0, 13.0, 17.0);
    let mut normal = Direction3::X;
    flag(GeomTool::normal(&surface, u, vv, &mut point, &mut normal).unwrap());
    point3(point);
    real(normal.x());
    real(normal.y());
    real(normal.z());
    let mut splitter = DefaultRangeSplitter::default();
    splitter.reset(&face, &Default::default());
    let bounds = range_fixture::range_bounds(v[3]);
    splitter.add_point(Point2::new(bounds[0], bounds[2]));
    splitter.add_point(Point2::new(bounds[1], bounds[3]));
    splitter.adjust_range().unwrap();
    let counts = GeomTool::cells_count(
        &surface.cast(),
        [0, 1, 4, 31, 1000][v[4] as usize],
        [0.0, 0.01, 1.0][v[5] as usize],
        Some(&splitter),
    )
    .unwrap();
    int(counts.0);
    int(counts.1);
    let absent = GeomTool::cells_count(&surface.cast(), 0, 0.0, None).unwrap();
    int(absent.0);
    int(absent.1);
    println!();
}
pub fn sampling_case(v: &[i32]) {
    let (edge, face) = curve_fixture::refinement_fixture(v[0], v[1] != 0, false);
    let surface = face.get_surface();
    let adaptor = BRepAdaptorCurve::with_edge(&edge.get_edge().unwrap()).unwrap();
    let mut tool = if v[2] == 0 {
        GeomTool::new(&adaptor, (0.0, 1.0), 0.1, 0.5, 2, 1e-7).unwrap()
    } else {
        GeomTool::on_surface(
            &surface,
            if v[2] == 1 {
                IsoType::IsoU
            } else {
                IsoType::IsoV
            },
            0.5,
            (0.0, 1.0),
            (0.1, 0.5),
            2,
            1e-7,
        )
        .unwrap()
    };
    int(tool.nb_points());
    int(tool
        .add_point(Point3::new(7.0, 8.0, 9.0), 0.5, v[3] != 0)
        .unwrap());
    int(tool
        .add_point(Point3::new(2.0, 3.0, 4.0), 0.5, v[3] != 0)
        .unwrap());
    int(tool.nb_points());
    for index in 0..=tool.nb_points() + 1 {
        report_sample(&tool, index, &surface);
    }
    println!();
}
fn report_sample(tool: &GeomTool<'_>, index: i32, surface: &Handle<BRepAdaptorSurface>) {
    let mut parameter = 23.0;
    let mut point = Point3::new(11.0, 13.0, 17.0);
    let mut uv = Point2::new(17.0, 19.0);
    flag(
        tool.value_iso(index, 0.37, &mut parameter, &mut point, &mut uv)
            .unwrap(),
    );
    real(parameter);
    point3(point);
    point2(uv);
    flag(
        tool.value_surface(index, surface, &mut parameter, &mut point, &mut uv)
            .unwrap(),
    );
    real(parameter);
    point3(point);
    point2(uv);
}
#[path = "../../tests/support/geom_cells_fixture.rs"]
mod cells_fixture;
pub fn cells_case(v: &[i32]) {
    let surface = if v[0] == 12 {
        Handle::new(rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface::default()).cast()
    } else {
        Handle::new(
            rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface::with_surface(
                &cells_fixture::cells_surface(v[0]),
            )
            .unwrap(),
        )
        .cast()
    };
    let splitter = DefaultRangeSplitter {
        range_u: (0.0, 3.0),
        range_v: (-1.0, 2.0),
        delta: (0.25, 0.75),
        ..Default::default()
    };
    let counts = GeomTool::cells_count(
        &surface,
        [-1, 0, 1, 4, 31, 1000][v[1] as usize],
        [0.0, 1e-5, 0.01, 1.0][v[2] as usize],
        Some(&splitter),
    )
    .unwrap();
    int(counts.0);
    int(counts.1);
    println!();
}
pub fn overflow_steps_case() {
    let face = range_fixture::range_face(0, 1e-7);
    face.set_deflection(0.1);
    let mut splitter = rustycad_tkmesh::brep_mesh::ConeRangeSplitter::default();
    splitter.reset(&face, &Default::default());
    splitter.add_point(Point2::new(0.0, 0.0));
    splitter.add_point(Point2::new(1e100, 1.0));
    let mut counts = (0, 0);
    let steps = splitter
        .get_split_steps(
            &rustycad_tkmesh::imesh_tools::MeshParameters {
                min_size: 0.001,
                ..Default::default()
            },
            &mut counts,
        )
        .unwrap();
    int(counts.0);
    int(counts.1);
    real(steps.0);
    real(steps.1);
    println!();
}
