//! Hatch probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_hatch.cxx.
use super::common::{Reader, attempt, end, flag, int, real, show};
use super::gcc_ent::lin2d;
use rustycad_tkgeomalgo::hatch::{Hatcher, LineForm};

const FORMS: [LineForm; 3] = [LineForm::XLine, LineForm::YLine, LineForm::AnyLine];

// One hatching line: 0 `x y dx dy form` (AddLine), 1 `dx dy dist`, 2 `x` (AddXLine), 3 `y`.
fn add_line(r: &mut Reader, h: &mut Hatcher) {
    match r.integer() {
        0 => {
            let l = r.lin2d();
            h.add_line(l, FORMS[r.integer() as usize]);
        }
        1 => {
            let d = r.dir2d();
            h.add_line_at_distance(d, r.real());
        }
        2 => h.add_x_line(r.real()),
        _ => h.add_y_line(r.real()),
    }
}

// One trimming: 0 `x y dx dy index` (whole line), 1 `x y dx dy start end index`, 2 `p1 p2 index`.
fn trim(r: &mut Reader, h: &mut Hatcher) {
    match r.integer() {
        0 => {
            let l = r.lin2d();
            let index = r.integer();
            attempt(|| h.trim(l, index));
        }
        1 => {
            let l = r.lin2d();
            let (s, e) = (r.real(), r.real());
            let index = r.integer();
            attempt(|| h.trim_range(l, s, e, index));
        }
        _ => {
            let (p1, p2) = (r.pnt2d(), r.pnt2d());
            let index = r.integer();
            attempt(|| h.trim_segment(p1, p2, index));
        }
    }
}

// The intervals of line `i`, queried from J = 0 to NbIntervals + 1.
fn line_report(h: &Hatcher, i: i32) {
    show(h.line(i), lin2d);
    show(h.line_form(i), |form| int(form as i32));
    show(h.is_x_line(i), flag);
    show(h.is_y_line(i), flag);
    show(h.coordinate(i), real);
    let mut n = -1;
    show(h.nb_intervals_on(i), |count| {
        n = count;
        int(count);
    });
    for j in 0..=n + 1 {
        show(h.start(i, j), real);
        show(h.start_index(i, j), index_par2);
        show(h.end(i, j), real);
        show(h.end_index(i, j), index_par2);
    }
}

fn index_par2((index, par2): (i32, f64)) {
    int(index);
    real(par2);
}

/// `tol oriented nlines lines... ntrims trims...`: the hatcher state after the trimmings,
/// line by line from index 0 to NbLines + 1.
pub fn hatcher_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tol = r.real();
    let mut h = Hatcher::new(tol, r.flag());
    for _ in 0..r.integer() {
        add_line(&mut r, &mut h);
    }
    for _ in 0..r.integer() {
        trim(&mut r, &mut h);
    }
    real(h.tolerance());
    h.set_tolerance(tol * 2.0);
    real(h.tolerance());
    h.set_tolerance(tol);
    int(h.nb_lines());
    int(h.nb_intervals());
    for i in 0..=h.nb_lines() + 1 {
        line_report(&h, i);
    }
    end();
}
