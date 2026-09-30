//! BRepBndLib probe operations over shapes read by `tp_shape`, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepbndlib.cxx.
use super::common::{Reader, attempt, end, flag, real};
use super::shape::tp_shape;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// `tp_box`: void flag, limits, gap, whole flag and open flags.
pub fn print_box(b: &BoundingBox3) {
    flag(b.is_void());
    if b.is_void() {
        return;
    }
    b.get()
        .expect("limits of a box that is not void")
        .into_iter()
        .for_each(real);
    real(b.gap());
    flag(b.is_whole());
    [
        b.is_open_xmin(),
        b.is_open_xmax(),
        b.is_open_ymin(),
        b.is_open_ymax(),
        b.is_open_zmin(),
        b.is_open_zmax(),
    ]
    .into_iter()
    .for_each(flag);
}

/// `tp_initial_box`: an empty box or an initial box to extend.
fn initial_box(r: &mut Reader<'_>) -> BoundingBox3 {
    let mut b = BoundingBox3::default();
    if !r.flag() {
        return b;
    }
    let lower = [r.real(), r.real(), r.real()];
    let upper = [r.real(), r.real(), r.real()];
    b.update_bounds(lower, upper);
    b.set_gap(r.real());
    b
}

/// `shape pre mode flags`: Add, AddClose or AddOptimal into the box, then the box.
pub fn add_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let mut b = initial_box(&mut r);
        let mode = r.integer();
        let (f1, f2) = (r.flag(), r.flag());
        attempt(|| match mode {
            0 => BRepBndLib::add(&s.shape, &mut b, f1),
            1 => BRepBndLib::add_close(&s.shape, &mut b),
            _ => BRepBndLib::add_optimal(&s.shape, &mut b, f1, f2),
        });
        print_box(&b);
        Ok(())
    });
    end();
}
