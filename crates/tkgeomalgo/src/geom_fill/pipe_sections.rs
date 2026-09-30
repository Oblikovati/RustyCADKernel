// Rust translation of the static CheckSense and the Init members with several
// sections of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Pipe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pipes through several sections: each section placed on the path, oriented
//! like the first one, and skinned by a `GeomFill_NSections` law.
use super::corrected_frenet::GeomFillCorrectedFrenet;
use super::curve_and_trihedron::GeomFillCurveAndTrihedron;
use super::nsections::GeomFillNSections;
use super::pipe::GeomFillPipe;
use super::pipe_error::GeomFillPipeError;
use super::section_placement::GeomFillSectionPlacement;
use super::sweep_failures::{construction_error, live};
use super::trihedron_law::GeomFillTrihedronLaw;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::geom_lib::axe_of_inertia;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// The number of points sampled on each section.
const NP: i32 = 21;

/// The turning of a section: its inertia centre and axis, and the signed angles of
/// three consecutive chords near its middle seen from the centre.
struct SectionTurn {
    no_sing: bool,
    angles: [f64; 3],
}

fn live_curve(c: &Handle<dyn GeomCurve>) -> Result<&dyn GeomCurve, Failure> {
    live(c, "GeomFill_Pipe CheckSense", "C")
}

/// `NP` points of `c` from its start by `h`, the last ones clamped to its end.
fn sample_section(c: &dyn GeomCurve, h: f64) -> Result<Array1<'static, Point3>, Failure> {
    let (f, l) = (c.first_parameter(), c.last_parameter());
    let mut tab: Array1<'static, Point3> = one_based(NP)?;
    let mut u = f;
    for ip in 1..=NP {
        put(&mut tab, ip, c.value(u)?)?;
        u += h;
        if (u - f) * (u - l) > 0.0 {
            u = l;
        }
    }
    Ok(tab)
}

/// The sampled points of `c`, their inertia axis, and the angles of the chords
/// around `(f + l - h) / 2` seen from the inertia centre about `axe_ref` (the
/// curve's own axis when `None`).
fn section_turn(
    c: &dyn GeomCurve,
    h: f64,
    axe_ref: Option<Direction3>,
) -> Result<(SectionTurn, Direction3), Failure> {
    let inertia = axe_of_inertia(&sample_section(c, h)?, 1.0e-7)?;
    let (pos, own) = (inertia.axe.location(), inertia.axe.direction());
    let axis = Vector3::from(axe_ref.unwrap_or(own));
    let mut u = (c.first_parameter() + c.last_parameter() - h) / 2.0 - h;
    let mut p1 = c.value(u)?;
    let mut angles = [0.0; 3];
    for angle in &mut angles {
        u += h;
        let p2 = c.value(u)?;
        *angle = Vector3::between(pos, p1).angle_with_ref(Vector3::between(pos, p2), axis)?;
        p1 = p2;
    }
    let turn = SectionTurn {
        no_sing: !inertia.is_singular,
        angles,
    };
    Ok((turn, own))
}

/// Whether a section turns like the first one, from the signs of the products of
/// their chord angles that are not null.
fn same_sense(alpha: &[f64; 3], beta: &[f64; 3]) -> bool {
    let conf = precision::confusion();
    let pasnul: Vec<bool> = (0..3)
        .map(|i| alpha[i].abs() > conf && beta[i].abs() > conf)
        .collect();
    let pos = |i: usize| alpha[i] * beta[i] > 0.0;
    match (pasnul[0], pasnul[1], pasnul[2]) {
        (true, true, true) if pos(0) => pos(1) || pos(2),
        (true, true, true) => pos(1) && pos(2),
        (true, true, false) => pos(0) || pos(1),
        (true, false, true) => pos(0) || pos(2),
        (false, true, true) => pos(1) || pos(2),
        (true, false, false) => pos(0),
        (false, true, false) => pos(1),
        (false, false, true) => pos(2),
        (false, false, false) => true,
    }
}

/// The static `CheckSense(Seq1, Seq2)`: reverses in place each section that turns
/// against the first one (while no section is singular); false when a section is
/// singular. The native second sequence holds the same curves, so it is not built.
pub(crate) fn check_sense(seq: &[Handle<dyn GeomCurve>]) -> Result<bool, Failure> {
    let first = seq.first().ok_or_else(|| {
        construction_error("GeomFill_Pipe CheckSense: no section; expected one at least")
    })?;
    let c1 = live_curve(first)?;
    let h = (c1.first_parameter() - c1.last_parameter()).abs() / 20.0;
    let (turn1, axe_ref) = section_turn(c1, h, None)?;
    let mut no_sing = turn1.no_sing;
    for c in &seq[1..] {
        let c2 = live_curve(c)?;
        let (turn2, _) = section_turn(c2, h, Some(axe_ref))?;
        no_sing &= turn2.no_sing;
        if no_sing && !same_sense(&turn1.angles, &turn2.angles) {
            c2.reverse()?;
        }
    }
    Ok(no_sing)
}

