//! Native public isoline segment value types; pointer adapters preserve actual inline storage.
use rustycad_tkmath::gp::Point3;
/// One initialized native point/parameter pair; `StdPrsPntOnIso { pnt: Point3::default(), param: 0. }`.
#[derive(Clone, Copy, Debug)]
pub struct StdPrsPntOnIso {
    /// The actual spatial point.
    pub pnt: Point3,
    /// Parameter used for ordering, explicitly initialized rather than native indeterminate storage.
    pub param: f64,
}
/// Two inline points forming a segment; `segment.less_than(&other)`.
#[derive(Clone, Copy, Debug)]
pub struct StdPrsSegOnIso {
    /// Native array storage, including caller mutations observed through pointer conversions.
    pub pnts: [StdPrsPntOnIso; 2],
}
impl StdPrsSegOnIso {
    /// Borrow the actual native-style array pointer; `let pointer = segment.as_ptr()`.
    pub fn as_ptr(&self) -> *const StdPrsPntOnIso {
        // Valid element pointers alias Pnts; moved/destroyed object lifetime violations are undefined natively.
        self.pnts.as_ptr()
    }
    /// Return mutable inline storage without copying; `let pointer = segment.as_mut_ptr()`.
    pub fn as_mut_ptr(&mut self) -> *mut StdPrsPntOnIso {
        // Conversion is nonconst upstream and exposes the same two initialized elements.
        self.pnts.as_mut_ptr()
    }
    /// Compare intervals with native IEEE `<`; `segment.less_than(&other)`.
    pub fn less_than(&self, other: &Self) -> bool {
        // Comparing end against the other start is intentionally different from lexicographic sorting.
        self.pnts[1].param < other.pnts[0].param
    }
}
