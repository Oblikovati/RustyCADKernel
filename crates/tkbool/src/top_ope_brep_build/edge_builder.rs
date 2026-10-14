// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_PaveClassifier,
// TopOpeBRepBuild_Area1dBuilder and TopOpeBRepBuild_EdgeBuilder (.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge builder: the paves of an edge grouped into edges (areas of the 1d area
//! builder) by the pave classifier.
use super::area_builder::TopOpeBRepBuildAreaBuilder;
use super::loops::{TopOpeBRepBuildLoop, TopOpeBRepBuildLoopClassifier, TopOpeBRepBuildLoopSet};
use super::pave::TopOpeBRepBuildPave;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::elclib::in_period;
use std::ops::{Deref, DerefMut};

/// The classifier of the paves of an edge (`TopOpeBRepBuild_PaveClassifier`); natively
/// the first parameter and period of a non-periodic edge, the compared orientations,
/// parameters and cases start uninitialized (0, FORWARD here);
/// `let mut pc = TopOpeBRepBuildPaveClassifier::new(&edge)?;`.
#[derive(Clone)]
pub struct TopOpeBRepBuildPaveClassifier {
    my_edge: Edge,
    my_edge_periodic: bool,
    my_first: f64,
    my_period: f64,
    my_same_parameters: bool,
    my_closed_vertices: bool,
    my_cas1: i32,
    my_cas2: i32,
    my_o1: Orientation,
    my_o2: Orientation,
    my_p1: f64,
    my_p2: f64,
}

impl TopOpeBRepBuildPaveClassifier {
    /// `TopOpeBRepBuild_PaveClassifier(E)`: the period of the curve of a periodic
    /// non-degenerated edge (with its vertex parameter as first for a closed edge, the
    /// edge range without vertices); `TopOpeBRepBuildPaveClassifier::new(&edge)?`.
    pub fn new(e: &Shape) -> Result<Self, Failure> {
        let mut pc = Self {
            my_edge: topods::edge(e)?,
            my_edge_periodic: false,
            my_first: 0.,
            my_period: 0.,
            my_same_parameters: false,
            my_closed_vertices: false,
            my_cas1: 0,
            my_cas2: 0,
            my_o1: Orientation::Forward,
            my_o2: Orientation::Forward,
            my_p1: 0.,
            my_p2: 0.,
        };
        if BRepTool::degenerated(&pc.my_edge)? {
            return Ok(pc);
        }
        let (c, f, l) = BRepTool::curve(&pc.my_edge)?;
        let Some(cr) = c.get() else {
            return Ok(pc);
        };
        if !cr.is_periodic() {
            return Ok(pc);
        }
        let (v1, v2) = topexp::vertices(&pc.my_edge, false);
        if !v1.is_null() && !v2.is_null() {
            pc.my_first = f;
            pc.my_period = cr.last_parameter() - cr.first_parameter();
            pc.my_same_parameters = v1.is_same(&v2);
            pc.my_edge_periodic = pc.my_same_parameters;
            if pc.my_same_parameters {
                pc.my_first = BRepTool::parameter(&v1, &pc.my_edge)?;
            }
        } else {
            (
                pc.my_first,
                pc.my_period,
                pc.my_edge_periodic,
                pc.my_same_parameters,
            ) = (f, l - f, true, false);
        }
        Ok(pc)
    }

    /// `CompareOnNonPeriodic()` (private): by the orientation of the second vertex and
    /// the order of the parameters.
    fn compare_on_non_periodic(&self) -> State {
        let lower = match self.my_o2 {
            Orientation::Forward => false,
            Orientation::Reversed => true,
            Orientation::Internal => return State::In,
            Orientation::External => return State::Out,
        };
        if self.my_p1 == self.my_p2 {
            return if self.my_o1 == self.my_o2 {
                State::In
            } else {
                State::Out
            };
        }
        if (self.my_p1 < self.my_p2) == lower {
            State::In
        } else {
            State::Out
        }
    }

