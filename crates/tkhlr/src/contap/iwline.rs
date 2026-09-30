// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_TheIWLineOfTheIWalking.hxx and Contap_TheIWLineOfTheIWalking_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_TheIWLineOfTheIWalking: the walking line of the contour computation.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkgeomalgo::int_patch::IntPatchTheIWLineOfTheIWalking;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPathPoint, IntSurfPntOn2S};
use rustycad_tkgeomalgo::int_walk::TheIWLine;
use rustycad_tkmath::gp::Vector3;

/// Contap_TheIWLineOfTheIWalking: the `IntWalk_IWLine.gxx` instance of Contap. Its body
/// is the one of `IntPatch_TheIWLineOfTheIWalking` (the same template on the same
/// types), so the port keeps an IntPatch line and forwards matching members. Contap has its
/// own runtime type and its beginning-tangency setter writes the end flag.
///
/// ```
/// use rustycad_tkhlr::contap::ContapTheIWLineOfTheIWalking;
/// let line = ContapTheIWLineOfTheIWalking::new(None);
/// assert_eq!(line.nb_points(), 0);
/// assert!(!line.has_first_point());
/// ```
pub struct ContapTheIWLineOfTheIWalking {
    parent: StandardTransient,
    line: IntPatchTheIWLineOfTheIWalking,
}
impl_transient!(
    ContapTheIWLineOfTheIWalking,
    "Contap_TheIWLineOfTheIWalking",
    StandardTransient,
    parent
);

impl ContapTheIWLineOfTheIWalking {
    /// Contap_TheIWLineOfTheIWalking(theAllocator): an empty line whose points use the
    /// allocator (`None` is the native null allocator);
    /// `ContapTheIWLineOfTheIWalking::new(None)`.
    pub fn new(allocator: Option<BaseAllocator>) -> Self {
        Self {
            parent: StandardTransient::default(),
            line: IntPatchTheIWLineOfTheIWalking::new(allocator),
        }
    }

    /// Reverse; `line.reverse()`.
    pub fn reverse(&self) {
        self.line.reverse();
    }

    /// Cut(Index): keeps the points before `index`; `line.cut(3)?`.
    pub fn cut(&self, index: i32) -> Result<(), Failure> {
        self.line.cut(index)
    }

    /// AddPoint; `line.add_point(&p)`.
    pub fn add_point(&self, p: &IntSurfPntOn2S) {
        self.line.add_point(p);
    }

    /// AddStatusFirst(Closed, HasFirst); `line.add_status_first(false, true)`.
    pub fn add_status_first(&self, closed: bool, has_first: bool) {
        self.line.add_status_first(closed, has_first);
    }

    /// AddStatusFirst(Closed, HasLast, Index, P); `line.add_status_first_point(false, true, 1, &p)`.
    pub fn add_status_first_point(
        &self,
        closed: bool,
        has_first: bool,
        index: i32,
        p: &IntSurfPathPoint,
    ) {
        self.line
            .add_status_first_point(closed, has_first, index, p);
    }

    /// AddStatusFirstLast; `line.add_status_first_last(true, false, false)`.
    pub fn add_status_first_last(&self, closed: bool, has_first: bool, has_last: bool) {
        self.line.add_status_first_last(closed, has_first, has_last);
    }

    /// AddStatusLast(HasLast); `line.add_status_last(false)`.
    pub fn add_status_last(&self, has_last: bool) {
        self.line.add_status_last(has_last);
    }

    /// AddStatusLast(HasLast, Index, P); `line.add_status_last_point(true, 1, &p)`.
    pub fn add_status_last_point(&self, has_last: bool, index: i32, p: &IntSurfPathPoint) {
        self.line.add_status_last_point(has_last, index, p);
    }

    /// AddIndexPassing; `line.add_index_passing(2)`.
    pub fn add_index_passing(&self, index: i32) {
        self.line.add_index_passing(index);
    }

    /// SetTangentVector; `line.set_tangent_vector(v, 1)`.
    pub fn set_tangent_vector(&self, v: Vector3, index: i32) {
        self.line.set_tangent_vector(v, index);
    }

    /// SetTangencyAtBegining; `line.set_tangency_at_begining(true)`.
    pub fn set_tangency_at_begining(&self, is_tangent: bool) {
        // OCCT 3d097a0: Contap header writes istgtend here, unlike IntPatch.
        self.line.set_tangency_at_end(is_tangent);
    }

    /// SetTangencyAtEnd; `line.set_tangency_at_end(true)`.
    pub fn set_tangency_at_end(&self, is_tangent: bool) {
        self.line.set_tangency_at_end(is_tangent);
    }

