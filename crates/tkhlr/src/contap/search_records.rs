//! Contap's IntStart record specializations at OCCT 3d097a0.
/// Identical native PathPoint specialization over Adaptor3d_HVertex and Adaptor2d_Curve2d.
/// `ContapThePathPointOfTheSearch::new().is_new()`.
pub type ContapThePathPointOfTheSearch =
    rustycad_tkgeomalgo::int_patch::IntPatchThePathPointOfTheSOnBounds;
/// Identical native Segment specialization; limits retain copies of path points.
/// `ContapTheSegmentOfTheSearch::new().has_first_point()`.
pub type ContapTheSegmentOfTheSearch =
    rustycad_tkgeomalgo::int_patch::IntPatchTheSegmentOfTheSOnBounds;
