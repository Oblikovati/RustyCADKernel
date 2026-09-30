// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (InsertKnots, static Copy) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot insertion on poles of any dimension by the Boor scheme.
//!
//! The native code slices the output arrays into views while it fills them;
//! [`super::views`] reproduces those views over the same storage.
use super::build::{build_boor, build_knots, get_pole};
use super::native::{epsilon, std_max};
use super::scheme::boor_scheme;
use super::views::{prefix_view, prefix_view_mut};
use crate::plib::access::{array_cell, array_value, non_negative};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const INSERT: &str = "BSplCLib::InsertKnots";

/// Input curve of an insertion; `InsertInput { degree, periodic, dimension, poles, knots, mults }`.
pub(crate) struct InsertInput<'r, 'a> {
    pub(crate) degree: i32,
    pub(crate) periodic: bool,
    pub(crate) dimension: i32,
    pub(crate) poles: &'r Array1<'a, f64>,
    pub(crate) knots: &'r Array1<'a, f64>,
    pub(crate) mults: &'r Array1<'a, i32>,
}

/// Inserts `add_knots` (with `add_mults`, or multiplicity 1 each) into a curve whose
/// poles hold `dimension` interleaved values (`BSplCLib::InsertKnots`).
///
/// The new arrays must be sized by [`prepare_insert_knots`](super::prepare_insert_knots).
/// Multiplicities never exceed the degree; with `add` false the maximum of the old
/// and requested multiplicity is kept. Quirk kept: the first new pole of each scheme
/// is written from `NewKnots.Lower() + (index+1)*Dimension`, i.e. the knot array's
/// lower bound is used as a pole index;
/// `bsplclib::insert_knots_flat(3, false, 3, &p, &k, &m, &add, None, &mut np, &mut nk, &mut nm, 0.0, true)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native thirteen-argument signature.
pub fn insert_knots_flat(
    degree: i32,
    periodic: bool,
    dimension: i32,
    poles: &Array1<'_, f64>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    add_knots: &Array1<'_, f64>,
    add_mults: Option<&Array1<'_, i32>>,
    new_poles: &mut Array1<'_, f64>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    tolerance: f64,
    add: bool,
) -> Result<(), Failure> {
    let input = InsertInput {
        degree,
        periodic,
        dimension,
        poles,
        knots,
        mults,
    };
    let deg: usize = non_negative(INSERT, "Degree", degree)?;
    let dim: usize = non_negative(INSERT, "Dimension", dimension)?;
    let mut state = Insertion {
        input: &input,
        new_poles,
        new_knots,
        new_mults,
        lk: vec![0.0; 2 * deg],
        lp: vec![0.0; (2 * deg + 1) * dim],
        curk: knots.lower() - 1,
        curnk: 0,
        curp: poles.lower(),
        curnp: 0,
        index: 0,
        firstmult: 0,
    };
    state.curnk = state.new_knots.lower() - 1;
    state.curnp = state.new_poles.lower();
    state.index = if periodic {
        -array_value(mults, mults.lower())?
    } else {
        -degree - 1
    };
    for kn in add_knots.lower()..=add_knots.upper() {
        state.insert_one(add_knots, add_mults, kn, tolerance, add)?;
    }
    state.copy_tail()?;
    state.process_first_last()
}

struct Insertion<'r, 'a, 'p, 'k, 'm> {
    input: &'r InsertInput<'r, 'a>,
    new_poles: &'r mut Array1<'p, f64>,
    new_knots: &'r mut Array1<'k, f64>,
    new_mults: &'r mut Array1<'m, i32>,
    lk: Vec<f64>,
    lp: Vec<f64>,
    curk: i32,
    curnk: i32,
    curp: i32,
    curnp: i32,
    index: i32,
    // Multiplicity of the first-last knot of a periodic curve, inserted at the end.
    firstmult: i32,
}

