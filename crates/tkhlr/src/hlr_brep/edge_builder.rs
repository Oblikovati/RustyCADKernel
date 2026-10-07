//! HLRBRep_EdgeBuilder.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{AreaLimit, VertexList};
use crate::hlr_algo::Intersection;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, State};

/// HLRBRep_EdgeBuilder: the areas of an edge between consecutive vertices of a
/// vertex list, with their states against a face and the edge, and the walk on the
/// parts of the edge whose areas have a state (`Builds`, `MoreEdges`,
/// `MoreVertices` ...). The file is compiled with `No_Exception`: the empty list and
/// the list without interference are not refused, and the first walk on an empty
/// list fails with Standard_NullObject where the native code reads a null handle.
pub struct EdgeBuilder {
    to_build: State,
    limits: Handle<AreaLimit>,
    left: Handle<AreaLimit>,
    right: Handle<AreaLimit>,
    current: i32,
}

fn same(a: &Handle<AreaLimit>, b: &Handle<AreaLimit>) -> bool {
    a.object_address() == b.object_address()
}

fn limit(h: &Handle<AreaLimit>) -> Result<&AreaLimit, Failure> {
    h.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "HLRBRep_EdgeBuilder: a null area limit; expected a vertex list with vertices",
        )
    })
}

// The states before and after an edge end or interference of the given orientation.
fn states(o: Orientation) -> (State, State) {
    match o {
        Orientation::Forward => (State::Out, State::In),
        Orientation::Reversed => (State::In, State::Out),
        Orientation::Internal => (State::In, State::In),
        Orientation::External => (State::Out, State::Out),
    }
}

// The area limit of the current vertex of the list.
fn area_limit(v_list: &VertexList<'_>) -> Result<Handle<AreaLimit>, Failure> {
    let (mut before, mut after, mut ebefore, mut eafter) = (
        State::Unknown,
        State::Unknown,
        State::Unknown,
        State::Unknown,
    );
    if v_list.is_boundary() {
        (ebefore, eafter) = states(v_list.orientation()?);
    }
    if v_list.is_interference() {
        (before, after) = states(v_list.transition()?);
        match v_list.boundary_transition()? {
            Orientation::Forward => after = State::On,
            Orientation::Reversed => before = State::On,
            Orientation::Internal => (before, after) = (State::On, State::On),
            Orientation::External => {}
        }
    }
    let v: Intersection = *v_list.current()?;
    Ok(Handle::new(AreaLimit::new(
        &v,
        v_list.is_boundary(),
        v_list.is_interference(),
        before,
        after,
        ebefore,
        eafter,
    )))
}

impl EdgeBuilder {
    /// HLRBRep_EdgeBuilder(VList): the area limits of the vertices of `v_list`
    /// (a circular list on a periodic edge), the unknown states propagated from the
    /// known ones, then `Builds(IN)`; `EdgeBuilder::new(&mut list)?`.
    pub fn new(v_list: &mut VertexList<'_>) -> Result<Self, Failure> {
        let mut eb = Self {
            to_build: State::Unknown,
            limits: Handle::default(),
            left: Handle::default(),
            right: Handle::default(),
            current: 0,
        };
        let mut last: Handle<AreaLimit> = Handle::default();
        while v_list.more() {
            let cur = area_limit(v_list)?;
            if eb.limits.is_null() {
                eb.limits = cur.clone();
            } else {
                limit(&last)?.set_next(&cur);
                limit(&cur)?.set_previous(&last);
            }
            last = cur;
            v_list.next()?;
        }
        if v_list.is_periodic() {
            limit(&last)?.set_next(&eb.limits);
            limit(&eb.limits)?.set_previous(&last);
        }
        eb.propagate_states()?;
        eb.builds(State::In)?;
        Ok(eb)
    }

    // The first known states, then each unknown state from the previous limit.
    fn propagate_states(&self) -> Result<(), Failure> {
        let (mut stat, mut estat) = (State::Unknown, State::Unknown);
        let mut cur = self.limits.clone();
        while let Some(c) = cur.get() {
            if stat == State::Unknown {
                stat = c.state_before();
                if stat == State::Unknown {
                    stat = c.state_after();
                }
            }
            if estat == State::Unknown {
                estat = c.edge_before();
                if estat == State::Unknown {
                    estat = c.edge_after();
                }
            }
            cur = c.next();
            if same(&cur, &self.limits) {
                break;
            }
        }
        // If no boundary the edge covers the whole curve.
        if estat == State::Unknown {
            estat = State::In;
        }
        let mut cur = self.limits.clone();
        while let Some(c) = cur.get() {
            if c.state_before() == State::Unknown {
                c.set_state_before(stat);
            } else {
                stat = c.state_after();
            }
            if c.state_after() == State::Unknown {
                c.set_state_after(stat);
            }
            if c.edge_before() == State::Unknown {
                c.set_edge_before(estat);
            } else {
                estat = c.edge_after();
            }
            if c.edge_after() == State::Unknown {
                c.set_edge_after(estat);
            }
            cur = c.next();
            if same(&cur, &self.limits) {
                break;
            }
        }
        Ok(())
    }

    /// InitAreas(): the area before the first limit; `eb.init_areas()?`.
    pub fn init_areas(&mut self) -> Result<(), Failure> {
        self.left = limit(&self.limits)?.previous();
        self.right = self.limits.clone();
        Ok(())
    }

    /// NextArea(); `eb.next_area()`.
    pub fn next_area(&mut self) {
        self.left = self.right.clone();
        if let Some(r) = self.right.get() {
            self.right = r.next();
        }
    }

