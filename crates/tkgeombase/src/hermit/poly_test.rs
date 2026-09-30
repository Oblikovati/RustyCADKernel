// Rust translation of ModelingData/TKGeomBase/Hermit/Hermit.cxx (PolyTest, Polemax,
// SignDenom) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PolyTest`: the knots to insert into the Hermite reparametrization a(u) so that
//! it keeps a constant sign and its poles stay within the tolerance ratio.
use super::HermitSpline;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;

/// The knots `U4` and `U5` found by one test (0 and 1 when none is needed).
pub(super) struct PolyKnots {
    pub(super) u4: f64,
    pub(super) u5: f64,
}

// The loop state of one test: its knots, the poles and the marks.
struct Test<'a> {
    knots: Array1<'static, f64>,
    // The native I1 and I3: the LocateU indices of Ux and Uy, reused as the hints
    // (in and out) of every LocateParameter call.
    hints: [i32; 2],
    poles: [f64; 4],
    nb_knots: i32,
    boucle: &'a mut i32,
    ux: f64,
    uy: f64,
}

fn impossible_tolerance() -> Failure {
    Failure::with_type("Standard_DimensionError", "Hermit Impossible Tolerance", "")
}

/// `PolyTest(Herm, BS, U4, U5, boucle, TolPoles, TolKnots, Ux, Uy)`: the knots to
/// insert, from the Hermite coefficients `herm` and the knots of the *original*
/// `bs` (plus `Ux`/`Uy` when they are not knots of it);
/// Standard_DimensionError ("Hermit Impossible Tolerance") when the end poles
/// cannot fit the ratio.
pub(super) fn poly_test<S: HermitSpline>(
    herm: [f64; 4],
    bs: &S,
    boucle: &mut i32,
    tol_poles: f64,
    (ux, uy): (f64, f64),
) -> Result<PolyKnots, Failure> {
    let (knots, hints) = test_knots(bs, ux, uy)?;
    let mut test = Test {
        knots,
        hints,
        poles: [
            herm[0],
            herm[0] + herm[1] / 3.0,
            herm[3] - herm[2] / 3.0,
            herm[3],
        ],
        nb_knots: bs.hermit_nb_knots(),
        boucle,
        ux,
        uy,
    };
    let mut dercas: bool = false;
    if tol_poles != 0.0 {
        dercas = test.tolerance_shift(tol_poles)?;
    }
    if test.poles[0].is_nan() || test.poles[0] < 0.0 {
        test.poles = test.poles.map(|pole| -pole);
    }
    test.positivity(dercas)
}

// The knots of `bs` completed with Ux and Uy when they fall inside a span
// (LocateU with a null tolerance gives two different indices), sorted, with the
// first LocateU indices I1 and I3.
fn test_knots<S: HermitSpline>(
    bs: &S,
    ux: f64,
    uy: f64,
) -> Result<(Array1<'static, f64>, [i32; 2]), Failure> {
    let (mut i1, mut i2, mut i3, mut i4): (i32, i32, i32, i32) = (0, 0, 0, 0);
    if ux != 1.0 {
        (i1, i2) = bs.hermit_locate_u(ux)?;
        if uy != 0.0 {
            (i3, i4) = bs.hermit_locate_u(uy)?;
        }
    }
    let mut knots: Vec<f64> = (1..=bs.hermit_nb_knots())
        .map(|i| bs.hermit_knot(i))
        .collect::<Result<_, _>>()?;
    let add_uy: bool = !(i3 == i4 || i3 == 0);
    if i1 != i2 {
        knots.push(ux);
    }
    if add_uy {
        knots.push(uy);
    }
    knots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, knots.len() as i32)
        .map_err(|error| Failure::new(&error.to_string()))?;
    for (index, knot) in knots.into_iter().enumerate() {
        array[index as i32 + 1] = knot;
    }
    Ok((array, [i1, i3]))
}

