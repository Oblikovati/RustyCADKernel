// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ElementFilter.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ElementFilter`: the preliminary check of a pair of mesh elements.
use rustycad_tkernel::standard::Failure;

/// `BRepExtrema_ElementFilter::FilterResult`: skip the pair, report it as overlapping
/// without a test, or test it; `assert_eq!(BRepExtremaFilterResult::DoCheck as i32, 2)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepExtremaFilterResult {
    /// `NoCheck`.
    NoCheck = 0,
    /// `Overlap`.
    Overlap = 1,
    /// `DoCheck`.
    DoCheck = 2,
}

/// `BRepExtrema_ElementFilter` (docs/topalgo-design.md section 13): the virtual
/// `PreCheckElements`, which tells whether two mesh elements must be tested for
/// overlapping (the correct and incorrect cases of shared edges and vertices).
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::{BRepExtremaElementFilter, BRepExtremaFilterResult};
/// struct Everything;
/// impl BRepExtremaElementFilter for Everything {}
/// assert_eq!(Everything.pre_check_elements(1, 2)?, BRepExtremaFilterResult::DoCheck);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait BRepExtremaElementFilter: Send {
    /// `PreCheckElements(theIndex1, theIndex2)`: `DoCheck` by default. It may raise in an
    /// override (the self-intersection filter reads its triangle set).
    fn pre_check_elements(
        &mut self,
        _index1: i32,
        _index2: i32,
    ) -> Result<BRepExtremaFilterResult, Failure> {
        Ok(BRepExtremaFilterResult::DoCheck)
    }
}
