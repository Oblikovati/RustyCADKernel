// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_LinearCriteria.cxx (Hessian, Gradient,
// InputVector, BuildCache)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The element Hessian and gradient of `AppDef_LinearCriteria`, the basis cache
//! of the points of an element, and the solution written back into the curve.
use super::super::multi_line::MultiLine;
use super::super::point_tabs::PointTabs;
use super::quality::knot;
use super::{CriteriaState, domain, non_null};
use crate::app_par_curves::LineTool;
use crate::fem_tool::{AssemblyTable, Curve, ElementaryCriterion};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::math::{Matrix, Vector};

const CONTEXT: &str = "AppDef_LinearCriteria";

// The power of the element half-length applied to Hermite basis function `i`.
fn hermite_power(i: i32, order: i32) -> i32 {
    if i <= order { i } else { i - order - 1 }
}

fn param(parameters: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    parameters
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, CONTEXT))
}

/// `BuildCache(E)`: the parameters of the element's points (`IF..=IL`, the first
/// element also takes its left knot) and the basis values at each; with no point the
/// range becomes `IL..=IL - 1` of the previous range and the old cache stays.
fn build_cache(state: &mut CriteriaState, curve: &Curve, element: i32) -> Result<(), Failure> {
    let (u_first, u_last) = (knot(curve, element)?, knot(curve, element + 1)?);
    let parameters: Array1<'static, f64> = non_null(&state.parameters, "the parameters")?
        .array1()
        .clone();
    state.first_index = 0;
    for ipnt in parameters.lower()..=parameters.upper() {
        let t: f64 = parameters[ipnt];
        if (t > u_first && t <= u_last) || (element == 1 && t == u_first) {
            if state.first_index == 0 {
                state.first_index = ipnt;
            }
            state.last_index = ipnt;
        } else if t > u_last {
            break;
        }
    }
    if state.first_index == 0 {
        // no points in the interval.
        state.first_index = state.last_index;
        state.last_index -= 1;
    } else {
        state.cache = fill_cache(state, curve, &parameters, [u_first, u_last])?;
    }
    state.element = element;
    Ok(())
}

// The basis values D0((t + c0) * coeff) of the points IF..=IL, WorkDegree + 1 each.
fn fill_cache(
    state: &CriteriaState,
    curve: &Curve,
    parameters: &Array1<'_, f64>,
    [u_first, u_last]: [f64; 2],
) -> Result<Handle<HArray1<f64>>, Failure> {
    let order: i32 = curve.base().work_degree() + 1;
    let cache: HArray1<f64> =
        HArray1::with_bounds(1, (state.last_index - state.first_index + 1) * order)
            .map_err(|error| array_failure(error, CONTEXT))?;
    let mut basic_value: Array1<'static, f64> =
        Array1::with_bounds(0, order - 1).map_err(|error| array_failure(error, CONTEXT))?;
    for (ipnt, ii) in (state.first_index..=state.last_index).zip((1..).step_by(order as usize)) {
        let t: f64 = param(parameters, ipnt)?;
        let (coeff, c0) = (2. / (u_last - u_first), -(u_last + u_first) / 2.);
        let s: f64 = (t + c0) * coeff;
        curve.base().d0(s, &mut basic_value)?;
        let mut values = cache.change_array1();
        for (k, value) in (ii..).zip(basic_value.iter()) {
            values
                .set_value(k, *value)
                .map_err(|error| array_failure(error, CONTEXT))?;
        }
    }
    Ok(Handle::new(cache))
}

// The cache as an array (`&myCache->Value(ii)` natively reads from a null cache).
fn cache_values(state: &CriteriaState) -> Result<Array1<'static, f64>, Failure> {
    Ok(non_null(&state.cache, "the basis cache")?.array1().clone())
}

fn cache_at(cache: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    cache
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, CONTEXT))
}

// The quality part: the three element Hessians weighted by
// QualityWeight * percent / estimation.
fn quality_hessian(
    state: &CriteriaState,
    ([u_first, u_last], [dimension1, dimension2]): ([f64; 2], [i32; 2]),
    h: &mut Matrix<'_>,
    aux_h: &mut Matrix<'_>,
) -> Result<(), Failure> {
    h.init(0.);
    for icrit in 0..3 {
        let criterion: &dyn ElementaryCriterion =
            non_null(&state.criteria[icrit], "an element criterion")?;
        criterion.set(u_first, u_last);
        criterion.hessian(dimension1, dimension2, aux_h)?;
        let weight: f64 = state.quality_weight * state.percent[icrit] / state.estimation[icrit];
        h.add(&aux_h.multiplied(weight))?;
    }
    Ok(())
}

