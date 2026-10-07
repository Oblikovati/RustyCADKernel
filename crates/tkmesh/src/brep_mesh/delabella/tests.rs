//! Unit tests of the DelaBella port: the degenerate outputs, the logs, the Delaunay
//! property and the failure modes.
use super::{DelaBella, dela_bella};
use std::sync::{Arc, Mutex};

/// A log sink that records every message.
#[derive(Clone, Default)]
struct RecordingLog(Arc<Mutex<Vec<String>>>);

impl RecordingLog {
    fn attach(&self, triangulator: &mut DelaBella) {
        let sink = Arc::clone(&self.0);
        triangulator.set_err_log(Some(Box::new(move |text: &str| {
            sink.lock().unwrap().push(text.to_string());
        })));
    }
    fn messages(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

fn triangles(triangulator: &DelaBella) -> Vec<[i32; 3]> {
    let mut result = Vec::new();
    let mut face = triangulator.get_first_delaunay_triangle();
    while let Some(index) = face {
        let triangle = triangulator.triangle(index).unwrap();
        result.push(triangle.v.map(|v| triangulator.vertex(v).unwrap().i));
        face = triangle.next;
    }
    result
}

fn hull_list(triangulator: &DelaBella, limit: usize) -> Vec<i32> {
    let mut result = Vec::new();
    let first = triangulator.get_first_hull_vertex();
    let mut vert = first;
    while let Some(index) = vert.filter(|_| result.len() < limit) {
        if !result.is_empty() && vert == first {
            break;
        }
        let vertex = triangulator.vertex(index).unwrap();
        result.push(vertex.i);
        vert = vertex.next;
    }
    result
}

/// A deterministic cloud on a 1/64 grid (exact in binary).
fn grid_cloud(count: usize) -> Vec<f64> {
    let mut state: u32 = 12345;
    let mut xy = Vec::with_capacity(2 * count);
    for _ in 0..2 * count {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        xy.push(f64::from((state >> 16) % 4096) / 64.0);
    }
    xy
}

#[test]
fn trivial_triangle_is_clockwise() {
    let mut triangulator = DelaBella::create();
    let log = RecordingLog::default();
    log.attach(&mut triangulator);
    let xy = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
    assert_eq!(triangulator.triangulate_f64(3, &xy, None, 0).unwrap(), 3);
    let tris = triangles(&triangulator);
    assert_eq!(tris.len(), 1);
    let [a, b, c] = tris[0].map(|i| (xy[2 * i as usize], xy[2 * i as usize + 1]));
    let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    assert!(area < 0.0, "triangle {:?} is not clockwise", tris[0]);
    assert_eq!(
        log.messages(),
        ["[NFO] trivial case of 3 points, thank you.\n"]
    );
    assert_eq!(hull_list(&triangulator, 10).len(), 3);
}

#[test]
fn quad_gives_the_native_triangles() {
    // Native: 6, then (0 2 1) and (1 2 3).
    let mut triangulator = DelaBella::create();
    let xy = [0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 3.0, 2.0];
    assert_eq!(triangulator.triangulate_f64(4, &xy, None, 0).unwrap(), 6);
    assert_eq!(triangles(&triangulator), [[0, 2, 1], [1, 2, 3]]);
    assert_eq!(triangulator.get_num_input_points(), 4);
    assert_eq!(triangulator.get_num_output_verts(), 6);
}

#[test]
fn all_cocircular_points_overrun_the_face_pool() {
    // Native quirk: the retracted contour still holds the cone top, so the cone needs
    // 2n-2 faces of a 2n-4 pool and Face::Alloc dereferences null (a segfault).
    let mut triangulator = DelaBella::create();
    let log = RecordingLog::default();
    log.attach(&mut triangulator);
    let xy = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
    let failure = triangulator.triangulate_f64(4, &xy, None, 0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert_eq!(log.messages(), ["[NFO] all input points are cocircular.\n"]);
}

#[test]
fn colinear_points_return_a_segment_list() {
    let mut triangulator = DelaBella::create();
    let log = RecordingLog::default();
    log.attach(&mut triangulator);
    let xy = [0.0, 0.0, 2.0, 2.0, 1.0, 1.0, 3.0, 3.0];
    assert_eq!(triangulator.triangulate_f64(4, &xy, None, 0).unwrap(), -4);
    assert_eq!(hull_list(&triangulator, 10).len(), 4);
    assert_eq!(
        log.messages(),
        ["[WRN] all input points are colinear, returning segment list!\n"]
    );
}

#[test]
fn duplicates_are_removed_and_reported() {
    let mut triangulator = DelaBella::create();
    let log = RecordingLog::default();
    log.attach(&mut triangulator);
    let xy = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    assert_eq!(triangulator.triangulate_f64(5, &xy, None, 0).unwrap(), 3);
    assert_eq!(log.messages()[0], "[WRN] detected 2 dups in xy array!\n");
}

#[test]
fn one_and_two_distinct_points() {
    let mut triangulator = DelaBella::create();
    let log = RecordingLog::default();
    log.attach(&mut triangulator);
    assert_eq!(
        triangulator
            .triangulate_f64(2, &[0.0, 0.0, 0.0, 0.0], None, 0)
            .unwrap(),
        -1
    );
    assert_eq!(hull_list(&triangulator, 10).len(), 1);
    assert_eq!(
        triangulator
            .triangulate_f64(2, &[0.0, 0.0, 1.0, 0.0], None, 0)
            .unwrap(),
        -2
    );
    assert_eq!(hull_list(&triangulator, 10), [0, 1]);
    assert_eq!(
        log.messages()[1..],
        [
            "[WRN] all input points are identical, returning single point!\n",
            "[WRN] all input points are colinear, returning single segment!\n"
        ]
    );
}

#[test]
fn random_cloud_is_delaunay_with_euler_count() {
    let xy = grid_cloud(200);
    let mut triangulator = DelaBella::create();
    let verts = triangulator.triangulate_f64(200, &xy, None, 0).unwrap();
    let tris = triangles(&triangulator);
    assert_eq!(verts, 3 * tris.len() as i32);
    let hull = hull_list(&triangulator, 1000);
    let distinct = 200 - count_duplicates(&xy);
    assert_eq!(tris.len(), 2 * distinct - 2 - hull.len());
    for triangle in &tris {
        assert_empty_circumcircle(&xy, *triangle);
    }
}

fn count_duplicates(xy: &[f64]) -> usize {
    let mut points: Vec<(u64, u64)> = xy
        .chunks(2)
        .map(|p| (p[0].to_bits(), p[1].to_bits()))
        .collect();
    let total = points.len();
    points.sort_unstable();
    points.dedup();
    total - points.len()
}

/// No input point lies strictly inside the circumcircle (exact on the 1/64 grid).
fn assert_empty_circumcircle(xy: &[f64], triangle: [i32; 3]) {
    let point = |i: i32| (xy[2 * i as usize], xy[2 * i as usize + 1]);
    let [a, b, c] = triangle.map(point);
    for d in xy.chunks(2).map(|p| (p[0], p[1])) {
        let rows = [a, b, c].map(|p| {
            let (x, y) = (p.0 - d.0, p.1 - d.1);
            (x, y, x * x + y * y)
        });
        let det = rows[0].0 * (rows[1].1 * rows[2].2 - rows[2].1 * rows[1].2)
            - rows[0].1 * (rows[1].0 * rows[2].2 - rows[2].0 * rows[1].2)
            + rows[0].2 * (rows[1].0 * rows[2].1 - rows[2].0 * rows[1].1);
        // Clockwise triangles: a point inside gives a negative determinant.
        assert!(
            det >= 0.0,
            "point {d:?} inside the circumcircle of {triangle:?}"
        );
    }
}

#[test]
fn float_overload_widens_to_double() {
    let xy64 = grid_cloud(40);
    let xy32: Vec<f32> = xy64.iter().map(|&v| v as f32).collect();
    let mut first = DelaBella::create();
    let mut second = DelaBella::create();
    let a = first.triangulate_f64(40, &xy64, None, 0).unwrap();
    let b = second.triangulate_f32(40, &xy32, None, 0).unwrap();
    assert_eq!(a, b);
    assert_eq!(triangles(&first), triangles(&second));
}

#[test]
fn separate_y_slice_and_advance() {
    let x = [0.0, 9.0, 9.0, 1.0, 9.0, 9.0, 0.0, 9.0, 9.0];
    let y = [0.0, 9.0, 9.0, 0.0, 9.0, 9.0, 1.0, 9.0, 9.0];
    let mut triangulator = DelaBella::create();
    assert_eq!(triangulator.triangulate_f64(3, &x, Some(&y), 3).unwrap(), 3);
}

#[test]
fn invalid_inputs_are_named_failures() {
    let mut triangulator = DelaBella::create();
    let kind = |result: Result<i32, rustycad_tkernel::standard::Failure>| {
        result.unwrap_err().exception_type().to_string()
    };
    assert_eq!(
        kind(triangulator.triangulate_f64(0, &[0.0; 2], None, 0)),
        "Standard_DomainError"
    );
    assert_eq!(
        kind(triangulator.triangulate_f64(3, &[0.0; 4], None, 0)),
        "Standard_OutOfRange"
    );
    let nan = [0.0, 0.0, f64::NAN, 1.0, 1.0, 0.0];
    assert_eq!(
        kind(triangulator.triangulate_f64(3, &nan, None, 0)),
        "Standard_DomainError"
    );
}

#[test]
fn deprecated_interface_writes_indices() {
    let mut messages = Vec::new();
    let mut errlog = |text: &str| messages.push(text.to_string());
    let xy = [0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 3.0, 2.0];
    let mut abc = [-1; 6];
    assert_eq!(
        dela_bella(4, &xy, Some(&mut abc), Some(&mut errlog)).unwrap(),
        6
    );
    assert_eq!(
        messages,
        ["[WRN] Depreciated interface! errlog disabled.\n"]
    );
    assert_eq!(abc, [0, 2, 1, 1, 2, 3]);
    let mut line = [-1; 3];
    let colinear = [0.0, 0.0, 1.0, 1.0, 2.0, 2.0];
    assert_eq!(dela_bella(3, &colinear, Some(&mut line), None).unwrap(), -3);
    assert_eq!(dela_bella(0, &xy, None, None).unwrap(), 0);
    let mut short = [0; 2];
    assert!(dela_bella(4, &xy, Some(&mut short), None).is_err());
}
