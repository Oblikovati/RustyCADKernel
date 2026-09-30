// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierSurface.cxx (ExchangeUV, Increase, InsertPole*, RemovePole*, Segment) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The structural edits of Geom_BezierSurface: exchange of the directions, degree
//! elevation, insertion and removal of pole rows and columns, and segmentation.
use super::grid::{add_col, add_row, at1, delete_col, delete_row, one_based_grid, put2};
use super::state::SurfaceState;
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::spline_data::bezier_knots;
use rustycad_tkmath::bsplslib::{self, CacheSource};
use rustycad_tkmath::gp::{Point3, resolution};
use rustycad_tkmath::{bsplclib, plib};

/// Which pole line an insertion or removal addresses: a column (V index) or a row
/// (U index).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Line {
    /// A column of constant V index (`...PoleCol...`).
    Col,
    /// A row of constant U index (`...PoleRow...`).
    Row,
}

fn check_line_index(index: i32, count: i32, member: &str) -> Result<(), Failure> {
    if index < 1 || index > count {
        return Err(errors::out_of_range(&format!(
            "Geom_BezierSurface::{member}: Index = {index}; expected in [1, {count}]"
        )));
    }
    Ok(())
}

fn check_weights_positive(weights: &Array1<'_, f64>, member: &str) -> Result<(), Failure> {
    for index in weights.lower()..=weights.upper() {
        let weight: f64 = at1(weights, index)?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom_BezierSurface::{member}: Weights({index}) = {weight}; expected > gp::Resolution()"
            )));
        }
    }
    Ok(())
}

