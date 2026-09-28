//! CSLib normals, normal derivatives, NormalPolyDef and Class2d oracle counterpart.
use rustycad_tkernel::ncollection::{Array1, Array2, DynamicArray, Sequence};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::cslib::{self, Class2d, NormalPolyDef, NormalResult, ParametricPosition};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3, Xyz};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// Sequential reader over one request's numeric fields.
struct Fields<'v> {
    values: &'v [f64],
    next: usize,
}

impl Fields<'_> {
    fn real(&mut self) -> f64 {
        self.next += 1;
        self.values[self.next - 1]
    }
    fn int(&mut self) -> i32 {
        self.real() as i32
    }
    fn vector(&mut self) -> Vector3 {
        Vector3::new(self.real(), self.real(), self.real())
    }
    fn table(&mut self, size: i32) -> Array2<'static, Vector3> {
        let mut table = Array2::with_bounds(0, size, 0, size).unwrap();
        for i in 0..=size {
            for j in 0..=size {
                table[(i, j)] = self.vector();
            }
        }
        table
    }
}

fn xyz(value: Xyz) {
    print!(
        "{} {} {} ",
        value.x().to_bits(),
        value.y().to_bits(),
        value.z().to_bits()
    );
}

// The native probe prints its default-constructed gp_Dir when no normal is written.
fn direction(normal: Option<Direction3>) {
    xyz(Xyz::from(normal.unwrap_or(Direction3::X)));
}

fn failure(error: &Failure) {
    print!("{} ", error.exception_type());
}

fn vector_or_failure(result: Result<Vector3, Failure>) {
    match result {
        Ok(value) => xyz(*value.xyz()),
        Err(error) => failure(&error),
    }
}

fn status_normal<S: Copy + Into<i32>>(result: Result<NormalResult<S>, Failure>) {
    match result {
        Ok(value) => {
            print!("{} ", value.status.into());
            direction(value.normal);
        }
        Err(error) => failure(&error),
    }
}

pub fn cslib_normal_case(values: &[f64]) {
    let mut fields = Fields { values, next: 0 };
    let [d1u, d1v, d2u, d2v, d2uv] = std::array::from_fn(|_| fields.vector());
    let tol = fields.real();
    status_normal(cslib::normal(d1u, d1v, tol).map(|r| as_i32(r, |s| s as i32)));
    let second = cslib::normal_from_second_derivatives(d1u, d1v, d2u, d2v, d2uv, tol);
    print!("{} {} ", i32::from(second.is_done()), second.status as i32);
    direction(second.normal);
    status_normal(cslib::normal_with_magnitude(d1u, d1v, tol).map(|r| as_i32(r, |s| s as i32)));
    println!();
}

fn as_i32<S: Copy>(result: NormalResult<S>, code: impl Fn(S) -> i32) -> NormalResult<i32> {
    NormalResult {
        status: code(result.status),
        normal: result.normal,
    }
}

pub fn cslib_singular_case(values: &[f64]) {
    let mut fields = Fields { values, next: 0 };
    let order = fields.int();
    let size = fields.int();
    let tol = fields.real();
    let position = ParametricPosition {
        u: fields.real(),
        v: fields.real(),
        u_min: fields.real(),
        u_max: fields.real(),
        v_min: fields.real(),
        v_max: fields.real(),
    };
    let der = fields.table(size);
    match cslib::normal_at_singularity(order, &der, tol, position) {
        Ok(value) => {
            print!("{} ", value.result.status as i32);
            direction(value.result.normal);
            print!("{} {} ", value.order_u, value.order_v);
        }
        Err(error) => failure(&error),
    }
    println!();
}

pub fn cslib_dn_case(values: &[f64]) {
    let mut fields = Fields { values, next: 0 };
    let [size, nu, nv, iu, iv] = std::array::from_fn(|_| fields.int());
    let first = fields.table(size);
    let second = fields.table(size);
    vector_or_failure(cslib::dnnuv(nu, nv, &first));
    vector_or_failure(cslib::dnnuv_osculating(nu, nv, &first, &second));
    vector_or_failure(cslib::dn_normal(nu, nv, &first, iu, iv));
    println!();
}