impl Insertion<'_, '_, '_, '_, '_> {
    fn insert_one(
        &mut self,
        add_knots: &Array1<'_, f64>,
        add_mults: Option<&Array1<'_, i32>>,
        kn: i32,
        tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let u: f64 = array_value(add_knots, kn)?;
        let eps: f64 = std_max(tolerance, epsilon(u));
        self.copy_knots_up_to(u, eps)?;
        let view_upper: i32 = self.curnk + self.input.knots.upper() - self.curk;
        self.copy_scheme_knots(view_upper)?;
        let sameknot: bool = (u - array_value(self.new_knots, self.curnk)?).abs() <= eps;
        let degree: i32 = self.input.degree;
        let length: i32 = if sameknot {
            0.max(degree - array_value(self.new_mults, self.curnk)?)
        } else {
            degree
        };
        let requested: i32 = match add_mults {
            None => 1,
            Some(add_mults) => degree.min(array_value(add_mults, kn)?),
        };
        let depth: i32 = if sameknot {
            self.same_knot_depth(requested, add)?
        } else {
            requested
        };
        if depth <= 0 {
            return Ok(());
        }
        self.run_scheme(u, view_upper, length, depth)?;
        self.index += depth;
        if sameknot {
            *array_cell(self.new_mults, self.curnk)? += depth;
        } else {
            self.curnk += 1;
            *array_cell(self.new_knots, self.curnk)? = u;
            *array_cell(self.new_mults, self.curnk)? = depth;
        }
        Ok(())
    }

    // Finds the position in the old knots and copies them to the new knots.
    fn copy_knots_up_to(&mut self, u: f64, eps: f64) -> Result<(), Failure> {
        let knots: &Array1<'_, f64> = self.input.knots;
        while self.curk < knots.upper() && array_value(knots, self.curk + 1)? - u <= eps {
            self.curk += 1;
            self.curnk += 1;
            *array_cell(self.new_knots, self.curnk)? = array_value(knots, self.curk)?;
            let mult: i32 = array_value(self.input.mults, self.curk)?;
            *array_cell(self.new_mults, self.curnk)? = mult;
            self.index += mult;
        }
        Ok(())
    }

    // Copies enough knots after (and, periodically, before) the insertion point into
    // the view `NewKnots(Lower..view_upper)` for the insertion schema.
    fn copy_scheme_knots(&mut self, view_upper: i32) -> Result<(), Failure> {
        let knots: &Array1<'_, f64> = self.input.knots;
        let degree: i32 = self.input.degree;
        let (mut k, mut i, mut mult): (i32, i32, i32) = (self.curk, self.curnk, 0);
        while mult < degree && k < knots.upper() {
            k += 1;
            i += 1;
            mult += self.write_view_knot(view_upper, i, k)?;
        }
        if !self.input.periodic {
            return Ok(());
        }
        (mult, k, i) = (0, knots.upper(), view_upper);
        while mult < degree && i > self.curnk {
            mult += self.write_view_knot(view_upper, i, k)?;
            k -= 1;
            i -= 1;
        }
        let first: i32 = array_value(self.new_mults, self.new_mults.lower())?;
        view_check(self.new_mults.lower(), view_upper, view_upper)?;
        *array_cell(self.new_mults, view_upper)? = first;
        Ok(())
    }

    fn write_view_knot(&mut self, view_upper: i32, i: i32, k: i32) -> Result<i32, Failure> {
        view_check(self.new_knots.lower(), view_upper, i)?;
        *array_cell(self.new_knots, i)? = array_value(self.input.knots, k)?;
        let mult: i32 = array_value(self.input.mults, k)?;
        *array_cell(self.new_mults, i)? = mult;
        Ok(mult)
    }

    fn same_knot_depth(&mut self, requested: i32, add: bool) -> Result<i32, Failure> {
        let current: i32 = array_value(self.new_mults, self.curnk)?;
        let mut depth: i32 = requested;
        if add {
            if current + depth > self.input.degree {
                depth = self.input.degree - current;
            }
        } else {
            depth = 0.max(depth - current);
        }
        let knots: &Array1<'_, f64> = self.input.knots;
        if self.input.periodic && (self.curk == knots.lower() || self.curk == knots.upper()) {
            // The first and last knot of a periodic curve are delayed to the end, once.
            if self.firstmult == 0 {
                self.firstmult += depth;
            }
            depth = 0;
        }
        Ok(depth)
    }

    fn run_scheme(
        &mut self,
        u: f64,
        view_upper: i32,
        length: i32,
        depth: i32,
    ) -> Result<(), Failure> {
        let (degree, dimension) = (self.input.degree, self.input.dimension);
        {
            let nknots = prefix_view(INSERT, self.new_knots, view_upper)?;
            let nmults = prefix_view(INSERT, self.new_mults, view_upper)?;
            build_knots(
                degree,
                self.curnk,
                self.input.periodic,
                &nknots,
                Some(&nmults),
                &mut self.lk,
            )?;
        }
        let poles: &Array1<'_, f64> = self.input.poles;
        let need: i32 = (self.new_poles.lower() + (self.index + length + 1) * dimension
            - self.curnp)
            .min(poles.upper() - self.curp + 1);
        let (mut p, mut np): (i32, i32) = (self.curp, self.curnp);
        copy_poles(need, &mut p, poles, &mut np, self.new_poles)?;
        self.curp += need;
        self.curnp += need;
        {
            let npoles = prefix_view(INSERT, self.new_poles, self.curnp - 1)?;
            build_boor(self.index, length, dimension, &npoles, &mut self.lp)?;
        }
        boor_scheme(u, degree, &self.lk, dimension, &mut self.lp, depth, length)?;
        self.curnp += depth * dimension;
        let mut the_poles = prefix_view_mut(INSERT, self.new_poles, self.curnp - 1)?;
        let mut np: i32 = self.new_knots.lower() + (self.index + 1) * dimension;
        for i in 1..=length + depth {
            get_pole(
                i,
                length,
                depth,
                dimension,
                &self.lp,
                &mut np,
                &mut the_poles,
            )?;
        }
        Ok(())
    }

    // Copies the last poles and knots.
    fn copy_tail(&mut self) -> Result<(), Failure> {
        let poles: &Array1<'_, f64> = self.input.poles;
        copy_poles(
            poles.upper() - self.curp + 1,
            &mut self.curp,
            poles,
            &mut self.curnp,
            self.new_poles,
        )?;
        let (knots, mults) = (self.input.knots, self.input.mults);
        while self.curk < knots.upper() {
            self.curk += 1;
            self.curnk += 1;
            *array_cell(self.new_knots, self.curnk)? = array_value(knots, self.curk)?;
            *array_cell(self.new_mults, self.curnk)? = array_value(mults, self.curk)?;
        }
        Ok(())
    }

    // Inserts the delayed first-last knot of a periodic curve, rotating the new poles.
    fn process_first_last(&mut self) -> Result<(), Failure> {
        if self.firstmult <= 0 {
            return Ok(());
        }
        let degree: i32 = self.input.degree;
        let curnk: i32 = self.new_knots.lower();
        let current: i32 = array_value(self.new_mults, curnk)?;
        if current + self.firstmult > degree {
            self.firstmult = degree - current;
        }
        if self.firstmult <= 0 {
            return Ok(());
        }
        let (length, depth): (i32, i32) = (degree - current, self.firstmult);
        self.first_last_scheme(curnk, length, depth)?;
        self.rotate_first_last(length, depth)?;
        let (lower, upper): (i32, i32) = (self.new_mults.lower(), self.new_mults.upper());
        *array_cell(self.new_mults, lower)? += depth;
        *array_cell(self.new_mults, upper)? += depth;
        Ok(())
    }

    fn first_last_scheme(&mut self, curnk: i32, length: i32, depth: i32) -> Result<(), Failure> {
        let dimension: i32 = self.input.dimension;
        build_knots(
            self.input.degree,
            curnk,
            self.input.periodic,
            self.new_knots,
            Some(self.new_mults),
            &mut self.lk,
        )?;
        {
            let upper: i32 = self.new_poles.upper() - depth * dimension;
            let npoles = prefix_view(INSERT, self.new_poles, upper)?;
            build_boor(0, length, dimension, &npoles, &mut self.lp)?;
        }
        let u: f64 = array_value(self.new_knots, curnk)?;
        boor_scheme(
            u,
            self.input.degree,
            &self.lk,
            dimension,
            &mut self.lp,
            depth,
            length,
        )
    }

    // Copies the new poles, rotated by depth.
    fn rotate_first_last(&mut self, length: i32, depth: i32) -> Result<(), Failure> {
        let dimension: i32 = self.input.dimension;
        let mut np: i32 = self.new_poles.lower();
        for i in depth..length + depth {
            get_pole(
                i,
                length,
                depth,
                dimension,
                &self.lp,
                &mut np,
                self.new_poles,
            )?;
        }
        np = self.new_poles.upper() - depth * dimension + 1;
        for i in 0..depth {
            get_pole(
                i,
                length,
                depth,
                dimension,
                &self.lp,
                &mut np,
                self.new_poles,
            )?;
        }
        Ok(())
    }
}

