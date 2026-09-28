//! Empty-bounds DoubleTab, Matrix and VectorBase oracle counterpart
//! (tools/oracle/tkmath_empty_bounds_probe.cxx).
/// Runs a printing expression inside a `guarded` body; `shown!(print!("x"))`.
macro_rules! shown {
    ($printed:expr) => {{
        $printed;
        Ok(())
    }};
}
mod solvers;
use super::vector_base_probe::{cells, dumped, guarded, reals, values};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Crout, DoubleTab, Gauss, IntegerVector, Matrix, Vector};
pub(super) use solvers::empty_solvers_case;

/// Inclusive `[rl, ru] x [cl, cu]` bounds of one request.
#[derive(Clone, Copy)]
struct Bounds {
    rl: i32,
    ru: i32,
    cl: i32,
    cu: i32,
}

fn table(t: &DoubleTab<'_>) {
    print!(
        "{} {} {} {} {} {} {} {} {} ",
        t.lower_row(),
        t.upper_row(),
        t.lower_col(),
        t.upper_col(),
        t.nb_rows(),
        t.nb_columns(),
        t.row_size(),
        t.col_size(),
        u8::from(t.is_deletable())
    );
}
fn array_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    error.into_failure("math_DoubleTab")
}
fn double_tab_part(b: Bounds, s: f64) {
    let mut t = DoubleTab::with_bounds(b.rl, b.ru, b.cl, b.cu).unwrap();
    table(&t);
    t.init(s);
    let mut copy = t.clone();
    table(&copy);
    let mut other = DoubleTab::with_bounds(b.rl, b.ru, b.cl, b.cu).unwrap();
    guarded(|| {
        t.copy_to(&mut other).map_err(array_failure)?;
        table(&other);
        Ok(())
    });
    guarded(|| {
        shown!(print!(
            "{} ",
            t.value(b.rl, b.cl).map_err(array_failure)?.to_bits()
        ))
    });
    guarded(|| shown!(print!("{} ", t.at(0, 0).map_err(array_failure)?.to_bits())));
    table(&DoubleTab::move_construct(&mut copy));
    table(&DoubleTab::with_size(
        extent(b.rl, b.ru),
        extent(b.cl, b.cu),
    ));
    let mut buffer = [s; 4];
    table(&DoubleTab::from_buffer(&mut buffer, b.rl, b.ru, b.cl, b.cu).unwrap());
}
fn extent(lower: i32, upper: i32) -> usize {
    (upper - lower + 1) as usize
}
fn matrix_values(m: &Matrix<'_>, b: Bounds, s: f64) {
    cells(&m.multiplied(s));
    cells(&m.opposite());
    cells(&m.transposed());
    guarded(|| shown!(cells(&m.divided(s)?)));
    let mut a = m.clone();
    a.multiply(s);
    guarded(|| a.add(m));
    guarded(|| a.subtract(m));
    cells(&a);
    guarded(|| shown!(cells(&m.added(m)?)));
    guarded(|| shown!(cells(&m.subtracted(m)?)));
    guarded(|| shown!(cells(&m.multiplied_matrix(&m.transposed())?)));
    guarded(|| shown!(cells(&m.t_multiply(m)?)));
    matrix_products(m, b);
    matrix_edits(m, b, s);
}
fn matrix_products(m: &Matrix<'_>, b: Bounds) {
    let mut product = Matrix::with_bounds(b.rl, b.ru, b.rl, b.ru).unwrap();
    guarded(|| {
        product.set_product(m, &m.transposed())?;
        shown!(cells(&product))
    });
    let mut t_product = Matrix::with_bounds(b.cl, b.cu, b.cl, b.cu).unwrap();
    guarded(|| {
        t_product.set_transpose_product(m, m)?;
        shown!(cells(&t_product))
    });
}
fn matrix_edits(m: &Matrix<'_>, b: Bounds, s: f64) {
    let edited = |edit: &dyn Fn(&mut Matrix<'_>) -> Result<(), Failure>| {
        guarded(|| {
            let mut w = m.clone();
            edit(&mut w)?;
            shown!(cells(&w))
        })
    };
    edited(&|w| w.set_diag(s));
    edited(&|w| w.transpose());
    edited(&|w| w.set(b.rl, b.ru, b.cl, b.cu, m));
}
fn matrix_rows(m: &Matrix<'_>, b: Bounds, s: f64) {
    guarded(|| shown!(reals(&m.row(b.rl)?)));
    guarded(|| shown!(reals(&m.col(b.cl)?)));
    let row = Vector::filled(b.cl, b.cu, s).unwrap();
    let col = Vector::filled(b.rl, b.ru, -s).unwrap();
    let edited = |edit: &dyn Fn(&mut Matrix<'_>) -> Result<(), Failure>| {
        guarded(|| {
            let mut w = m.clone();
            edit(&mut w)?;
            shown!(cells(&w))
        })
    };
    edited(&|w| w.set_row(b.rl, &row));
    edited(&|w| w.set_col(b.cl, &col));
    edited(&|w| w.swap_row(b.rl, b.ru));
    edited(&|w| w.swap_col(b.cl, b.cu));
    guarded(|| shown!(reals(&m.multiplied_vector(&row)?)));
    edited(&|w| w.set_outer_product(&col, &row));
}
fn matrix_solvers(m: &Matrix<'_>) {
    guarded(|| shown!(print!("{} ", m.determinant()?.to_bits())));
    guarded(|| shown!(cells(&m.inverse()?)));
    guarded(|| {
        let solver = Gauss::new(m)?;
        print!("{} ", u8::from(solver.is_done()));
        shown!(print!("{} ", solver.determinant()?.to_bits()))
    });
    guarded(|| {
        let solver = Crout::new(m)?;
        print!("{} ", u8::from(solver.is_done()));
        print!("{} ", solver.determinant()?.to_bits());
        shown!(cells(solver.inverse()?))
    });
}
fn matrix_part(b: Bounds, s: f64) {
    let mut m = Matrix::filled(b.rl, b.ru, b.cl, b.cu, s).unwrap();
    cells(&m);
    print!("{} {} ", m.row_number(), m.col_number());
    let plain = Matrix::with_bounds(b.rl, b.ru, b.cl, b.cu).unwrap();
    print!("{} {} ", plain.row_number(), plain.col_number());
    matrix_values(&m, b, s);
    matrix_rows(&m, b, s);
    matrix_solvers(&m);
    cells(&Matrix::with_size_filled(
        extent(b.rl, b.ru),
        extent(b.cl, b.cu),
        s,
    ));
    let mut buffer = [s, -s, s, -s];
    cells(&Matrix::from_buffer(&mut buffer, b.rl, b.ru, b.cl, b.cu).unwrap());
    let moved = Matrix::move_construct(&mut m);
    cells(&moved);
    dumped(&moved);
}
fn vector_values(v: &Vector<'_>, s: f64) {
    guarded(|| shown!(print!("{} ", v.norm()?.to_bits())));
    guarded(|| shown!(print!("{} ", v.norm2()?.to_bits())));
    print!("{} {} ", v.max_index(), v.min_index());
    reals(&v.opposite());
    reals(&v.multiplied(s));
    guarded(|| shown!(reals(&v.divided(s)?)));
    guarded(|| shown!(reals(&v.normalized()?)));
    guarded(|| {
        let mut w = v.clone();
        w.normalize()?;
        shown!(reals(&w))
    });
    guarded(|| shown!(reals(&v.inverse()?)));
    guarded(|| {
        let mut w = v.clone();
        w.invert()?;
        shown!(reals(&w))
    });
    guarded(|| shown!(reals(&v.added(v)?)));
    guarded(|| shown!(reals(&v.subtracted(v)?)));
    guarded(|| shown!(print!("{} ", v.dot(v)?.to_bits())));
}
fn vector_part(vl: i32, vu: i32, s: f64) {
    let mut v = Vector::filled(vl, vu, s).unwrap();
    reals(&v);
    vector_values(&v, s);
    guarded(|| shown!(reals(&v.slice(vl, vu)?)));
    guarded(|| {
        let mut w = v.clone();
        w.set(vl, vu, &v)?;
        shown!(reals(&w))
    });
    reals(&Vector::with_size_filled(extent(vl, vu), s));
    let mut buffer = [s, -s];
    reals(&Vector::from_buffer(&mut buffer, vl, vu).unwrap());
    let moved = Vector::move_construct(&mut v);
    reals(&moved);
    dumped(&moved);
    integer_part(vl, vu, s as i32);
}
fn integer_part(vl: i32, vu: i32, s: i32) {
    let iv = IntegerVector::filled(vl, vu, s).unwrap();
    values(&iv);
    guarded(|| shown!(print!("{} ", iv.norm()?.to_bits())));
    print!("{} {} ", iv.max_index(), iv.min_index());
    guarded(|| shown!(values(&iv.added(&iv)?)));
    guarded(|| shown!(print!("{} ", iv.dot(&iv)?)));
}
/// `rl ru cl cu vl vu scalar`.
pub(super) fn empty_bounds_case(v: &[f64]) {
    let b = Bounds {
        rl: v[0] as i32,
        ru: v[1] as i32,
        cl: v[2] as i32,
        cu: v[3] as i32,
    };
    double_tab_part(b, v[6]);
    matrix_part(b, v[6]);
    vector_part(v[4] as i32, v[5] as i32, v[6]);
    println!();
}
