//! TopTrans probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_toptrans.cxx.
use super::common::{Reader, attempt, end, int};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::toptrans::{CurveTransition, SurfaceTransition, get_after, get_before};

fn orientation(r: &mut Reader) -> Orientation {
    Orientation::ALL[r.integer() as usize]
}
fn states(before: State, after: State) {
    int(before as i32);
    int(after as i32);
}

/// Commands on one curve transition, the states after each.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut transition = CurveTransition::new();
    for _ in 0..r.integer() {
        match r.integer() {
            0 => {
                let (t, n) = (r.dir(), r.dir());
                transition.reset(t, n, r.real());
            }
            1 => transition.reset_line(r.dir()),
            _ => {
                let (tole, t, n, c) = (r.real(), r.dir(), r.dir(), r.real());
                let s = orientation(&mut r);
                transition.compare(tole, t, n, c, s, orientation(&mut r));
            }
        }
        states(transition.state_before(), transition.state_after());
    }
    end();
}

fn surface_command(r: &mut Reader, transition: &mut SurfaceTransition, command: i32) {
    if command == 0 {
        let (t, n) = (r.dir(), r.dir());
        return attempt(|| transition.reset_line(t, n));
    }
    if command == 1 {
        let (t, n, mx, mn, cx, cn) = (r.dir(), r.dir(), r.dir(), r.dir(), r.real(), r.real());
        return attempt(|| transition.reset(t, n, mx, mn, cx, cn));
    }
    let (tole, n) = (r.real(), r.dir());
    if command == 2 {
        let (s, o) = (orientation(r), orientation(r));
        return attempt(|| transition.compare_plane(tole, n, s, o));
    }
    let (mx, mn, cx, cn) = (r.dir(), r.dir(), r.real(), r.real());
    let (s, o) = (orientation(r), orientation(r));
    attempt(|| transition.compare(tole, n, mx, mn, cx, cn, s, o));
}

/// Commands on one surface transition, the exception and the states after each.
pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut transition = SurfaceTransition::new();
    for _ in 0..r.integer() {
        let command = r.integer();
        surface_command(&mut r, &mut transition, command);
        states(transition.state_before(), transition.state_after());
    }
    end();
}

/// `orientation`: the static state tables.
pub fn states_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let o = orientation(&mut r);
    states(get_before(o), get_after(o));
    end();
}
