// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/
// IntPatch_TheIWLineOfTheIWalking.hxx/_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_TheIWLineOfTheIWalking`: the walking line of `IntPatch_TheIWalking`, the
//! `IntWalk_TheIWLine` of the IntWalk template (docs/geomalgo-design.md section 10).
use super::failures::domain_error;
use crate::int_start::failures::sequence_value;
use crate::int_surf::{IntSurfCouple, IntSurfLineOn2S, IntSurfPathPoint, IntSurfPntOn2S};
use crate::int_walk::TheIWLine;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{BaseAllocator, INC_DEFAULT_BLOCK_SIZE};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::Vector3;

/// The fields of the line besides its points.
struct IWLineState {
    couple: Vec<IntSurfCouple>,
    closed: bool,
    has_first: bool,
    has_last: bool,
    first_index: i32,
    last_index: i32,
    the_first_point: IntSurfPathPoint,
    the_last_point: IntSurfPathPoint,
    indextg: i32,
    vcttg: Vector3,
    istgtbeg: bool,
    istgtend: bool,
}

/// `IntPatch_TheIWLineOfTheIWalking`: the points of a walking line, its start and end
/// path points, its passing points and its tangency data. It is shared through handles
/// and edited through them, so every member takes `&self`.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchTheIWLineOfTheIWalking;
/// let line = IntPatchTheIWLineOfTheIWalking::new(None);
/// assert_eq!(line.nb_points(), 0);
/// assert!(line.first_point().is_err());
/// ```
pub struct IntPatchTheIWLineOfTheIWalking {
    parent: StandardTransient,
    line: Handle<IntSurfLineOn2S>,
    state: SharedState<IWLineState>,
}
impl_transient!(
    IntPatchTheIWLineOfTheIWalking,
    "IntPatch_TheIWLineOfTheIWalking",
    StandardTransient,
    parent
);

const IWLINE: &str = "IntPatch_TheIWLineOfTheIWalking";

fn not_marching(member: &str, which: &str) -> Failure {
    domain_error(
        &format!("{IWLINE}::{member}"),
        &format!("the {which} point of the line is not a marching point"),
    )
}

impl IntPatchTheIWLineOfTheIWalking {
    /// `IntPatch_TheIWLineOfTheIWalking(theAllocator)`: an empty line whose points use
    /// the allocator (`None` is the native null allocator).
    /// `IntPatchTheIWLineOfTheIWalking::new(None)`.
    pub fn new(allocator: Option<BaseAllocator>) -> Self {
        let state = IWLineState {
            couple: Vec::new(),
            closed: false,
            has_first: false,
            has_last: false,
            first_index: -1,
            last_index: -1,
            the_first_point: IntSurfPathPoint::new(),
            the_last_point: IntSurfPathPoint::new(),
            indextg: -1,
            vcttg: Vector3::default(),
            istgtbeg: false,
            istgtend: false,
        };
        Self {
            parent: StandardTransient::default(),
            line: Handle::new(IntSurfLineOn2S::new(allocator)),
            state: SharedState::new(state),
        }
    }

    /// `Reverse()`: reverses the points and renumbers the passing points; the first and
    /// last flags are kept. `line.reverse()`.
    pub fn reverse(&self) {
        self.line.reverse();
        let n: i32 = self.line.nb_points();
        for couple in &mut self.state.write().couple {
            *couple = IntSurfCouple::with_indices(n - couple.first() + 1, couple.second());
        }
    }

    /// `Cut(Index)`: keeps the points before `index`. `line.cut(3)?`.
    pub fn cut(&self, index: i32) -> Result<(), Failure> {
        self.line.split(index).map(|_lost| ())
    }

    /// `AddPoint(P)`. `line.add_point(&p)`.
    pub fn add_point(&self, p: &IntSurfPntOn2S) {
        self.line.add(p);
    }

    /// `AddStatusFirst(Closed, HasFirst)`. `line.add_status_first(false, true)`.
    pub fn add_status_first(&self, closed: bool, has_first: bool) {
        let mut s = self.state.write();
        (s.closed, s.has_first) = (closed, has_first);
    }

    /// `AddStatusFirst(Closed, HasFirst, Index, P)`.
    /// `line.add_status_first_point(false, true, 1, &path_point)`.
    pub fn add_status_first_point(
        &self,
        closed: bool,
        has_first: bool,
        index: i32,
        p: &IntSurfPathPoint,
    ) {
        let mut s = self.state.write();
        (s.closed, s.has_first, s.first_index) = (closed, has_first, index);
        s.the_first_point = p.clone();
    }