// The least-squares part of the Hessian of one point (upper triangle).
fn point_hessian(
    aux_h: &mut Matrix<'_>,
    bv: &[f64],
    poid: f64,
    [coeff, order, mx_deg]: [f64; 3],
) -> Result<(), Failure> {
    let (order, mx_deg): (i32, i32) = (order as i32, mx_deg as i32);
    let deg_h: i32 = 2 * order + 1;
    for i in 0..=deg_h {
        let curcoeff: f64 = coeff.powf(f64::from(hermite_power(i, order))) * poid * bv[i as usize];
        // Hermite*Hermite part of matrix
        for j in i..=deg_h {
            *aux_h.change_value(i, j)? +=
                curcoeff * coeff.powf(f64::from(hermite_power(j, order))) * bv[j as usize];
        }
        // Hermite*Jacobi part of matrix
        for j in deg_h + 1..=mx_deg {
            *aux_h.change_value(i, j)? += curcoeff * bv[j as usize];
        }
    }
    // Jacoby*Jacobi part of matrix
    for i in deg_h + 1..=mx_deg {
        let curcoeff: f64 = bv[i as usize] * poid;
        for j in i..=mx_deg {
            *aux_h.change_value(i, j)? += curcoeff * bv[j as usize];
        }
    }
    Ok(())
}

// The basis values of cache entry `ii`: `&myCache->Value(ii)` read over WorkDegree + 1.
fn basis_slice(cache: &Array1<'_, f64>, ii: i32, count: i32) -> Result<Vec<f64>, Failure> {
    (ii..ii + count).map(|k| cache_at(cache, k)).collect()
}

/// `Hessian(Element, Dimension1, Dimension2, H)` after the null and dependence
/// checks: the quality Hessian plus QuadraticWeight times the least-squares one,
/// symmetrized from its upper triangle.
pub(super) fn hessian(
    state: &mut CriteriaState,
    curve: &Handle<Curve>,
    (element, dimensions): (i32, [i32; 2]),
    h: &mut Matrix<'_>,
) -> Result<(), Failure> {
    let curve: &Curve = non_null(curve, "the curve")?;
    let (mx_deg, order) = (curve.base().work_degree(), curve.base().niv_constr());
    let mut aux_h: Matrix<'static> =
        Matrix::filled(0, h.row_number() - 1, 0, h.col_number() - 1, 0.)
            .map_err(|error| array_failure(error, CONTEXT))?;
    let (u_first, u_last) = (knot(curve, element)?, knot(curve, element + 1)?);
    quality_hessian(state, ([u_first, u_last], dimensions), h, &mut aux_h)?;
    aux_h.init(0.);
    let coeff: f64 = (u_last - u_first) / 2.;
    let parameters_lower: i32 = non_null(&state.parameters, "the parameters")?
        .array1()
        .lower();
    let di: i32 = state.pnt_weight.lower() - parameters_lower;
    if state.element != element {
        build_cache(state, curve, element)?;
    }
    let points = state.first_index..=state.last_index;
    // The cache is read only when the element has points (a null one is never read).
    let cache: Array1<'static, f64> = if points.is_empty() {
        Array1::default()
    } else {
        cache_values(state)?
    };
    for (ipnt, ii) in points.zip((1..).step_by((mx_deg + 1) as usize)) {
        let poid: f64 = weight(state, di + ipnt)? * 2.;
        let bv: Vec<f64> = basis_slice(&cache, ii, mx_deg + 1)?;
        point_hessian(
            &mut aux_h,
            &bv,
            poid,
            [coeff, f64::from(order), f64::from(mx_deg)],
        )?;
    }
    symmetrize(h, &aux_h, state.quadratic_weight, mx_deg)
}

fn weight(state: &CriteriaState, index: i32) -> Result<f64, Failure> {
    state
        .pnt_weight
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, CONTEXT))
}

// H(i1, j1) += QuadraticWeight * AuxH(i, j) over the upper triangle, mirrored.
fn symmetrize(
    h: &mut Matrix<'_>,
    aux_h: &Matrix<'_>,
    quadratic_weight: f64,
    mx_deg: i32,
) -> Result<(), Failure> {
    let (i0, j0) = (h.lower_row(), h.lower_col());
    for i in 0..=mx_deg {
        let i1: i32 = i0 + i;
        for (j, j1) in (i..=mx_deg).zip(j0 + i..) {
            let value: f64 = *h.value(i1, j1)? + quadratic_weight * *aux_h.value(i, j)?;
            *h.change_value(i1, j1)? = value;
            *h.change_value(j1, i1)? = value;
        }
    }
    Ok(())
}

/// The coordinate `Gradient` reads: `In3d`, the point `IndPnt` and its coordinate
/// `IndCrd` of the line dimension `Dimension`.
pub(super) struct GradientTarget {
    in_3d: bool,
    point: i32,
    coordinate: i32,
    dims: [i32; 2],
}