// Polemax: the indices of the first minimum and maximum ordinates.
fn pole_extremes(poles: &[f64; 4]) -> (usize, usize) {
    let (mut min, mut max): (usize, usize) = (0, 0);
    let (mut min_value, mut max_value): (f64, f64) = (poles[0], poles[0]);
    for (i, pole) in poles.iter().enumerate().skip(1) {
        if *pole < min_value {
            min_value = *pole;
            min = i;
        }
        if *pole > max_value {
            max_value = *pole;
            max = i;
        }
    }
    (min, max)
}

impl Test<'_> {
    fn shift(&mut self, amount: f64) {
        self.poles = self.poles.map(|pole| pole - amount);
    }

    // The tolerance loop: shift the poles down when their ratio exceeds 1/Tol;
    // returns `dercas` (the first loop shifted by the geometric mean).
    fn tolerance_shift(&mut self, tol: f64) -> Result<bool, Failure> {
        let (min, max) = pole_extremes(&self.poles);
        let pole_min: f64 = self.poles[min];
        let pole_max: f64 = self.poles[max];
        if !((pole_max >= (1.0 / tol) * pole_min)
            || ((pole_min == 0.0) && (pole_max >= (1.0 / tol))))
        {
            return Ok(false);
        }
        let [p0, _, _, p3] = self.poles;
        if p0 >= (1.0 / tol) * p3 || p0 <= tol * p3 {
            return Err(impossible_tolerance());
        }
        if max == 0 || max == 3 {
            self.shift(tol * pole_max);
            return Ok(false);
        }
        if min == 0 || min == 3 {
            self.shift((1.0 / tol) * pole_min);
            return Ok(false);
        }
        Ok(self.interior_shift(tol, pole_min, pole_max))
    }

    // Both extremes are interior poles: shift by Tol*max, else by min/Tol, else by
    // the geometric-mean rule of the loop mark.
    fn interior_shift(&mut self, tol: f64, pole_min: f64, pole_max: f64) -> bool {
        let mut mark: bool = false;
        if (tol * pole_max < self.poles[0]) && (tol * pole_max < self.poles[3]) {
            self.shift(tol * pole_max);
            mark = true;
        }
        if (1.0 / tol * pole_min > self.poles[0]) && (1.0 / tol * pole_min > self.poles[3]) && !mark
        {
            self.shift(1.0 / tol * pole_min);
            mark = true;
        }
        if mark {
            return false;
        }
        let (pole0, pole3): (f64, f64) = (self.poles[0], self.poles[3]);
        let mut dercas: bool = false;
        if pole0 < pole3 {
            dercas |= self.mean_shift((pole3 / pole0).log10(), pole3, pole0, tol);
        }
        if pole0 > pole3 {
            dercas |= self.mean_shift((pole0 / pole3).log10(), pole0, pole3, tol);
        }
        dercas
    }

    // With `a = log10(high / low)`: the second loop shifts by
    // `high * 10^(-log10(Tol)/2 - a/2)`, the first by `low * 10^(a/2 + log10(Tol)/2)`
    // and sets dercas.
    fn mean_shift(&mut self, a: f64, high: f64, low: f64, tol: f64) -> bool {
        if *self.boucle == 2 {
            self.shift(high * 10.0_f64.powf(-0.5 * tol.log10() - a / 2.0));
            return false;
        }
        if *self.boucle == 1 {
            self.shift(low * 10.0_f64.powf(a / 2.0 + 0.5 * tol.log10()));
            return true;
        }
        false
    }

    fn knot(&self, index: i32) -> f64 {
        self.knots[index]
    }

    fn len(&self) -> i32 {
        self.knots.len() as i32
    }

    // BSplCLib::LocateParameter(3, knots, U, false, 1, len, I, U) on the test knots,
    // with the hint I1 (`hint` 0) or I3 (`hint` 1) updated in place.
    fn locate(&mut self, u: f64, hint: usize) -> Result<(i32, f64), Failure> {
        let mut new_u: f64 = u;
        let len: i32 = self.len();
        let index: &mut i32 = &mut self.hints[hint];
        bsplclib::locate_parameter_flat_knots(3, &self.knots, u, false, 1, len, index, &mut new_u)?;
        Ok((*index, new_u))
    }

    // The positivity loop: the knots that keep a(u) positive near either end.
    fn positivity(mut self, dercas: bool) -> Result<PolyKnots, Failure> {
        let mut result = PolyKnots { u4: 0.0, u5: 1.0 };
        let [p0, p1, p2, p3] = self.poles;
        if p1 < 0.0 && p2 >= 0.0 {
            result.u4 = self.left_knot(p0 / (p0 - p1))?;
        }
        if p1 >= 0.0 && p2 < 0.0 {
            result.u5 = self.right_knot(p2 / (p2 - p3))?;
        }
        if dercas {
            *self.boucle += 1;
        }
        if p1 < 0.0 && p2 < 0.0 {
            self.both_negative(p0 / (p0 - p1), p2 / (p2 - p3), &mut result)?;
        }
        Ok(result)
    }

    // A negative second pole only: a knot left of the first sign change.
    fn left_knot(&mut self, mut us1: f64) -> Result<f64, Failure> {
        if *self.boucle == 2 {
            us1 *= self.knot(2);
        }
        if *self.boucle == 1 && self.ux != 0.0 {
            us1 *= self.ux;
        }
        let (i1, us1) = self.locate(us1, 0)?;
        Ok(if i1 < 2 { us1 } else { self.knot(i1) })
    }

    // A negative third pole only: a knot right of the last sign change.
    fn right_knot(&mut self, mut us2: f64) -> Result<f64, Failure> {
        let len: i32 = self.len();
        if *self.boucle == 2 {
            us2 = self.knot(len - 1) + us2 * (1.0 - self.knot(len - 1));
        }
        if *self.boucle == 1 && self.ux != 0.0 {
            us2 = self.uy + us2 * (1.0 - self.uy);
        }
        let (i1, us2) = self.locate(us2, 0)?;
        Ok(if i1 >= len - 1 {
            us2
        } else {
            self.knot(i1 + 1)
        })
    }

    // Both inner poles negative: one knot for both changes, or one on each side.
    fn both_negative(
        &mut self,
        mut us1: f64,
        mut us2: f64,
        result: &mut PolyKnots,
    ) -> Result<(), Failure> {
        if *self.boucle != 0 && self.ux != 0.0 {
            us1 *= self.ux;
            us2 = self.uy + us2 * (1.0 - self.uy);
        }
        if us2 <= us1 {
            return self.one_knot(us1, us2, result);
        }
        let (i1, us1) = self.locate(us1, 0)?;
        result.u4 = if i1 >= 2 { self.knot(i1) } else { us1 };
        let (i3, us2) = self.locate(us2, 1)?;
        result.u5 = if i3 < self.nb_knots - 1 {
            self.knot(i3 + 1)
        } else {
            us2
        };
        Ok(())
    }

    // Both sign changes in one span order: one knot for both when a knot separates
    // them, else a knot on each side, else their midpoint.
    fn one_knot(&mut self, us1: f64, us2: f64, result: &mut PolyKnots) -> Result<(), Failure> {
        let (i1, us1) = self.locate(us1, 0)?;
        if self.knot(i1) >= us2 {
            result.u4 = self.knot(i1);
            return Ok(());
        }
        let mut cas: bool = false;
        let mut us2: f64 = us2;
        if i1 >= 2 {
            result.u4 = self.knot(i1);
            let i3: i32;
            (i3, us2) = self.locate(us2, 1)?;
            if i3 < self.nb_knots - 1 {
                result.u5 = self.knot(i3 + 1);
                cas = true;
            }
        }
        if !cas {
            result.u4 = (us1 + us2) / 2.0;
        }
        Ok(())
    }
}
