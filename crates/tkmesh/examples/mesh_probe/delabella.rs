//! DelaBella twin of tkmesh_probe_delabella.cxx: deterministic clouds through the raw
//! triangulator, reported by result, triangles and hull vertices.
use super::super::int;
use rustycad_tkmesh::brep_mesh::delabella::DelaBella;

/// The native 64-bit LCG of `DelabellaCloud`, top 53 bits.
struct DelabellaCloud(u64);

impl DelabellaCloud {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 11
    }
}

/// `delabella_cloud(kind, count)`: interleaved coordinates.
fn cloud(kind: i32, count: i32) -> Vec<f64> {
    let seed = 0x9E37_79B9_7F4A_7C15_u64 ^ (kind * 100_000 + count) as u64;
    let mut cloud = DelabellaCloud(seed);
    let mut xy = Vec::with_capacity(2 * count as usize);
    for k in 0..count {
        let (x, y) = cloud_point(&mut cloud, kind, k, count);
        xy.push(x);
        xy.push(y);
    }
    xy
}

fn cloud_point(cloud: &mut DelabellaCloud, kind: i32, k: i32, count: i32) -> (f64, f64) {
    let unit = 2.0_f64.powi(-53);
    match kind {
        0 => {
            let x = (cloud.next() % 4096) as f64 / 64.0;
            (x, (cloud.next() % 4096) as f64 / 64.0)
        }
        1 | 5 => {
            let x = cloud.next() as f64 * unit - 0.5;
            (x, cloud.next() as f64 * unit - 0.5)
        }
        2 if k == 0 => (0.1, 0.05),
        2 => {
            let t = (f64::from(k) + 0.5) / f64::from(count) * 4.0 - 2.0;
            ((1.0 - t * t) / (1.0 + t * t), 2.0 * t / (1.0 + t * t))
        }
        3 => (f64::from(k % 7), f64::from(k / 7)),
        _ => (f64::from(k), 2.0 * f64::from(k) + 1.0),
    }
}

/// `mesh_delabella_points <kind> <count>`.
pub fn points_case(values: &[i32]) {
    let (kind, count) = (values[0], values[1]);
    let xy = cloud(kind, count);
    let mut triangulator = DelaBella::create();
    let result = if kind == 5 {
        let xyf: Vec<f32> = xy.iter().map(|&value| value as f32).collect();
        triangulator.triangulate_f32(count, &xyf, None, 0)
    } else {
        triangulator.triangulate_f64(count, &xy, None, 0)
    };
    match result {
        Ok(result) => report(&triangulator, result, count),
        Err(failure) => print!("{} ", failure.exception_type()),
    }
    println!();
}

fn report(triangulator: &DelaBella, result: i32, count: i32) {
    print!("OK ");
    int(result);
    let mut face = triangulator
        .get_first_delaunay_triangle()
        .filter(|_| result > 0);
    while let Some(triangle) = face.and_then(|index| triangulator.triangle(index)) {
        for vertex in triangle.v {
            int(triangulator.vertex(vertex).map_or(-1, |v| v.i));
        }
        face = triangle.next;
    }
    let mut vertex = triangulator.get_first_hull_vertex();
    let mut hull = 0;
    while let Some(current) = vertex.and_then(|index| triangulator.vertex(index)) {
        if hull > count {
            break;
        }
        int(current.i);
        vertex = current.next;
        hull += 1;
    }
}
