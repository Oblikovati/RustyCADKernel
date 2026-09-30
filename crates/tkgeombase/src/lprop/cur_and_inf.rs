// Rust translation of ModelingData/TKGeomBase/LProp/LProp_CurAndInf.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LProp_CurAndInf`: the sorted particular points of a curve.
use super::CIType;
use rustycad_tkernel::standard::Failure;

/// `LProp_CurAndInf`: the parameters of the curvature extrema and inflections of
/// a curve, kept in increasing order with their [`CIType`].
///
/// ```
/// use rustycad_tkgeombase::lprop::{CIType, CurAndInf};
/// let mut points = CurAndInf::new();
/// points.add_inflection(2.0);
/// points.add_ext_cur(1.0, true);
/// assert_eq!(points.type_of(1).unwrap(), CIType::MinCur);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurAndInf {
    the_params: Vec<f64>,
    the_types: Vec<CIType>,
}

impl CurAndInf {
    /// An empty store (`LProp_CurAndInf()`); `CurAndInf::new().is_empty()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an inflection at `param` (`AddInflection`); `points.add_inflection(0.5)`.
    ///
    /// A parameter equal to the last one (or NaN) is dropped, as natively.
    pub fn add_inflection(&mut self, param: f64) {
        self.insert_sorted(param, CIType::Inflection);
    }

    /// Add a curvature extremum at `param`, a minimum of the radius of curvature
    /// when `is_min` (`AddExtCur`); `points.add_ext_cur(0.5, true)`.
    pub fn add_ext_cur(&mut self, param: f64, is_min: bool) {
        let type_point: CIType = if is_min {
            CIType::MinCur
        } else {
            CIType::MaxCur
        };
        self.insert_sorted(param, type_point);
    }

    // The native insertion: append above the last parameter, otherwise insert before
    // the first strictly greater one. A parameter equal to the last one, or NaN,
    // finds no greater parameter and is silently dropped.
    fn insert_sorted(&mut self, param: f64, kind: CIType) {
        if self.the_params.last().is_none_or(|last| param > *last) {
            self.the_params.push(param);
            self.the_types.push(kind);
            return;
        }
        if let Some(index) = self.the_params.iter().position(|value| *value > param) {
            self.the_params.insert(index, param);
            self.the_types.insert(index, kind);
        }
    }

    /// Remove every point (`Clear`); `points.clear()`.
    pub fn clear(&mut self) {
        self.the_params.clear();
        self.the_types.clear();
    }

    /// Whether no point is stored (`IsEmpty`); `points.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.the_params.is_empty()
    }

    /// Number of points, sorted by increasing parameter (`NbPoints`); `points.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.the_params.len() as i32
    }

    /// Parameter of the `n`-th point, `n` in `[1, NbPoints()]`, else
    /// Standard_OutOfRange (`Parameter`); `points.parameter(1)?`.
    pub fn parameter(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.the_params[self.checked_index(n, "Parameter")?])
    }

    /// Kind of the `n`-th point, `n` in `[1, NbPoints()]`, else Standard_OutOfRange
    /// (`Type`); `points.type_of(1)?`.
    pub fn type_of(&self, n: i32) -> Result<CIType, Failure> {
        Ok(self.the_types[self.checked_index(n, "Type")?])
    }

    fn checked_index(&self, n: i32, member: &str) -> Result<usize, Failure> {
        if n < 1 || n > self.nb_points() {
            return Err(Failure::out_of_range(&format!(
                "LProp_CurAndInf::{member}: N = {n}; expected in [1, {}]",
                self.nb_points()
            )));
        }
        Ok(n as usize - 1)
    }
}
