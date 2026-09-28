//! Rust counterpart of tools/oracle/tkmath_bnd_probe.cxx: Bnd_OBB, Bnd_BoundSortBox,
//! Bnd_Tools and the remaining Bnd_Box/Box2d/B2/B3 members.
use super::xyz;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::bnd::{BoundingBox3, ObbHalfSizes, OrientedBoundingBox};
use rustycad_tkmath::gp::{Direction3, Point3, Xyz};

mod arrays;
mod extra;
mod sort;
pub(super) use arrays::compact_arrays_case;
pub(super) use extra::{box_extra_case, box2_extra_case, compact_trsf_case};
pub(super) use sort::sortbox_case;

/// Sequential reader over one request's numeric fields.
pub(crate) struct Reader<'v> {
    values: &'v [f64],
    next: usize,
}
impl<'v> Reader<'v> {
    pub(crate) fn new(values: &'v [f64]) -> Self {
        Self { values, next: 0 }
    }
    pub(crate) fn real(&mut self) -> f64 {
        self.next += 1;
        self.values[self.next - 1]
    }
    pub(crate) fn integer(&mut self) -> i32 {
        self.real() as i32
    }
    pub(crate) fn triple(&mut self) -> Xyz {
        Xyz::new(self.real(), self.real(), self.real())
    }
}

pub(crate) fn failure(error: &Failure) {
    print!("{} ", error.exception_type());
}
fn flag(value: bool) -> i32 {
    i32::from(value)
}

fn obb_values(obb: &OrientedBoundingBox) {
    print!("{} {} ", flag(obb.is_void()), flag(obb.is_aa_box()));
    for value in [
        obb.center(),
        obb.x_direction(),
        obb.y_direction(),
        obb.z_direction(),
    ] {
        xyz(value);
    }
    let ObbHalfSizes { x, y, z } = obb.half_sizes();
    for value in [x, y, z, obb.square_extent()] {
        print!("{} ", value.to_bits());
    }
}
fn obb_vertices(obb: &OrientedBoundingBox) {
    let vertices = obb.vertices();
    print!("{} ", flag(vertices.is_some()));
    for vertex in vertices.into_iter().flatten() {
        xyz(vertex.xyz());
    }
}
fn obb_position(obb: &OrientedBoundingBox) {
    match obb.position() {
        Ok(frame) => {
            xyz(frame.location().xyz());
            xyz(frame.direction().into());
            xyz(frame.x_direction().into());
            xyz(frame.y_direction().into());
        }
        Err(error) => failure(&error),
    }
}
fn obb_dump(obb: &OrientedBoundingBox, depth: i32) {
    let mut stream = DumpStream::default();
    obb.dump_json(&mut stream, depth).unwrap();
    print!("{} | ", stream.text());
}
fn status(result: Result<(), Failure>) {
    match result {
        Ok(()) => print!("ok "),
        Err(error) => failure(&error),
    }
}