impl GradientTarget {
    /// The target of `dimension`, counting the 3D coordinates first;
    /// Standard_DomainError above `2 NbP2d + 3 NbP3d` (the native message names
    /// ErrorValues).
    pub(super) fn new(ssp: &MultiLine, dimension: i32) -> Result<Self, Failure> {
        let (nb_p2d, nb_p3d) = (ssp.nb_p2d()?, ssp.nb_p3d()?);
        if dimension > 2 * nb_p2d + 3 * nb_p3d {
            return Err(domain(
                "ErrorValues",
                &format!(
                    "dimension {dimension}; expected <= {}",
                    2 * nb_p2d + 3 * nb_p3d
                ),
            ));
        }
        let (in_3d, relative, size) = if dimension <= 3 * nb_p3d {
            (true, dimension, 3)
        } else {
            (false, dimension - 3 * nb_p3d, 2)
        };
        let (coordinate, point) = match relative % size {
            0 => (size, relative / size),
            rest => (rest, relative / size + 1),
        };
        Ok(Self {
            in_3d,
            point,
            coordinate,
            dims: [nb_p3d, nb_p2d],
        })
    }

    // Pnt = TabP3d(IndPnt).Coord(IndCrd) or its 2D twin.
    fn coordinate(&self, ssp: &MultiLine, ipnt: i32, tabs: &mut PointTabs) -> Result<f64, Failure> {
        if self.in_3d {
            tabs.read_3d(ssp, ipnt)?;
            let point = tabs
                .p3d
                .value(self.point)
                .map_err(|error| array_failure(error, CONTEXT))?;
            return point.coord(self.coordinate);
        }
        tabs.read_2d(ssp, ipnt)?;
        let point = tabs
            .p2d
            .value(self.point)
            .map_err(|error| array_failure(error, CONTEXT))?;
        point.coord(self.coordinate)
    }
}

/// `Gradient(Element, Dimension, G)` after the null and dimension checks: twice
/// QuadraticWeight times the weighted point coordinates on the basis values, the
/// Hermite entries scaled by the powers of the element half-length.
pub(super) fn gradient(
    state: &mut CriteriaState,
    curve: &Handle<Curve>,
    element: i32,
    (ssp, target): (&MultiLine, &GradientTarget),
    g: &mut Vector<'_>,
) -> Result<(), Failure> {
    let curve: &Curve = non_null(curve, "the curve")?;
    let mut tabs: PointTabs = PointTabs::new(target.dims)?;
    let (u_first, u_last) = (knot(curve, element)?, knot(curve, element + 1)?);
    let coeff: f64 = (u_last - u_first) / 2.;
    let (order, mx_deg) = (curve.base().niv_constr(), curve.base().work_degree());
    let (i0, parameters_lower) = (
        g.lower(),
        non_null(&state.parameters, "the parameters")?
            .array1()
            .lower(),
    );
    let di: i32 = state.pnt_weight.lower() - parameters_lower;
    if state.element != element {
        build_cache(state, curve, element)?;
    }
    g.init(0.);
    let cache: Array1<'static, f64> = cache_values(state)?;
    for (ipnt, ii) in
        (state.first_index..=state.last_index).zip((1..).step_by((mx_deg + 1) as usize))
    {
        let curcoeff: f64 = target.coordinate(ssp, ipnt, &mut tabs)? * weight(state, di + ipnt)?;
        for i in 0..=mx_deg {
            *g.change_value(i0 + i)? += cache_at(&cache, ii + i)? * curcoeff;
        }
    }
    g.multiply(2. * state.quadratic_weight);
    for i in 0..=2 * order + 1 {
        *g.change_value(i0 + i)? *= coeff.powf(f64::from(hermite_power(i, order)));
    }
    Ok(())
}

/// `InputVector(X, AssTable)`: every element takes WorkDegree and the coefficients
/// `X(GlobIndex(i))` of the assembly table.
pub(super) fn input_vector(
    curve: &Curve,
    x: &Vector<'_>,
    ass_table: &AssemblyTable,
) -> Result<(), Failure> {
    let (nb_dim, nb_elm, mx_deg) = (
        curve.dimension(),
        curve.nb_elements(),
        curve.base().work_degree(),
    );
    let mut coeff_el: Array2<'static, f64> =
        Array2::with_bounds(0, mx_deg, 1, nb_dim).map_err(|error| array_failure(error, CONTEXT))?;
    let table = non_null(ass_table, "the assembly table")?.array2().clone();
    let i0: i32 = x.lower() - 1;
    for el in 1..=nb_elm {
        for dim in 1..=nb_dim {
            let glob_index = table
                .value(dim, el)
                .map_err(|error| array_failure(error, CONTEXT))?;
            let glob_index = non_null(glob_index, "an assembly table entry")?
                .array1()
                .clone();
            for i in 0..=mx_deg {
                let index: i32 = *glob_index
                    .value(i)
                    .map_err(|error| array_failure(error, CONTEXT))?;
                *coeff_el
                    .change_value(i, dim)
                    .map_err(|error| array_failure(error, CONTEXT))? = *x.value(i0 + index)?;
            }
        }
        curve.set_degree(el, mx_deg)?;
        curve.set_element(el, &coeff_el)?;
    }
    Ok(())
}
