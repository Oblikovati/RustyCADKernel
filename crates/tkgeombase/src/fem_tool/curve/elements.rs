// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Curve.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The element access of `FEmTool_Curve`: SetElement, GetElement, GetPolynom and
//! the lazy canonical conversion `Update`.
use super::Curve;
use super::state::{CurveState, slot};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

// Coeffs(row, col), unchecked natively (FEmTool_Curve.cxx compiles the checks out).
fn cell<'a>(coeffs: &'a mut Array2<'_, f64>, row: i32, col: i32) -> Result<&'a mut f64, Failure> {
    coeffs
        .change_value(row, col)
        .map_err(|error| array_failure(error, "FEmTool_Curve: Coeffs"))
}

impl Curve {
    /// `SetElement(IndexOfElement, Coeffs)`: stores the `Degree + 1` rows of
    /// `coeffs` (one column per dimension, from its lower bounds) and scales the
    /// Hermite derivative rows by `(h/2)^k` (h the element length);
    /// Standard_OutOfRange outside `1..=NbElements`. `curve.set_element(1, &coeffs)?`.
    pub fn set_element(
        &self,
        index_of_element: i32,
        coeffs: &Array2<'_, f64>,
    ) -> Result<(), Failure> {
        self.check_element(index_of_element)?;
        let stenor: f64 = (self.knot(index_of_element + 1)? - self.knot(index_of_element)?) / 2.;
        let mut state = self.state.write();
        let (dimension, degree) = (self.dimension, state.degree(index_of_element)?);
        let base: i32 = state.element_ptr(index_of_element) - 1;
        let (i2, j1) = (coeffs.lower_row() - 1, coeffs.lower_col() - 1);
        for i in 1..=degree + 1 {
            for j in 1..=dimension {
                let value: f64 = *coeffs
                    .value(i2 + i, j1 + j)
                    .map_err(|error| array_failure(error, "FEmTool_Curve::SetElement: Coeffs"))?;
                let position: usize =
                    slot(&state.coeff, base + (i - 1) * dimension + j, "SetElement")?;
                state.coeff[position] = value;
            }
        }
        let niv_constr: i32 = self.base.niv_constr();
        for i in 1..=niv_constr {
            let mfact: f64 = stenor.powf(i as f64);
            for j in 1..=dimension {
                for address in [
                    base + i * dimension + j,
                    base + (niv_constr + 1 + i) * dimension + j,
                ] {
                    let position: usize = slot(&state.coeff, address, "SetElement")?;
                    state.coeff[position] *= mfact;
                }
            }
        }
        state.invalidate(index_of_element)
    }

    /// `GetElement(IndexOfElement, Coeffs)`: the element rows scaled back by
    /// `(2/h)^k`. The native rescaling loop runs over the columns
    /// `LowerCol..=Dimension` (not `LowerCol..LowerCol + Dimension`), which the port
    /// keeps; `curve.get_element(1, &mut coeffs)?`.
    pub fn get_element(
        &self,
        index_of_element: i32,
        coeffs: &mut Array2<'_, f64>,
    ) -> Result<(), Failure> {
        self.check_element(index_of_element)?;
        let stenor: f64 = 2. / (self.knot(index_of_element + 1)? - self.knot(index_of_element)?);
        let state = self.state.read();
        let (dimension, degree) = (self.dimension, state.degree(index_of_element)?);
        let base: i32 = state.element_ptr(index_of_element) - 1;
        let (i2, j1) = (coeffs.lower_row() - 1, coeffs.lower_col() - 1);
        for i in 1..=degree + 1 {
            for j in 1..=dimension {
                let position: usize =
                    slot(&state.coeff, base + (i - 1) * dimension + j, "GetElement")?;
                *cell(coeffs, i2 + i, j1 + j)? = state.coeff[position];
            }
        }
        let (i2, niv_constr) = (coeffs.lower_row(), self.base.niv_constr());
        let i3: i32 = i2 + niv_constr + 1;
        for i in 1..=niv_constr {
            let mfact: f64 = stenor.powf(i as f64);
            for j in j1 + 1..=dimension {
                *cell(coeffs, i2 + i, j)? *= mfact;
                *cell(coeffs, i3 + i, j)? *= mfact;
            }
        }
        Ok(())
    }