    /// `AddStatusFirstLast(Closed, HasFirst, HasLast)`.
    /// `line.add_status_first_last(true, false, false)`.
    pub fn add_status_first_last(&self, closed: bool, has_first: bool, has_last: bool) {
        let mut s = self.state.write();
        (s.closed, s.has_first, s.has_last) = (closed, has_first, has_last);
    }

    /// `AddStatusLast(HasLast)`. `line.add_status_last(true)`.
    pub fn add_status_last(&self, has_last: bool) {
        self.state.write().has_last = has_last;
    }

    /// `AddStatusLast(HasLast, Index, P)`. `line.add_status_last_point(true, 2, &path_point)`.
    pub fn add_status_last_point(&self, has_last: bool, index: i32, p: &IntSurfPathPoint) {
        let mut s = self.state.write();
        (s.has_last, s.last_index) = (has_last, index);
        s.the_last_point = p.clone();
    }

    /// `AddIndexPassing(Index)`: the next point of the line passes through the start
    /// point `index`. `line.add_index_passing(2)`.
    pub fn add_index_passing(&self, index: i32) {
        let n: i32 = self.line.nb_points();
        self.state
            .write()
            .couple
            .push(IntSurfCouple::with_indices(n + 1, index));
    }

    /// `SetTangentVector(V, Index)`. `line.set_tangent_vector(v, 1)`.
    pub fn set_tangent_vector(&self, v: Vector3, index: i32) {
        let mut s = self.state.write();
        (s.indextg, s.vcttg) = (index, v);
    }

    /// `SetTangencyAtBegining(IsTangent)`. `line.set_tangency_at_begining(true)`.
    pub fn set_tangency_at_begining(&self, is_tangent: bool) {
        self.state.write().istgtbeg = is_tangent;
    }

    /// `SetTangencyAtEnd(IsTangent)`. `line.set_tangency_at_end(true)`.
    pub fn set_tangency_at_end(&self, is_tangent: bool) {
        self.state.write().istgtend = is_tangent;
    }

    /// `NbPoints()`. `line.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.line.nb_points()
    }

    /// `Value(Index)`: Standard_OutOfRange outside `1..=NbPoints()`. `line.value(1)?`.
    pub fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.line.value(index)
    }

    /// `Line()`: the points. `line.line()`.
    pub fn line(&self) -> Handle<IntSurfLineOn2S> {
        self.line.clone()
    }

    /// `IsClosed()`. `line.is_closed()`.
    pub fn is_closed(&self) -> bool {
        self.state.read().closed
    }

    /// `HasFirstPoint()`. `line.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.state.read().has_first
    }

    /// `FirstPoint()`: Standard_DomainError unless the line starts on a marching point.
    /// `line.first_point()?`.
    pub fn first_point(&self) -> Result<IntSurfPathPoint, Failure> {
        let s = self.state.read();
        if !s.has_first {
            return Err(not_marching("FirstPoint", "first"));
        }
        Ok(s.the_first_point.clone())
    }

    /// `FirstPointIndex()`: Standard_DomainError unless the line starts on a marching
    /// point. `line.first_point_index()?`.
    pub fn first_point_index(&self) -> Result<i32, Failure> {
        let s = self.state.read();
        if !s.has_first {
            return Err(not_marching("FirstPointIndex", "first"));
        }
        Ok(s.first_index)
    }

    /// `HasLastPoint()`. `line.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        self.state.read().has_last
    }

    /// `LastPoint()`: Standard_DomainError unless the line ends on a marching point.
    /// `line.last_point()?`.
    pub fn last_point(&self) -> Result<IntSurfPathPoint, Failure> {
        let s = self.state.read();
        if !s.has_last {
            return Err(not_marching("LastPoint", "last"));
        }
        Ok(s.the_last_point.clone())
    }

    /// `LastPointIndex()`: Standard_DomainError unless the line ends on a marching
    /// point. `line.last_point_index()?`.
    pub fn last_point_index(&self) -> Result<i32, Failure> {
        let s = self.state.read();
        if !s.has_last {
            return Err(not_marching("LastPointIndex", "last"));
        }
        Ok(s.last_index)
    }

    /// `NbPassingPoint()`. `line.nb_passing_point()`.
    pub fn nb_passing_point(&self) -> i32 {
        self.state.read().couple.len() as i32
    }

    /// `PassingPoint(Index, IndexLine, IndexPnts)`: the index on the line and the index
    /// of the start point; Standard_OutOfRange outside `1..=NbPassingPoint()`.
    /// `line.passing_point(1)?`.
    pub fn passing_point(&self, index: i32) -> Result<(i32, i32), Failure> {
        let s = self.state.read();
        let member: String = format!("{IWLINE}::PassingPoint");
        let couple: &IntSurfCouple = sequence_value(&s.couple, index, &member)?;
        Ok((couple.first(), couple.second()))
    }

    /// `TangentVector(Index)`: the vector and its index. `line.tangent_vector()`.
    pub fn tangent_vector(&self) -> (Vector3, i32) {
        let s = self.state.read();
        (s.vcttg, s.indextg)
    }

    /// `IsTangentAtBegining()`. `line.is_tangent_at_begining()`.
    pub fn is_tangent_at_begining(&self) -> bool {
        self.state.read().istgtbeg
    }

    /// `IsTangentAtEnd()`. `line.is_tangent_at_end()`.
    pub fn is_tangent_at_end(&self) -> bool {
        self.state.read().istgtend
    }
}