/// `obb_build optimal with_tolerances point_lower tolerance_lower n x y z [tol]...`
pub(super) fn obb_build_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (optimal, tolerant) = (reader.integer() != 0, reader.integer() != 0);
    let (lower, tolerance_lower, count) = (reader.integer(), reader.integer(), reader.integer());
    let mut points = Array1::with_bounds(lower, lower + count - 1).unwrap();
    let mut tolerances = Array1::with_bounds(tolerance_lower, tolerance_lower + count - 1).unwrap();
    for i in 0..count {
        points[lower + i] = Point3::from(reader.triple());
        if tolerant {
            tolerances[tolerance_lower + i] = reader.real();
        }
    }
    let mut obb = initial_obb();
    status(obb.rebuild(&points, tolerant.then_some(&tolerances), optimal));
    obb_values(&obb);
    obb_vertices(&obb);
    obb_position(&obb);
    for point in points.iter() {
        print!("{}", flag(obb.is_out_point(*point)));
    }
    print!(" ");
    obb_dump(&obb, -1);
    obb_dump(&obb, 0);
    println!();
}
fn initial_obb() -> OrientedBoundingBox {
    let axes = [Direction3::X, Direction3::Y, Direction3::Z];
    OrientedBoundingBox::new(Point3::new(7.0, 8.0, 9.0), axes, 1.0, 2.0, 3.0)
}
fn obb_read(reader: &mut Reader<'_>) -> OrientedBoundingBox {
    let center = Point3::from(reader.triple());
    let axes = [(); 3].map(|_| Direction3::from_xyz(reader.triple()).unwrap());
    let half = reader.triple();
    let mut obb = OrientedBoundingBox::new(center, axes, half.x(), half.y(), half.z());
    obb.set_aa_box(reader.integer() != 0);
    obb
}
pub(crate) fn box_read(reader: &mut Reader<'_>) -> BoundingBox3 {
    let low = reader.triple();
    let high = reader.triple();
    let (gap, flags) = (reader.real(), reader.integer());
    let mut bounds = BoundingBox3::default();
    if flags & 64 == 0 {
        bounds.update_bounds(*low.coordinates(), *high.coordinates());
    }
    bounds.set_gap(gap);
    let openers: [fn(&mut BoundingBox3); 6] = [
        BoundingBox3::open_xmin,
        BoundingBox3::open_xmax,
        BoundingBox3::open_ymin,
        BoundingBox3::open_ymax,
        BoundingBox3::open_zmin,
        BoundingBox3::open_zmax,
    ];
    for (bit, open) in openers.iter().enumerate() {
        if flags & (1 << bit) != 0 {
            open(&mut bounds);
        }
    }
    bounds
}
fn obb_add_box(mut obb: OrientedBoundingBox, other: &OrientedBoundingBox) {
    status(obb.add_box(other));
    obb_values(&obb);
}
fn obb_add_point(mut obb: OrientedBoundingBox, point: Point3) {
    status(obb.add_point(point));
    obb_values(&obb);
}

/// `obb_pair A(16) B(16) point(3) enlarge box(6+gap+flags)`
pub(super) fn obb_pair_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (a, b) = (obb_read(&mut reader), obb_read(&mut reader));
    let point = Point3::from(reader.triple());
    let gap = reader.real();
    let bounds = box_read(&mut reader);
    obb_values(&a);
    obb_values(&b);
    obb_vertices(&a);
    obb_position(&a);
    for value in [
        a.is_out_box(&b),
        b.is_out_box(&a),
        a.intersects(&b),
        a.is_completely_inside(&b),
        b.is_completely_inside(&a),
        a.is_out_point(point),
        a.contains(point),
        b.is_out_point(point),
    ] {
        print!("{}", flag(value));
    }
    print!(" ");
    pair_accumulation(&a, &b, point);
    pair_conversions(&a, point, gap, &bounds);
    println!();
}
fn pair_accumulation(a: &OrientedBoundingBox, b: &OrientedBoundingBox, point: Point3) {
    obb_add_box(*a, b);
    obb_add_box(*b, a);
    obb_add_point(*a, point);
    obb_add_box(OrientedBoundingBox::default(), a);
    obb_add_box(*a, &OrientedBoundingBox::default());
    obb_add_point(OrientedBoundingBox::default(), point);
}
fn pair_conversions(a: &OrientedBoundingBox, point: Point3, gap: f64, bounds: &BoundingBox3) {
    let mut enlarged = *a;
    enlarged.enlarge(gap);
    obb_values(&enlarged);
    let from_box = OrientedBoundingBox::from_box(bounds);
    obb_values(&from_box);
    print!(
        "{}{} ",
        flag(from_box.is_out_box(a)),
        flag(a.is_out_box(&from_box))
    );
    let mut cleared = *a;
    cleared.set_void();
    obb_values(&cleared);
    obb_position(&cleared);
    print!(
        "{}{} ",
        flag(cleared.is_out_point(point)),
        flag(cleared.is_out_box(a))
    );
    obb_dump(a, -1);
    obb_dump(a, 1);
}