impl SurfaceState {
    /// `ExchangeUV`: transpose poles and weights and swap the rationality flags. Quirk
    /// kept: the evaluation representation stays.
    pub(super) fn exchange_uv(&mut self) -> Result<(), Failure> {
        let (nu, nv) = (self.nb_u_poles(), self.nb_v_poles());
        let mut npoles: Array2<'static, Point3> = one_based_grid(nv, nu)?;
        let mut nweights: Array2<'static, f64> = one_based_grid(nv, nu)?;
        for i in 1..=nv {
            for j in 1..=nu {
                put2(&mut npoles, i, j, self.poles[(j, i)])?;
                put2(&mut nweights, i, j, self.weights[(j, i)])?;
            }
        }
        self.poles = npoles;
        self.weights = if self.is_rational() {
            nweights
        } else {
            bsplslib::unit_weights(nv, nu)?
        };
        std::mem::swap(&mut self.u_rational, &mut self.v_rational);
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `Increase(UDeg, VDeg)`: degree elevation by BSplSLib::IncreaseDegree, U first.
    /// Quirks kept: the V elevation passes the U knots, and the evaluation
    /// representation stays.
    pub(super) fn increase(&mut self, u_deg: i32, v_deg: i32) -> Result<(), Failure> {
        let max: i32 = bsplclib::max_degree();
        if u_deg < self.u_degree() || u_deg > max || v_deg < self.v_degree() || v_deg > max {
            return Err(errors::construction(&format!(
                "Geom_BezierSurface::Increase: UDeg = {u_deg}, VDeg = {v_deg}; expected in [{}, {max}] x [{}, {max}]",
                self.u_degree(),
                self.v_degree()
            )));
        }
        let (old_u, old_v) = (self.u_degree(), self.v_degree());
        if u_deg == old_u && v_deg == old_v {
            return Ok(());
        }
        if u_deg > old_u {
            self.elevate(true, old_u, u_deg, (u_deg + 1, old_v + 1))?;
        }
        if v_deg > old_v {
            self.elevate(false, old_v, v_deg, (u_deg + 1, v_deg + 1))?;
        }
        if self.is_rational() {
            self.update_rationality()?;
        } else {
            self.weights = self.unit_weights()?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    // One BSplSLib::IncreaseDegree call into grids of `size`.
    fn elevate(
        &mut self,
        u_dir: bool,
        degree: i32,
        new_degree: i32,
        size: (i32, i32),
    ) -> Result<(), Failure> {
        let mut npoles: Array2<'static, Point3> = one_based_grid(size.0, size.1)?;
        let mut nweights: Array2<'static, f64> = one_based_grid(size.0, size.1)?;
        let mut nknots: Array1<'static, f64> =
            Array1::with_bounds(1, 2).map_err(errors::array_bounds)?;
        let mut nmults: Array1<'static, i32> =
            Array1::with_bounds(1, 2).map_err(errors::array_bounds)?;
        let mults = if u_dir {
            self.u_mults()?
        } else {
            self.v_mults()?
        };
        let rational: bool = self.is_rational();
        bsplslib::increase_degree(
            u_dir,
            degree,
            new_degree,
            false,
            &self.poles,
            self.weights_view(),
            &bezier_knots(),
            &mults,
            &mut npoles,
            rational.then_some(&mut nweights),
            &mut nknots,
            &mut nmults,
        )?;
        if rational {
            self.weights = nweights;
        }
        self.poles = npoles;
        Ok(())
    }
    /// `InsertPoleColAfter` / `InsertPoleRowAfter` without weights: a rational surface
    /// gets unit weights on the new line and keeps its rationality flags.
    pub(super) fn insert_after(
        &mut self,
        line: Line,
        index: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        let (along, across) = self.line_counts(line);
        check_line_index(index, along, "InsertPoleAfter")?;
        if poles.len() as i32 != across {
            return Err(errors::construction(&format!(
                "Geom_BezierSurface::InsertPoleAfter: {} poles; expected {across}",
                poles.len()
            )));
        }
        let rational: bool = self.is_rational();
        let mut unit: Array1<'static, f64> =
            Array1::with_bounds(1, across).map_err(errors::array_bounds)?;
        unit.init(1.);
        let (npoles, nweights) = self.inserted(line, index, poles, &unit)?;
        self.weights = if rational {
            nweights
        } else {
            self.grown_unit_weights(line)?
        };
        self.poles = npoles;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `InsertPoleColAfter` / `InsertPoleRowAfter` with weights; the rationality is
    /// computed again on the new weights.
    pub(super) fn insert_weighted_after(
        &mut self,
        line: Line,
        index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (along, across) = self.line_counts(line);
        check_line_index(index, along, "InsertPoleAfter")?;
        if poles.len() as i32 != across || weights.len() != poles.len() {
            return Err(errors::construction(&format!(
                "Geom_BezierSurface::InsertPoleAfter: {} poles and {} weights; expected {across} each",
                poles.len(),
                weights.len()
            )));
        }
        check_weights_positive(weights, "InsertPoleAfter")?;
        let (npoles, nweights) = self.inserted(line, index, poles, weights)?;
        self.poles = npoles;
        self.weights = nweights;
        self.update_rationality()?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    // (along, across): the count of lines and the count of poles in one line.
    fn line_counts(&self, line: Line) -> (i32, i32) {
        match line {
            Line::Col => (self.nb_v_poles(), self.nb_u_poles()),
            Line::Row => (self.nb_u_poles(), self.nb_v_poles()),
        }
    }
    fn grown_size(&self, line: Line, delta: i32) -> (i32, i32) {
        match line {
            Line::Col => (self.nb_u_poles(), self.nb_v_poles() + delta),
            Line::Row => (self.nb_u_poles() + delta, self.nb_v_poles()),
        }
    }
    fn grown_unit_weights(&self, line: Line) -> Result<Array2<'static, f64>, Failure> {
        let (rows, cols) = self.grown_size(line, 1);
        bsplslib::unit_weights(rows, cols)
    }
    // AddPoleCol/AddRatPoleCol (or the row twins) into new one-based grids.
    fn inserted(
        &self,
        line: Line,
        index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
        let (rows, cols) = self.grown_size(line, 1);
        let mut npoles: Array2<'static, Point3> = one_based_grid(rows, cols)?;
        let mut nweights: Array2<'static, f64> = one_based_grid(rows, cols)?;
        if line == Line::Col {
            add_col(&self.poles, poles, index, &mut npoles)?;
            add_col(&self.weights, weights, index, &mut nweights)?;
        } else {
            add_row(&self.poles, poles, index, &mut npoles)?;
            add_row(&self.weights, weights, index, &mut nweights)?;
        }
        Ok((npoles, nweights))
    }
    /// `RemovePoleCol` / `RemovePoleRow`: at least 2 lines must remain.
    pub(super) fn remove(&mut self, line: Line, index: i32) -> Result<(), Failure> {
        let (along, _) = self.line_counts(line);
        check_line_index(index, along, "RemovePole")?;
        if along <= 2 {
            return Err(errors::construction(&format!(
                "Geom_BezierSurface::RemovePole: {along} lines; expected more than 2"
            )));
        }
        let (rows, cols) = self.grown_size(line, -1);
        let mut npoles: Array2<'static, Point3> = one_based_grid(rows, cols)?;
        let mut nweights: Array2<'static, f64> = one_based_grid(rows, cols)?;
        let delete_poles = if line == Line::Col {
            delete_col::<Point3>
        } else {
            delete_row::<Point3>
        };
        let delete_weights = if line == Line::Col {
            delete_col::<f64>
        } else {
            delete_row::<f64>
        };
        delete_poles(&self.poles, index, &mut npoles)?;
        if self.is_rational() {
            delete_weights(&self.weights, index, &mut nweights)?;
            self.weights = nweights;
            self.update_rationality_with_size(rows, cols)?;
        } else {
            self.weights = bsplslib::unit_weights(rows, cols)?;
        }
        self.poles = npoles;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    // Rational(nweights) with unit weights of the new size when no longer rational.
    fn update_rationality_with_size(&mut self, rows: i32, cols: i32) -> Result<(), Failure> {
        (self.u_rational, self.v_rational) = super::state::rationality(&self.weights)?;
        if !self.is_rational() {
            self.weights = bsplslib::unit_weights(rows, cols)?;
        }
        Ok(())
    }
}

impl SurfaceState {
    /// `Segment(U1, U2, V1, V2)`: Taylor coefficients at (0.5, 0.5) by
    /// BSplSLib::BuildCache, transposed when UDegree <= VDegree, trimmed by PLib in U
    /// then V and converted back to poles. Quirk kept: the evaluation representation
    /// stays.
    pub(super) fn segment(&mut self, u1: f64, u2: f64, v1: f64, v2: f64) -> Result<(), Failure> {
        let rational: bool = self.is_rational();
        let (ud, vd) = (self.u_degree(), self.v_degree());
        let (min_degree, max_degree) = if ud <= vd { (ud, vd) } else { (vd, ud) };
        let mut coefs: Array2<'static, Point3> = one_based_grid(max_degree + 1, min_degree + 1)?;
        let mut wcoefs: Array2<'static, f64> = one_based_grid(max_degree + 1, min_degree + 1)?;
        let (u_flat, v_flat) = (self.u_knot_sequence()?, self.v_knot_sequence()?);
        let source = CacheSource {
            u: 0.5,
            v: 0.5,
            u_span_domain: 0.5,
            v_span_domain: 0.5,
            u_periodic: false,
            v_periodic: false,
            u_degree: ud,
            v_degree: vd,
            u_index: 0,
            v_index: 0,
            u_flat_knots: &u_flat,
            v_flat_knots: &v_flat,
        };
        let cache_weights = rational.then_some(&mut wcoefs);
        bsplslib::build_cache(
            &source,
            &self.poles,
            self.weights_view(),
            &mut coefs,
            cache_weights,
        )?;
        if ud <= vd {
            coefs = transposed(&coefs, ud + 1, vd + 1)?;
            if rational {
                wcoefs = transposed(&wcoefs, ud + 1, vd + 1)?;
            }
        }
        let (uf, ul) = (2. * (u1 - 0.5), 2. * (u2 - 0.5));
        let (vf, vl) = (2. * (v1 - 0.5), 2. * (v2 - 0.5));
        if rational {
            plib::u_trimming(uf, ul, &mut coefs, Some(&mut wcoefs))?;
            plib::v_trimming(vf, vl, &mut coefs, Some(&mut wcoefs))?;
            plib::coefficients_poles_grid(
                &coefs,
                Some(&wcoefs),
                &mut self.poles,
                Some(&mut self.weights),
            )?;
        } else {
            plib::u_trimming(uf, ul, &mut coefs, None)?;
            plib::v_trimming(vf, vl, &mut coefs, None)?;
            plib::coefficients_poles_grid(&coefs, None, &mut self.poles, None)?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
}

// `aTransposed(ii, jj) = aCoefs(jj, ii)` into a (rows x cols) one-based grid.
fn transposed<T: Copy + Default>(
    source: &Array2<'_, T>,
    rows: i32,
    cols: i32,
) -> Result<Array2<'static, T>, Failure> {
    let mut target: Array2<'static, T> = one_based_grid(rows, cols)?;
    for ii in 1..=rows {
        for jj in 1..=cols {
            put2(&mut target, ii, jj, super::grid::at2(source, jj, ii)?)?;
        }
    }
    Ok(target)
}
