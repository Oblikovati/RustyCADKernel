//! Native handle-list output routing preserves aliased U/V destinations and appended sequence identity.
use rustycad_tkernel::{
    ncollection::{HSequence, List, Sequence},
    standard::Handle,
};
use rustycad_tkmath::gp::Point3;
/// Ordered native polyline handles; `let lines = StdPrsIsolinePolylines::default()`.
pub type StdPrsIsolinePolylines = List<Handle<HSequence<Point3>>>;
/// Distinct or genuinely aliased output references; `StdPrsIsolineDestination::Shared(&mut lines)`.
pub enum StdPrsIsolineDestination<'a> {
    /// U and V outputs preserve their independent append order.
    Distinct {
        /// Appended U lines.
        u: &'a mut StdPrsIsolinePolylines,
        /// Appended V lines.
        v: &'a mut StdPrsIsolinePolylines,
    },
    /// Both native references denote the same list, without copying/rerouting afterward.
    Shared(&'a mut StdPrsIsolinePolylines),
}
impl StdPrsIsolineDestination<'_> {
    pub(super) fn append(&mut self, is_u: bool, points: Sequence<Point3>) {
        let polyline = Handle::new(HSequence::from_owned(points));
        self.append_handle(is_u, polyline);
    }
    pub(super) fn append_handle(&mut self, is_u: bool, polyline: Handle<HSequence<Point3>>) {
        let target = match self {
            Self::Distinct { u, v } => isoline_distinct_target(is_u, u, v),
            Self::Shared(output) => output,
        };
        target.append(polyline);
    }
    /// Preserve two distinct native output references; `StdPrsIsolineDestination::distinct(&mut u,&mut v)`.
    pub fn distinct<'a>(
        u: &'a mut StdPrsIsolinePolylines,
        v: &'a mut StdPrsIsolinePolylines,
    ) -> StdPrsIsolineDestination<'a> {
        // Native list references must stay live rather than collecting detached vectors first.
        StdPrsIsolineDestination::Distinct { u, v }
    }
}
fn isoline_distinct_target<'a>(
    is_u: bool,
    u: &'a mut StdPrsIsolinePolylines,
    v: &'a mut StdPrsIsolinePolylines,
) -> &'a mut StdPrsIsolinePolylines {
    if is_u { u } else { v }
}
pub(super) fn empty_isoline_lists() -> [StdPrsIsolinePolylines; 2] {
    // Each presentation overload owns independent lists even when its final aspects compare equal.
    std::array::from_fn(|_| StdPrsIsolinePolylines::default())
}