    /// PreviousArea(); `eb.previous_area()`.
    pub fn previous_area(&mut self) {
        self.right = self.left.clone();
        if let Some(l) = self.left.get() {
            self.left = l.previous();
        }
    }

    /// HasArea(); `eb.has_area()`.
    pub fn has_area(&self) -> bool {
        if self.left.is_null() && self.right.is_null() {
            return false;
        }
        !same(&self.right, &self.limits)
    }

    /// AreaState(): the state of the current area against the face;
    /// `eb.area_state()`.
    pub fn area_state(&self) -> State {
        let mut stat = State::Unknown;
        if let Some(l) = self.left.get() {
            stat = l.state_after();
        }
        if let Some(r) = self.right.get() {
            stat = r.state_before();
        }
        stat
    }

    /// AreaEdgeState(): the state of the current area against the edge;
    /// `eb.area_edge_state()`.
    pub fn area_edge_state(&self) -> State {
        let mut stat = State::Unknown;
        if let Some(l) = self.left.get() {
            stat = l.edge_after();
        }
        if let Some(r) = self.right.get() {
            stat = r.edge_before();
        }
        stat
    }

    /// LeftLimit(); `eb.left_limit()`.
    pub fn left_limit(&self) -> Handle<AreaLimit> {
        self.left.clone()
    }

    /// RightLimit(); `eb.right_limit()`.
    pub fn right_limit(&self) -> Handle<AreaLimit> {
        self.right.clone()
    }

    // The current part starts at the current area: at its left limit, or at its
    // right one when it has no left limit.
    fn start_part(&mut self) {
        self.current = if self.left.is_null() { 2 } else { 1 };
    }

    fn area_to_build(&self) -> bool {
        self.area_state() == self.to_build && self.area_edge_state() == State::In
    }

    /// Builds(ToBuild): the first part of the edge whose areas have the state
    /// `to_build`; `eb.builds(State::In)?`.
    pub fn builds(&mut self, to_build: State) -> Result<(), Failure> {
        self.to_build = to_build;
        self.init_areas()?;
        loop {
            if self.area_to_build() {
                self.start_part();
                return Ok(());
            }
            self.next_area();
            if !self.has_area() {
                break;
            }
        }
        self.current = 3;
        Ok(())
    }

    /// MoreEdges(); `eb.more_edges()`.
    pub fn more_edges(&self) -> bool {
        self.has_area()
    }

    /// NextEdge(): the next part; `eb.next_edge()`.
    pub fn next_edge(&mut self) {
        // Clean the current edge.
        while self.area_state() == self.to_build {
            self.next_area();
        }
        // Go to the next edge.
        while self.has_area() {
            if self.area_to_build() {
                self.start_part();
                return;
            }
            self.next_area();
        }
    }

    /// MoreVertices(); `eb.more_vertices()`.
    pub fn more_vertices(&self) -> bool {
        self.current < 3
    }

    /// NextVertex(): the end of the current part; `eb.next_vertex()?`.
    pub fn next_vertex(&mut self) -> Result<(), Failure> {
        match self.current {
            1 => {
                self.current = if self.right.is_null() { 3 } else { 2 };
            }
            2 => {
                self.next_area();
                self.current = if self.area_to_build() { 2 } else { 3 };
            }
            _ => return Err(no_vertex("NextVertex : No current edge")),
        }
        Ok(())
    }

    fn current_limit(&self, member: &str) -> Result<&AreaLimit, Failure> {
        match self.current {
            1 => limit(&self.left),
            2 => limit(&self.right),
            _ => Err(no_vertex(&format!("{member} : No current vertex"))),
        }
    }

    /// Current(): the vertex of the current limit; `eb.current()?`.
    pub fn current(&self) -> Result<Intersection, Failure> {
        Ok(*self.current_limit("Current")?.vertex())
    }

    /// IsBoundary(); `eb.is_boundary()?`.
    pub fn is_boundary(&self) -> Result<bool, Failure> {
        Ok(self.current_limit("IsBoundary")?.is_boundary())
    }

    /// IsInterference(); `eb.is_interference()?`.
    pub fn is_interference(&self) -> Result<bool, Failure> {
        Ok(self.current_limit("IsInterference")?.is_interference())
    }

    /// Orientation(): FORWARD at the start of the part, REVERSED at its end,
    /// INTERNAL at a limit without change of state; `eb.orientation()?`.
    pub fn orientation(&self) -> Result<Orientation, Failure> {
        let (l, end) = match self.current {
            1 => (limit(&self.left)?, Orientation::Forward),
            2 => (limit(&self.right)?, Orientation::Reversed),
            _ => return Ok(Orientation::External),
        };
        if l.state_before() == l.state_after() && l.edge_before() == l.edge_after() {
            return Ok(Orientation::Internal);
        }
        Ok(end)
    }

    /// Destroy(): unlinks the area limits (the native destructor calls it);
    /// `eb.destroy()`.
    pub fn destroy(&mut self) {
        let mut cur = self.limits.clone();
        while let Some(c) = cur.get() {
            let n = c.next();
            c.clear();
            cur = n;
        }
        self.left = Handle::default();
        self.right = Handle::default();
        self.limits = Handle::default();
    }
}

impl Drop for EdgeBuilder {
    fn drop(&mut self) {
        self.destroy();
    }
}

fn no_vertex(what: &str) -> Failure {
    Failure::with_message("Standard_NoSuchObject", &format!("EdgeBuilder::{what}"))
}