// Rejects a write outside the native view `Array(lower..upper)`.
fn view_check(lower: i32, upper: i32, index: i32) -> Result<(), Failure> {
    if index < lower || index > upper {
        return Err(Failure::out_of_range(&format!(
            "{INSERT}: index {index} outside the view {lower}..{upper}"
        )));
    }
    Ok(())
}

/// The native static `Copy`: copies `nb_poles` values cyclically after reducing both
/// start indices modulo their array length; `copy_poles(3, &mut p, &old, &mut np, &mut new)?`.
pub(crate) fn copy_poles(
    nb_poles: i32,
    old_first: &mut i32,
    old_poles: &Array1<'_, f64>,
    new_first: &mut i32,
    new_poles: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    *old_first = wrap_start(*old_first, old_poles.lower(), old_poles.len())?;
    *new_first = wrap_start(*new_first, new_poles.lower(), new_poles.len())?;
    for _ in 1..=nb_poles {
        *array_cell(new_poles, *new_first)? = array_value(old_poles, *old_first)?;
        *old_first += 1;
        if *old_first > old_poles.upper() {
            *old_first = old_poles.lower();
        }
        *new_first += 1;
        if *new_first > new_poles.upper() {
            *new_first = new_poles.lower();
        }
    }
    Ok(())
}

fn wrap_start(first: i32, lower: i32, length: usize) -> Result<i32, Failure> {
    if length == 0 {
        return Err(Failure::out_of_range(&format!(
            "BSplCLib Copy: empty pole array with lower bound {lower}; expected at least one value"
        )));
    }
    Ok(lower + (first - lower) % length as i32)
}
