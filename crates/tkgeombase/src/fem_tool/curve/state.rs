// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Curve.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mutable members of `FEmTool_Curve`: element degrees, the Hermite-Jacobi
//! coefficients, the lazily derived canonical coefficients and their flags, the
//! cached element lengths and the evaluation span cache.
use rustycad_tkernel::standard::Failure;

/// The fields of `FEmTool_Curve` a const handle can still change natively.
pub(super) struct CurveState {
    pub(super) dimension: i32,
    pub(super) work_degree: i32,
    pub(super) degrees: Vec<i32>,
    pub(super) coeff: Vec<f64>,
    pub(super) poly: Vec<f64>,
    pub(super) deri: Vec<f64>,
    pub(super) dsecn: Vec<f64>,
    pub(super) has_poly: Vec<bool>,
    pub(super) has_deri: Vec<bool>,
    pub(super) has_secn: Vec<bool>,
    pub(super) lengths: Vec<f64>,
    pub(super) span: SpanCache,
}

/// `myIndex`, `Uf`, `Ul`, `Denom`, `USum` and `myPtr`: the element found by the
/// last evaluation (`index` 0 before the first one).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SpanCache {
    pub(super) index: i32,
    pub(super) uf: f64,
    pub(super) ul: f64,
    pub(super) denom: f64,
    pub(super) usum: f64,
    pub(super) ptr: i32,
}

// The size `count` of an array of `per_element` values per element and dimension.
fn sized(dimension: i32, nb_elements: i32, per_element: i32) -> Result<usize, Failure> {
    usize::try_from(dimension * nb_elements * per_element).map_err(|_| {
        Failure::with_type(
            "Standard_RangeError",
            &format!(
                "FEmTool_Curve: {dimension} dimensions x {nb_elements} elements x {per_element}; expected a non-negative size"
            ),
            "",
        )
    })
}

/// The 0-based slot of the 1-based index `index` in `values`.
pub(super) fn slot(values: &[impl Sized], index: i32, name: &str) -> Result<usize, Failure> {
    usize::try_from(index - 1)
        .ok()
        .filter(|slot| *slot < values.len())
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FEmTool_Curve::{name}: index {index}; expected 1..={}",
                values.len()
            ))
        })
}

impl CurveState {
    /// Every element at `work_degree`, no derived coefficients, lengths -1.
    pub(super) fn new(dimension: i32, nb_elements: i32, work_degree: i32) -> Result<Self, Failure> {
        let elements: usize = sized(1, nb_elements, 1)?;
        Ok(Self {
            dimension,
            work_degree,
            degrees: vec![work_degree; elements],
            coeff: vec![0.; sized(dimension, nb_elements, work_degree + 1)?],
            poly: vec![0.; sized(dimension, nb_elements, work_degree + 1)?],
            deri: vec![0.; sized(dimension, nb_elements, work_degree)?],
            dsecn: vec![0.; sized(dimension, nb_elements, work_degree - 1)?],
            has_poly: vec![false; elements],
            has_deri: vec![false; elements],
            has_secn: vec![false; elements],
            lengths: vec![-1.; elements],
            span: SpanCache::default(),
        })
    }

    /// `myDegree(IndexOfElement)`.
    pub(super) fn degree(&self, index: i32) -> Result<i32, Failure> {
        Ok(self.degrees[slot(&self.degrees, index, "Degree")?])
    }

    /// `myDegree(IndexOfElement)` for writing.
    pub(super) fn degree_mut(&mut self, index: i32) -> Result<&mut i32, Failure> {
        let position: usize = slot(&self.degrees, index, "Degree")?;
        Ok(&mut self.degrees[position])
    }

    /// Drops the derived coefficients and the cached length of an element.
    pub(super) fn invalidate(&mut self, index: i32) -> Result<(), Failure> {
        let position: usize = slot(&self.degrees, index, "SetElement")?;
        self.has_poly[position] = false;
        self.has_deri[position] = false;
        self.has_secn[position] = false;
        self.lengths[position] = -1.;
        Ok(())
    }

    /// The 1-based start `myPtr` of element `index` in `myCoeff`/`myPoly`
    /// (`(index - 1) * (WorkDegree + 1) * Dimension + 1`).
    pub(super) fn element_ptr(&self, index: i32) -> i32 {
        (index - 1) * (self.work_degree + 1) * self.dimension + 1
    }

    /// The 0-based start of element `index` in `myCoeff`/`myPoly`.
    pub(super) fn element_start(&self, index: i32) -> Result<usize, Failure> {
        let ptr: i32 = self.element_ptr(index);
        usize::try_from(ptr - 1)
            .ok()
            .filter(|start| *start <= self.coeff.len())
            .ok_or_else(|| {
                Failure::out_of_range(&format!("FEmTool_Curve: element {index} outside the curve"))
            })
    }

    /// `myCoeff` from the 0-based `start`.
    pub(super) fn coeff_from(&self, start: usize) -> Result<&[f64], Failure> {
        self.coeff.get(start..).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FEmTool_Curve: coefficient {start} outside the curve"
            ))
        })
    }
}