    /// `AdjustCase(p1, o, first, period, tol, cas)`: `p1` moved by a period after the
    /// first (REVERSED at first), kept at first or last, else brought into the period;
    /// with the case 1 to 4; `pc.adjust_case(0.0, Orientation::Reversed, 0.0, 6.28, 1e-9)`.
    pub fn adjust_case(
        &self,
        p1: f64,
        o: Orientation,
        first: f64,
        period: f64,
        tol: f64,
    ) -> (f64, i32) {
        if (p1 - first).abs() < tol {
            return if o == Orientation::Reversed {
                (p1 + period, 1)
            } else {
                (p1, 2)
            };
        }
        let last = first + period;
        if (p1 - last).abs() < tol {
            return (p1, 3);
        }
        (in_period(p1, first, last), 4)
    }

    /// `AdjustOnPeriodic()` (private).
    fn adjust_on_periodic(&mut self) {
        if !self.to_adjust_on_periodic() {
            return;
        }
        let tol = precision::p_confusion();
        if self.my_same_parameters {
            (self.my_p1, self.my_cas1) =
                self.adjust_case(self.my_p1, self.my_o1, self.my_first, self.my_period, tol);
            (self.my_p2, self.my_cas2) =
                self.adjust_case(self.my_p2, self.my_o2, self.my_first, self.my_period, tol);
        } else if self.my_o1 != self.my_o2 {
            if self.my_o1 == Orientation::Forward {
                (self.my_p2, self.my_cas2) =
                    self.adjust_case(self.my_p2, self.my_o2, self.my_p1, self.my_period, tol);
            }
            if self.my_o2 == Orientation::Forward {
                (self.my_p1, self.my_cas1) =
                    self.adjust_case(self.my_p1, self.my_o1, self.my_p2, self.my_period, tol);
            }
        }
    }

    /// `ToAdjustOnPeriodic()` (private).
    fn to_adjust_on_periodic(&self) -> bool {
        self.my_same_parameters || self.my_o1 != self.my_o2
    }

    /// `CompareOnPeriodic()` (private).
    fn compare_on_periodic(&mut self) -> State {
        if self.to_adjust_on_periodic() {
            return self.compare_on_non_periodic();
        }
        let cas = match self.my_o1 {
            Orientation::Forward => 5,
            Orientation::Reversed => 6,
            _ => 7,
        };
        (self.my_cas1, self.my_cas2) = (cas, cas);
        State::Out
    }

    /// `SetFirstParameter(P)`: the first parameter, with same parameters;
    /// `pc.set_first_parameter(0.0)`.
    pub fn set_first_parameter(&mut self, p: f64) {
        self.my_first = p;
        self.my_same_parameters = true;
    }

    /// `ClosedVertices(Closed)`; `pc.closed_vertices(true)`.
    pub fn closed_vertices(&mut self, closed: bool) {
        self.my_closed_vertices = closed;
    }
}

/// The pave of a loop (natively a reinterpreting cast).
fn pave(l: &Handle<dyn TopOpeBRepBuildLoop>) -> Result<Handle<TopOpeBRepBuildPave>, Failure> {
    let pv: Handle<TopOpeBRepBuildPave> = l.cast();
    if pv.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "TopOpeBRepBuild_PaveClassifier: the loop is no TopOpeBRepBuild_Pave",
            "TopOpeBRepBuild_PaveClassifier",
        ));
    }
    Ok(pv)
}

impl TopOpeBRepBuildLoopClassifier for TopOpeBRepBuildPaveClassifier {
    /// `Compare(L1, L2)`: the state of the pave `l1` relative to the pave `l2` along the
    /// edge (adjusted on a periodic edge).
    fn compare(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        let (pv1, pv2) = (pave(l1)?, pave(l2)?);
        let (p1, p2) = (pv1.get().unwrap(), pv2.get().unwrap());
        (self.my_cas1, self.my_cas2) = (0, 0);
        (self.my_o1, self.my_o2) = (p1.vertex().orientation(), p2.vertex().orientation());
        (self.my_p1, self.my_p2) = (p1.parameter(), p2.parameter());
        if self.my_edge_periodic && self.to_adjust_on_periodic() {
            self.adjust_on_periodic();
        }
        Ok(if self.my_edge_periodic {
            self.compare_on_periodic()
        } else {
            self.compare_on_non_periodic()
        })
    }
}

