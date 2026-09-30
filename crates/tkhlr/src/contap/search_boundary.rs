//! Original IntStart restriction search instantiated for Contap.
use super::{
    ContapArcFunction, ContapHContTool, ContapThePathPointOfTheSearch, ContapTheSegmentOfTheSearch,
};
use rustycad_tkgeomalgo::int_start::{SearchOnBoundaries, SearchOnBoundariesTypes, TheTopolTool};
/// Native template arguments of Contap_TheSearch_0.cxx at OCCT 3d097a0.
/// Used through `ContapTheSearch::new()`.
pub struct ContapTheSearchTypes;
impl SearchOnBoundariesTypes for ContapTheSearchTypes {
    type TopolTool = dyn TheTopolTool;
    type SOBTool = ContapHContTool;
    type Function = ContapArcFunction;
    type PathPoint = ContapThePathPointOfTheSearch;
    type Segment = ContapTheSegmentOfTheSearch;
}
/// Contour roots and coincident segments on surface restrictions.
/// `ContapTheSearch::new().is_done()`.
pub type ContapTheSearch = SearchOnBoundaries<ContapTheSearchTypes>;
