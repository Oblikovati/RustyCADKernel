//! `BRepFont_Regularizer`: converts a font outline into regularized planar topology.
use super::planar_region::BRepFontPlanarRegion;
use super::regularizer_append::append_outline;
use super::regularizer_classify::classify_loops;
use super::regularizer_intersect::{BRepFontIntersectionKind, intersections};
use rustycad_tkernel::precision;
use rustycad_tkservice::font::FontGlyphOutline;
use std::collections::HashSet;

/// `BRepFont_Regularizer::Status`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRepFontRegularizerStatus {
    /// `Success`.
    Success = 0,
    /// `Empty`.
    Empty = 1,
    /// `InvalidOutline`.
    InvalidOutline = 2,
    /// `OpenContour`.
    OpenContour = 3,
    /// `IntersectingContours`.
    IntersectingContours = 4,
    /// `SelfIntersectingContour`: a marked outline contains a self-intersecting contour.
    SelfIntersectingContour = 5,
}

/// `BRepFont_Regularizer::Options`; `BRepFontRegularizerOptions { tolerance: 0.01 }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepFontRegularizerOptions {
    /// `Tolerance`, in the design units of the input outline.
    pub tolerance: f64,
}

impl Default for BRepFontRegularizerOptions {
    fn default() -> Self {
        Self { tolerance: 1.0e-7 }
    }
}

/// `BRepFont_Regularizer::Result`: a status and, when done, the region.
/// Example: `let result = BRepFontRegularizer.build(&outline); result.is_done()`.
#[derive(Clone, Debug)]
pub struct BRepFontRegularizerResult {
    status: BRepFontRegularizerStatus,
    region: Option<BRepFontPlanarRegion>,
}

impl Default for BRepFontRegularizerResult {
    fn default() -> Self {
        Self {
            status: BRepFontRegularizerStatus::InvalidOutline,
            region: None,
        }
    }
}

impl BRepFontRegularizerResult {
    /// `StatusValue`; `result.status_value()`.
    pub fn status_value(&self) -> BRepFontRegularizerStatus {
        self.status
    }
    /// `IsDone`: success or empty; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        matches!(
            self.status,
            BRepFontRegularizerStatus::Success | BRepFontRegularizerStatus::Empty
        )
    }
    /// `PlanarRegion`; `result.planar_region().is_some()`.
    pub fn planar_region(&self) -> &Option<BRepFontPlanarRegion> {
        &self.region
    }
    /// `ChangePlanarRegion`: the region for moving it out; `result.change_planar_region().take()`.
    pub fn change_planar_region(&mut self) -> &mut Option<BRepFontPlanarRegion> {
        &mut self.region
    }
    fn failed(status: BRepFontRegularizerStatus) -> Self {
        Self {
            status,
            region: None,
        }
    }
}

/// `isValidReal`: neither NaN nor `Precision::IsInfinite`.
pub(super) fn is_valid_real(value: f64) -> bool {
    !value.is_nan() && !precision::is_infinite(value)
}

/// `BRepFont_Regularizer`: a stateless outline-to-region converter.
/// Unmarked intersections fail without a region; marked intersections between loops are
/// kept as separate output regions; an intersection inside one loop is
/// `SelfIntersectingContour`.
/// Example: `BRepFontRegularizer.build(&outline).status_value()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepFontRegularizer;

impl BRepFontRegularizer {
    /// `Build(theOutline)` with the default options; `BRepFontRegularizer.build(&outline)`.
    pub fn build(&self, outline: &FontGlyphOutline) -> BRepFontRegularizerResult {
        self.build_with(outline, &BRepFontRegularizerOptions::default())
    }

    /// `Build(theOutline, theOptions)`; `BRepFontRegularizer.build_with(&outline, &options)`.
    pub fn build_with(
        &self,
        outline: &FontGlyphOutline,
        options: &BRepFontRegularizerOptions,
    ) -> BRepFontRegularizerResult {
        if !valid_inputs(outline, options) {
            return BRepFontRegularizerResult::default();
        }
        let mut region = BRepFontPlanarRegion {
            units_per_em: outline.units_per_em(),
            tolerance: options.tolerance,
            is_single_stroke: outline.is_single_stroke(),
            ..BRepFontPlanarRegion::default()
        };
        if outline.is_empty() {
            return empty_outline_result(outline, region);
        }
        if let Err(status) = append_outline(outline, options.tolerance, &mut region) {
            return BRepFontRegularizerResult::failed(status);
        }
        regularize_loops(outline, options.tolerance, region)
    }
}

fn valid_inputs(outline: &FontGlyphOutline, options: &BRepFontRegularizerOptions) -> bool {
    let tolerance = options.tolerance;
    outline.units_per_em() > 0.0
        && is_valid_real(outline.units_per_em())
        && tolerance > 0.0
        && is_valid_real(tolerance)
        && is_valid_real(tolerance * tolerance)
}

fn empty_outline_result(
    outline: &FontGlyphOutline,
    region: BRepFontPlanarRegion,
) -> BRepFontRegularizerResult {
    if !outline.contours().is_empty() {
        return BRepFontRegularizerResult::default();
    }
    BRepFontRegularizerResult {
        status: BRepFontRegularizerStatus::Empty,
        region: Some(region),
    }
}

// The intersection check and the loop classification of Build after appendOutline.
fn regularize_loops(
    outline: &FontGlyphOutline,
    tolerance: f64,
    mut region: BRepFontPlanarRegion,
) -> BRepFontRegularizerResult {
    let mut loop_pairs: HashSet<u64> = HashSet::new();
    let kind = if region.is_single_stroke() {
        BRepFontIntersectionKind::None
    } else {
        intersections(&region, tolerance, &mut loop_pairs)
    };
    if kind != BRepFontIntersectionKind::None {
        if !outline.has_possible_overlaps() {
            return BRepFontRegularizerResult::failed(
                BRepFontRegularizerStatus::IntersectingContours,
            );
        }
        if kind == BRepFontIntersectionKind::WithinLoop {
            return BRepFontRegularizerResult::failed(
                BRepFontRegularizerStatus::SelfIntersectingContour,
            );
        }
    }
    if !region.is_single_stroke() && !classify_loops(outline, &mut region, &loop_pairs) {
        return BRepFontRegularizerResult::failed(BRepFontRegularizerStatus::InvalidOutline);
    }
    let status = if region.is_empty() {
        BRepFontRegularizerStatus::Empty
    } else {
        BRepFontRegularizerStatus::Success
    };
    BRepFontRegularizerResult {
        status,
        region: Some(region),
    }
}