    /// `GetPolynom(Coeffs)`: the canonical coefficients of every element (on
    /// [-1, 1]), `(WorkDegree + 1) * Dimension` values per element from
    /// `Coeffs.Lower()`; `curve.get_polynom(&mut coeffs)?`.
    pub fn get_polynom(&self, coeffs: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let mut state = self.state.write();
        for index in 1..=self.nb_elements {
            if !state.has_poly[slot(&state.has_poly, index, "GetPolynom")?] {
                self.update(&mut state, index, 0)?;
            }
        }
        let lower: i32 = coeffs.lower();
        for (offset, value) in state.poly.iter().enumerate() {
            coeffs
                .set_value(lower + offset as i32, *value)
                .map_err(|error| array_failure(error, "FEmTool_Curve::GetPolynom"))?;
        }
        Ok(())
    }

    /// `Update(Element, Order)`: the canonical coefficients (order 0) and their
    /// first (order 1) and second (order 2) derivative coefficients of an element.
    pub(super) fn update(
        &self,
        state: &mut CurveState,
        index: i32,
        order: i32,
    ) -> Result<(), Failure> {
        let position: usize = slot(&state.degrees, index, "Update")?;
        if !state.has_poly[position] {
            self.convert_element(state, index)?;
            state.has_poly[position] = true;
        }
        if order < 1 {
            return Ok(());
        }
        let (dimension, degree, work_degree) =
            (self.dimension, state.degrees[position], state.work_degree);
        if !state.has_deri[position] {
            let (i1, i2) = (
                (index - 1) * work_degree * dimension,
                (index - 1) * (work_degree + 1) * dimension,
            );
            derive(&state.poly, &mut state.deri, [i1, i2], degree, dimension)?;
            state.has_deri[position] = true;
        }
        if order >= 2 && !state.has_secn[position] {
            let (i1, i2) = (
                (index - 1) * (work_degree - 1) * dimension,
                (index - 1) * work_degree * dimension,
            );
            derive(
                &state.deri,
                &mut state.dsecn,
                [i1, i2],
                degree - 1,
                dimension,
            )?;
            state.has_secn[position] = true;
        }
        Ok(())
    }

    // PLib_HermitJacobi::ToCoefficients of the element into myPoly.
    fn convert_element(&self, state: &mut CurveState, index: i32) -> Result<(), Failure> {
        let degree: i32 = state.degree(index)?;
        let start: usize = state.element_start(index)?;
        let count: usize = usize::try_from(self.dimension * (degree + 1)).unwrap_or(0);
        let source: &[f64] = state.coeff.get(start..start + count).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FEmTool_Curve::Update: element {index} outside the curve"
            ))
        })?;
        let herm_jac: Array1<'_, f64> = Array1::from_shared(source, 0);
        let mut canonical: Array1<'static, f64> = Array1::with_bounds(0, count as i32 - 1)
            .map_err(|error| array_failure(error, "FEmTool_Curve::Update"))?;
        self.base
            .to_coefficients(self.dimension, degree, &herm_jac, &mut canonical)?;
        state.poly[start..start + count].copy_from_slice(canonical.as_slice());
        Ok(())
    }
}

// target(i1 + (i-1)*dim + j) = i * source(i2 + i*dim + j) for i in 1..=degree: the
// coefficients of the derivative of a canonical polynomial (1-based addresses).
fn derive(
    source: &[f64],
    target: &mut [f64],
    [i1, i2]: [i32; 2],
    degree: i32,
    dimension: i32,
) -> Result<(), Failure> {
    for i in 1..=degree {
        for j in 1..=dimension {
            let from: usize = slot(source, i2 + i * dimension + j, "Update")?;
            let to: usize = slot(target, i1 + (i - 1) * dimension + j, "Update")?;
            target[to] = i as f64 * source[from];
        }
    }
    Ok(())
}