    /// NbPoints; `line.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.line.nb_points()
    }

    /// Value(Index): a copy of a point; `line.value(1)?`.
    pub fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.line.value(index)
    }

    /// Line: the points (shared); `line.line()`.
    pub fn line(&self) -> Handle<IntSurfLineOn2S> {
        self.line.line()
    }

    /// IsClosed; `line.is_closed()`.
    pub fn is_closed(&self) -> bool {
        self.line.is_closed()
    }

    /// HasFirstPoint; `line.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.line.has_first_point()
    }

    /// HasLastPoint; `line.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        self.line.has_last_point()
    }

    /// FirstPoint: Standard_DomainError unless the first point is a path point;
    /// `line.first_point()?`.
    pub fn first_point(&self) -> Result<IntSurfPathPoint, Failure> {
        self.line.first_point()
    }

    /// FirstPointIndex; `line.first_point_index()?`.
    pub fn first_point_index(&self) -> Result<i32, Failure> {
        self.line.first_point_index()
    }

    /// LastPoint; `line.last_point()?`.
    pub fn last_point(&self) -> Result<IntSurfPathPoint, Failure> {
        self.line.last_point()
    }

    /// LastPointIndex; `line.last_point_index()?`.
    pub fn last_point_index(&self) -> Result<i32, Failure> {
        self.line.last_point_index()
    }

    /// NbPassingPoint; `line.nb_passing_point()`.
    pub fn nb_passing_point(&self) -> i32 {
        self.line.nb_passing_point()
    }

    /// PassingPoint(Index, IndexLine, IndexPnts); `line.passing_point(1)?`.
    pub fn passing_point(&self, index: i32) -> Result<(i32, i32), Failure> {
        self.line.passing_point(index)
    }

    /// TangentVector(Index): the vector and its point index; `line.tangent_vector()`.
    pub fn tangent_vector(&self) -> (Vector3, i32) {
        self.line.tangent_vector()
    }

    /// IsTangentAtBegining; `line.is_tangent_at_begining()`.
    pub fn is_tangent_at_begining(&self) -> bool {
        self.line.is_tangent_at_begining()
    }

    /// IsTangentAtEnd; `line.is_tangent_at_end()`.
    pub fn is_tangent_at_end(&self) -> bool {
        self.line.is_tangent_at_end()
    }
}

/// `IntWalk_TheIWLine` of `Contap_TheIWalking`.
impl TheIWLine for ContapTheIWLineOfTheIWalking {
    fn with_inc_allocator() -> Self {
        Self {
            parent: StandardTransient::default(),
            line: <IntPatchTheIWLineOfTheIWalking as TheIWLine>::with_inc_allocator(),
        }
    }
    fn reverse(&self) {
        self.line.reverse();
    }
    fn cut(&self, index: i32) -> Result<(), Failure> {
        self.line.cut(index)
    }
    fn add_point(&self, p: &IntSurfPntOn2S) {
        self.line.add_point(p);
    }
    fn add_status_first(&self, closed: bool, has_first: bool) {
        self.line.add_status_first(closed, has_first);
    }
    fn add_status_first_point(
        &self,
        closed: bool,
        has_first: bool,
        index: i32,
        p: &IntSurfPathPoint,
    ) {
        self.line
            .add_status_first_point(closed, has_first, index, p);
    }
    fn add_status_first_last(&self, closed: bool, has_first: bool, has_last: bool) {
        self.line.add_status_first_last(closed, has_first, has_last);
    }
    fn add_status_last(&self, has_last: bool) {
        self.line.add_status_last(has_last);
    }
    fn add_status_last_point(&self, has_last: bool, index: i32, p: &IntSurfPathPoint) {
        self.line.add_status_last_point(has_last, index, p);
    }
    fn add_index_passing(&self, index: i32) {
        self.line.add_index_passing(index);
    }
    fn set_tangent_vector(&self, v: Vector3, index: i32) {
        self.line.set_tangent_vector(v, index);
    }
    fn set_tangency_at_begining(&self, is_tangent: bool) {
        // OCCT 3d097a0: Contap header writes istgtend here, unlike IntPatch.
        self.line.set_tangency_at_end(is_tangent);
    }
    fn set_tangency_at_end(&self, is_tangent: bool) {
        self.line.set_tangency_at_end(is_tangent);
    }
    fn nb_points(&self) -> i32 {
        self.line.nb_points()
    }
    fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.line.value(index)
    }
    fn line(&self) -> Handle<IntSurfLineOn2S> {
        self.line.line()
    }
    fn tangent_vector(&self) -> (Vector3, i32) {
        self.line.tangent_vector()
    }
    fn is_tangent_at_begining(&self) -> bool {
        self.line.is_tangent_at_begining()
    }
    fn is_tangent_at_end(&self) -> bool {
        self.line.is_tangent_at_end()
    }
}