/// `IntWalk_TheIWLine` of the `IntPatch_TheIWalking` instance.
impl TheIWLine for IntPatchTheIWLineOfTheIWalking {
    /// `new IntWalk_TheIWLine(new NCollection_IncAllocator())`.
    fn with_inc_allocator() -> Self {
        Self::new(Some(BaseAllocator::incremental(INC_DEFAULT_BLOCK_SIZE)))
    }
    fn reverse(&self) {
        IntPatchTheIWLineOfTheIWalking::reverse(self);
    }
    fn cut(&self, index: i32) -> Result<(), Failure> {
        IntPatchTheIWLineOfTheIWalking::cut(self, index)
    }
    fn add_point(&self, p: &IntSurfPntOn2S) {
        IntPatchTheIWLineOfTheIWalking::add_point(self, p);
    }
    fn add_status_first(&self, closed: bool, has_first: bool) {
        IntPatchTheIWLineOfTheIWalking::add_status_first(self, closed, has_first);
    }
    fn add_status_first_point(
        &self,
        closed: bool,
        has_first: bool,
        index: i32,
        p: &IntSurfPathPoint,
    ) {
        IntPatchTheIWLineOfTheIWalking::add_status_first_point(self, closed, has_first, index, p);
    }
    fn add_status_first_last(&self, closed: bool, has_first: bool, has_last: bool) {
        IntPatchTheIWLineOfTheIWalking::add_status_first_last(self, closed, has_first, has_last);
    }
    fn add_status_last(&self, has_last: bool) {
        IntPatchTheIWLineOfTheIWalking::add_status_last(self, has_last);
    }
    fn add_status_last_point(&self, has_last: bool, index: i32, p: &IntSurfPathPoint) {
        IntPatchTheIWLineOfTheIWalking::add_status_last_point(self, has_last, index, p);
    }
    fn add_index_passing(&self, index: i32) {
        IntPatchTheIWLineOfTheIWalking::add_index_passing(self, index);
    }
    fn set_tangent_vector(&self, v: Vector3, index: i32) {
        IntPatchTheIWLineOfTheIWalking::set_tangent_vector(self, v, index);
    }
    fn set_tangency_at_begining(&self, is_tangent: bool) {
        IntPatchTheIWLineOfTheIWalking::set_tangency_at_begining(self, is_tangent);
    }
    fn set_tangency_at_end(&self, is_tangent: bool) {
        IntPatchTheIWLineOfTheIWalking::set_tangency_at_end(self, is_tangent);
    }
    fn nb_points(&self) -> i32 {
        IntPatchTheIWLineOfTheIWalking::nb_points(self)
    }
    fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        IntPatchTheIWLineOfTheIWalking::value(self, index)
    }
    fn line(&self) -> Handle<IntSurfLineOn2S> {
        self.line.clone()
    }
    fn tangent_vector(&self) -> (Vector3, i32) {
        IntPatchTheIWLineOfTheIWalking::tangent_vector(self)
    }
    fn is_tangent_at_begining(&self) -> bool {
        IntPatchTheIWLineOfTheIWalking::is_tangent_at_begining(self)
    }
    fn is_tangent_at_end(&self) -> bool {
        IntPatchTheIWLineOfTheIWalking::is_tangent_at_end(self)
    }
}