pub fn cslib_poly_case(values: &[f64]) {
    let mut fields = Fields { values, next: 0 };
    let k = fields.int();
    let x = fields.real();
    let mut li = Array1::with_bounds(0, k.max(0)).unwrap();
    for i in 0..=k.max(0) {
        li[i] = fields.real();
    }
    match NormalPolyDef::new(k, &li) {
        Ok(mut polynomial) => poly_outputs(&mut polynomial, x),
        Err(error) => failure(&error),
    }
    println!();
}

fn poly_outputs(polynomial: &mut NormalPolyDef, x: f64) {
    let (mut f, mut d, mut vf, mut vd) = (91.0_f64, 92.0_f64, 93.0_f64, 94.0_f64);
    let a = polynomial.value(x, &mut f);
    let b = polynomial.derivative(x, &mut d);
    let c = polynomial.values(x, &mut vf, &mut vd);
    print!(
        "{} {} {} {} {} {} {} ",
        i32::from(a),
        i32::from(b),
        i32::from(c),
        f.to_bits(),
        d.to_bits(),
        vf.to_bits(),
        vd.to_bits()
    );
}

fn classifier(fields: &mut Fields<'_>, container: i32, tol: [f64; 2], b: [f64; 4]) -> Class2d {
    let n = fields.int();
    let mut points = Array1::with_bounds(1, n.max(1)).unwrap();
    let mut sequence = Sequence::default();
    let mut dynamic = DynamicArray::default();
    for i in 1..=n {
        points[i] = Point2::new(fields.real(), fields.real());
        sequence.append(points[i]);
        dynamic.append(points[i]);
    }
    let [tu, tv] = tol;
    match container {
        1 => Class2d::from_sequence(&sequence, tu, tv, b[0], b[1], b[2], b[3]),
        2 => Class2d::from_dynamic_array(&dynamic, tu, tv, b[0], b[1], b[2], b[3]),
        _ => Class2d::from_array(&points, tu, tv, b[0], b[1], b[2], b[3]),
    }
}

fn queries(classifier: &Class2d, points: &[Point2]) {
    for point in points {
        print!("{} ", classifier.si_dans(*point) as i32);
    }
}

pub fn cslib_class2d_case(values: &[f64]) {
    let mut fields = Fields { values, next: 0 };
    let special = fields.int();
    let container = fields.int();
    let mut tol = [fields.real(), fields.real()];
    let mut on = fields.real();
    let b: [f64; 4] = std::array::from_fn(|_| fields.real());
    if special & 1 != 0 {
        tol[0] = f64::INFINITY;
    }
    if special & 2 != 0 {
        tol[1] = f64::NAN;
    }
    if special & 4 != 0 {
        on = f64::INFINITY;
    }
    let mut c = classifier(&mut fields, container, tol, b);
    let m = fields.int();
    let q: Vec<Point2> = (0..m)
        .map(|_| Point2::new(fields.real(), fields.real()))
        .collect();
    queries(&c, &q);
    for point in &q {
        print!("{} ", c.si_dans_on_mode(*point, on) as i32);
    }
    lifecycle(
        &mut c,
        &q,
        Point2::new(b[0] / 2.0 + b[2] / 2.0, b[1] / 2.0 + b[3] / 2.0),
    );
    println!();
}

// Warms the lazy grid at the domain center, then exercises copy and both moves.
fn lifecycle(c: &mut Class2d, q: &[Point2], center: Point2) {
    for _ in 0..64 {
        print!("{}", c.si_dans(center) as i32);
    }
    print!(" ");
    queries(c, q);
    let copy = c.clone();
    queries(&copy, q);
    let mut moved = Class2d::moved_from(c);
    print!("{} ", c.si_dans(center) as i32);
    queries(&moved, q);
    let mut assigned = Class2d::default();
    assigned.move_from(&mut moved);
    print!("{} ", moved.si_dans(center) as i32);
    queries(&assigned, q);
}