impl GeomFillPipe {
    /// The corrected Frenet law on a copy of the path; false when the law refuses
    /// the path.
    fn corrected_frenet_law(&mut self, path: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
        let copy: Handle<dyn GeomCurve> = live(path, "GeomFill_Pipe::Init", "Path")?.copy()?.cast();
        self.my_adp_path = Handle::new(rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve::with_curve(
            &copy,
        )?)
        .cast();
        let t_law: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillCorrectedFrenet::new()).cast();
        self.my_loc = Handle::new(GeomFillCurveAndTrihedron::new(&t_law)).cast();
        live(&self.my_loc, "GeomFill_Pipe::Init", "myLoc")?.set_curve(&self.my_adp_path)
    }

    /// `Init(Path, NSections)`: the sections placed on the path, oriented, sorted
    /// by path parameter (Standard_ConstructionError for two at the same place) and
    /// skinned; `pipe.init_sections(&path, &sections)?`.
    pub fn init_sections(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        n_sections: &[Handle<dyn GeomCurve>],
    ) -> Result<(), Failure> {
        (self.my_type, self.my_error, self.my_radius) = (3, 0.0, 0.0);
        self.corrected_frenet_law(path)?;
        let (mut seq_c, mut seq_p) = (Vec::new(), Vec::new());
        for section in n_sections {
            let mut place = GeomFillSectionPlacement::new(&self.my_loc, &section.clone().cast())?;
            place.perform(precision::confusion())?;
            seq_p.push(place.parameter_on_path());
            seq_c.push(place.section(false)?);
        }
        check_sense(&seq_c)?;
        sort_by_parameter(&mut seq_p, &mut seq_c);
        if seq_p
            .windows(2)
            .any(|w| (w[1] - w[0]).abs() < precision::p_confusion())
        {
            return Err(construction_error(
                "GeomFill_Pipe::Init with NSections : invalid parameters",
            ));
        }
        self.set_nsections(path, &seq_c, &seq_p)
    }

    /// `mySec`: the N-section law over the first section's range and the path range.
    fn set_nsections(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        seq_c: &[Handle<dyn GeomCurve>],
        seq_p: &[f64],
    ) -> Result<(), Failure> {
        let p = live(path, "GeomFill_Pipe::Init", "Path")?;
        let (first, last) = (p.first_parameter(), p.last_parameter());
        let c = live_curve(
            seq_c
                .first()
                .ok_or_else(|| construction_error("GeomFill_Pipe::Init: no section"))?,
        )?;
        let (deb, fin) = (c.first_parameter(), c.last_parameter());
        let law = GeomFillNSections::with_domain(seq_c, seq_p, (deb, fin), (first, last))?;
        self.my_sec = Handle::new(law).cast();
        Ok(())
    }

    /// `Init(Path, FirstSect, LastSect)`: both sections placed at the path start
    /// and skinned over the whole path (status `GeomFill_ImpossibleContact` when
    /// the law refuses the path); `pipe.init_two_sections(&path, &first, &last)?`.
    pub fn init_two_sections(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        last_sect: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        (self.my_type, self.my_error, self.my_radius) = (3, 0.0, 0.0);
        let p = live(path, "GeomFill_Pipe::Init", "Path")?;
        let (first, last) = (p.first_parameter(), p.last_parameter());
        if !self.corrected_frenet_law(path)? {
            self.my_status = GeomFillPipeError::ImpossibleContact;
            return Ok(());
        }
        let mut seq_c = Vec::with_capacity(2);
        for sect in [first_sect, last_sect] {
            let mut pl = GeomFillSectionPlacement::new(&self.my_loc, &sect.clone().cast())?;
            pl.perform_at(first, precision::confusion())?;
            seq_c.push(pl.section(false)?);
        }
        check_sense(&seq_c)?;
        self.set_nsections(path, &seq_c, &[first, last])
    }
}

/// The native exchange sort of the sections by path parameter: every later
/// smaller parameter is swapped in, until a pass swaps nothing.
fn sort_by_parameter(seq_p: &mut [f64], seq_c: &mut [Handle<dyn GeomCurve>]) {
    let mut play_again = true;
    while play_again {
        play_again = false;
        for i in 0..seq_p.len() {
            for j in i..seq_p.len() {
                if seq_p[i] > seq_p[j] {
                    seq_p.swap(i, j);
                    seq_c.swap(i, j);
                    play_again = true;
                }
            }
        }
    }
}
