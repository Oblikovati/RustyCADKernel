//! IntPatch_PrmPrmIntersection probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_prmprm.cxx: the grid members on a bit map and
//! the start points of PointDepart.
use super::common::{Reader, attempt, end, flag, int};
use super::ics_geometry::surface;
use super::int_walk::pnt2s;
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchPrmPrmIntersection, IntPatchPrmPrmIntersectionT3Bits,
};
use rustycad_tkgeomalgo::int_surf::IntSurfLineOn2S;

/// `ga_pp_cells`.
fn cells(m: &IntPatchPrmPrmIntersectionT3Bits, limit: usize) {
    let mut n: i32 = 0;
    let mut first: Vec<i32> = Vec::new();
    for t in 0..128 * 128 * 128 {
        if m.val(t) != 0 {
            n += 1;
            if first.len() < limit {
                first.push(t);
            }
        }
    }
    int(n);
    first.into_iter().for_each(int);
}

fn ints<const N: usize>(r: &mut Reader) -> [i32; N] {
    std::array::from_fn(|_| r.integer())
}

/// One command of `ga_ip_ppgrid_case`.
fn grid_command(
    r: &mut Reader,
    pp: &IntPatchPrmPrmIntersection,
    m: &mut IntPatchPrmPrmIntersectionT3Bits,
    c: i32,
) {
    match c {
        1 => {
            let [a, b, d] = ints::<3>(r);
            int(pp.grille_integer(a, b, d));
        }
        2 => {
            let (a, b, d) = pp.integer_grille(r.integer());
            [a, b, d].into_iter().for_each(int);
        }
        3 => int(pp.dans_grille(r.integer())),
        4 => int(pp.nb_points_grille()),
        5 => {
            let v: [f64; 9] = std::array::from_fn(|_| r.real());
            int(pp.code_reject([v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]));
        }
        6 => {
            let v = ints::<6>(r);
            pp.remplit_lin([v[0], v[1], v[2]], [v[3], v[4], v[5]], m);
        }
        7 => {
            let v = ints::<9>(r);
            pp.remplit_tri(
                [v[0], v[1], v[2]],
                [v[3], v[4], v[5]],
                [v[6], v[7], v[8]],
                m,
            );
        }
        _ => {
            let [a, b, d] = ints::<3>(r);
            pp.remplit(a, b, d, m);
        }
    }
}

/// `ga_ip_ppgrid_case`.
pub fn ppgrid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let pp = IntPatchPrmPrmIntersection::new();
    let mut m = IntPatchPrmPrmIntersectionT3Bits::new(128);
    for _ in 0..r.integer() {
        let c = r.integer();
        grid_command(&mut r, &pp, &mut m, c);
    }
    cells(&m, 40);
    flag(pp.is_done());
    attempt(|| pp.is_empty().map(flag));
    attempt(|| pp.nb_lines().map(int));
    attempt(|| pp.line(1).map(|_| ()));
    end();
}

/// `ga_ip_ppdepart_case`.
pub fn ppdepart_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s1 = surface(&mut r);
        let (su1, sv1) = (r.integer(), r.integer());
        let s2 = surface(&mut r);
        let (su2, sv2) = (r.integer(), r.integer());
        let l: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
        let pp = IntPatchPrmPrmIntersection::new();
        pp.point_depart(&l, &s1, (su1, sv1), &s2, (su2, sv2))?;
        int(l.nb_points());
        for i in 1..=l.nb_points() {
            pnt2s(&l.value(i)?);
        }
        Ok(())
    });
    end();
}