/// The 1d area builder (`TopOpeBRepBuild_Area1dBuilder`): the areas of the base
/// algorithm; `TopOpeBRepBuildArea1dBuilder::with_loops(&mut ps, &mut pc, false)?`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildArea1dBuilder {
    parent: TopOpeBRepBuildAreaBuilder,
}

impl Deref for TopOpeBRepBuildArea1dBuilder {
    type Target = TopOpeBRepBuildAreaBuilder;

    fn deref(&self) -> &TopOpeBRepBuildAreaBuilder {
        &self.parent
    }
}

impl DerefMut for TopOpeBRepBuildArea1dBuilder {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildAreaBuilder {
        &mut self.parent
    }
}

impl TopOpeBRepBuildArea1dBuilder {
    /// `TopOpeBRepBuild_Area1dBuilder()`; `TopOpeBRepBuildArea1dBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_Area1dBuilder(LS, LC, ForceClass)`;
    /// `TopOpeBRepBuildArea1dBuilder::with_loops(&mut ps, &mut pc, false)?`.
    pub fn with_loops(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut ab = Self::new();
        ab.init_area_builder(ls, lc, force_class)?;
        Ok(ab)
    }

    /// `InitAreaBuilder(LS, LC, ForceClass)`: the areas of the 1d algorithm, which is
    /// the base one (shape loops into the areas of block loops containing them, block
    /// loops moving out the loops outside them); `ab.init_area_builder(&mut ps, &mut pc, false)?`.
    pub fn init_area_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.parent.init_area_builder(ls, lc, force_class)
    }
}

/// The edge builder (`TopOpeBRepBuild_EdgeBuilder`): the edges as the areas of the paves;
/// `let eb = TopOpeBRepBuildEdgeBuilder::with_paves(&mut ps, &mut pc, false)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildEdgeBuilder {
    parent: TopOpeBRepBuildArea1dBuilder,
}

impl Deref for TopOpeBRepBuildEdgeBuilder {
    type Target = TopOpeBRepBuildArea1dBuilder;

    fn deref(&self) -> &TopOpeBRepBuildArea1dBuilder {
        &self.parent
    }
}

impl DerefMut for TopOpeBRepBuildEdgeBuilder {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildArea1dBuilder {
        &mut self.parent
    }
}

impl TopOpeBRepBuildEdgeBuilder {
    /// `TopOpeBRepBuild_EdgeBuilder()`; `TopOpeBRepBuildEdgeBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_EdgeBuilder(LS, LC, ForceClass)`;
    /// `TopOpeBRepBuildEdgeBuilder::with_paves(&mut ps, &mut pc, false)?`.
    pub fn with_paves(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut eb = Self::new();
        eb.init_edge_builder(ls, lc, force_class)?;
        Ok(eb)
    }

    /// `InitEdgeBuilder(LS, LC, ForceClass)`; `eb.init_edge_builder(&mut ps, &mut pc, false)?`.
    pub fn init_edge_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.parent.init_area_builder(ls, lc, force_class)
    }

    /// `InitEdge()`; `eb.init_edge()`.
    pub fn init_edge(&mut self) {
        self.init_area();
    }

    /// `MoreEdge()`; `eb.more_edge()`.
    pub fn more_edge(&self) -> bool {
        self.more_area()
    }

    /// `NextEdge()`; `eb.next_edge()`.
    pub fn next_edge(&mut self) {
        self.next_area();
    }

    /// `InitVertex()`; `eb.init_vertex()`.
    pub fn init_vertex(&mut self) {
        self.init_loop();
    }

    /// `MoreVertex()`; `eb.more_vertex()`.
    pub fn more_vertex(&self) -> bool {
        self.more_loop()
    }

    /// `NextVertex()`; `eb.next_vertex()`.
    pub fn next_vertex(&mut self) {
        self.next_loop();
    }

    /// `Vertex()`: the vertex of the current pave; `eb.vertex()?`.
    pub fn vertex(&self) -> Result<Shape, Failure> {
        Ok(pave(self.loop_()?)?.get().unwrap().vertex())
    }

    /// `Parameter()`: the parameter of the current pave; `eb.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        Ok(pave(self.loop_()?)?.get().unwrap().parameter())
    }
}
