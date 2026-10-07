// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_AreaBuilder,
// TopOpeBRepBuild_Area2dBuilder and TopOpeBRepBuild_FaceAreaBuilder (.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The area builders: the loops of a loop set grouped into areas (an outer loop with
//! the loops inside it) by a loop classifier.
use super::loops::{
    TopOpeBRepBuildLoop, TopOpeBRepBuildLoopClassifier, TopOpeBRepBuildLoopEnum,
    TopOpeBRepBuildLoopSet,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use std::ops::{Deref, DerefMut};

/// The loops of an area.
pub type LoopList = Vec<Handle<dyn TopOpeBRepBuildLoop>>;

/// Standard_NullObject of a null loop.
fn null_loop() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepBuild_AreaBuilder: null loop; expected a loop",
        "TopOpeBRepBuild_AreaBuilder",
    )
}

/// The areas of a loop set (`TopOpeBRepBuild_AreaBuilder`);
/// `let ab = TopOpeBRepBuildAreaBuilder::with_loops(&mut ls, &mut lc, false)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildAreaBuilder {
    /// `myArea` (protected).
    pub(crate) my_area: Vec<LoopList>,
    my_area_iterator: usize,
    my_loop_iterator: Option<(usize, usize)>,
    /// `myUNKNOWNRaise` (protected): false, so an UNKNOWN state is taken as IN.
    pub(crate) my_unknown_raise: bool,
}

/// The step of the area algorithm the 2d builders add (Area2dBuilder): the boundary
/// loops become an area when there is none.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flavor {
    Base,
    TwoD,
}

impl TopOpeBRepBuildAreaBuilder {
    /// `TopOpeBRepBuild_AreaBuilder()`; `TopOpeBRepBuildAreaBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_AreaBuilder(LS, LC, ForceClass)`: `InitAreaBuilder`;
    /// `TopOpeBRepBuildAreaBuilder::with_loops(&mut ls, &mut lc, false)?`.
    pub fn with_loops(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut ab = Self::new();
        ab.init_area_builder(ls, lc, force_class)?;
        Ok(ab)
    }

    /// `CompareLoopWithListOfLoop(LC, L, LOL, le)` (protected): OUT for an empty list,
    /// else the state of `l` against the tested loops of the list (the shape loops for
    /// BOUNDARY, the block loops for BLOCK), stopping at the first OUT; UNKNOWN when
    /// none is tested.
    pub fn compare_loop_with_list_of_loop(
        &self,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        l: &Handle<dyn TopOpeBRepBuildLoop>,
        lol: &[Handle<dyn TopOpeBRepBuildLoop>],
        what: TopOpeBRepBuildLoopEnum,
    ) -> Result<State, Failure> {
        if lol.is_empty() {
            return Ok(State::Out);
        }
        let mut state = State::Unknown;
        for cur_l in lol {
            let is_shape = cur_l.get().ok_or_else(null_loop)?.is_shape();
            let totest = match what {
                TopOpeBRepBuildLoopEnum::AnyLoop => true,
                TopOpeBRepBuildLoopEnum::Boundary => is_shape,
                TopOpeBRepBuildLoopEnum::Block => !is_shape,
            };
            if totest {
                state = lc.compare(l, cur_l)?;
                if state == State::Out {
                    break;
                }
            }
        }
        Ok(state)
    }

    /// `Atomize(state, newstate)` (protected): `newstate`, or Standard_DomainError for
    /// an UNKNOWN state when UNKNOWN raises.
    pub fn atomize(&self, state: &mut State, newstate: State) -> Result<(), Failure> {
        if self.my_unknown_raise {
            if *state == State::Unknown {
                return Err(Failure::with_type(
                    "Standard_DomainError",
                    "AreaBuilder : Position Unknown",
                    "TopOpeBRepBuild_AreaBuilder",
                ));
            }
            return Ok(());
        }
        *state = newstate;
        Ok(())
    }

    /// The state of a comparison, UNKNOWN atomized to IN.
    fn known(&self, mut state: State) -> Result<State, Failure> {
        if state == State::Unknown {
            self.atomize(&mut state, State::In)?;
        }
        Ok(state)
    }

    /// The first non-empty area containing the loop (compared with the loops of kind
    /// `what`).
    fn area_containing(
        &self,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        l: &Handle<dyn TopOpeBRepBuildLoop>,
        what: TopOpeBRepBuildLoopEnum,
    ) -> Result<Option<usize>, Failure> {
        for (k, a_area) in self.my_area.iter().enumerate() {
            if a_area.is_empty() {
                continue;
            }
            let state = self.known(self.compare_loop_with_list_of_loop(lc, l, a_area, what)?)?;
            if state == State::In {
                return Ok(Some(k));
            }
        }
        Ok(None)
    }

    /// A block loop inside the area `k`: the loops of the area outside it moved out (to
    /// the boundary loops when they are all shapes, else to a new area), then the loop
    /// added to the area.
    fn insert_block_in_area(
        &mut self,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        l: &Handle<dyn TopOpeBRepBuildLoop>,
        k: usize,
        boundaryloops: &mut LoopList,
    ) -> Result<(), Failure> {
        let (mut all_shape, mut removed_loops) = (true, LoopList::new());
        let mut i = 0;
        while i < self.my_area[k].len() {
            let cur_l = self.my_area[k][i].clone();
            let state = self.known(lc.compare(&cur_l, l)?)?;
            if state == State::Out {
                all_shape = all_shape && cur_l.get().ok_or_else(null_loop)?.is_shape();
                removed_loops.push(cur_l);
                self.my_area[k].remove(i);
            } else {
                i += 1;
            }
        }
        self.my_area[k].push(l.clone());
        if removed_loops.is_empty() {
            return Ok(());
        }
        if all_shape {
            boundaryloops.extend(removed_loops);
        } else {
            self.my_area.push(removed_loops);
        }
        Ok(())
    }

    /// A block loop outside every area: a new area with the boundary loops that are
    /// inside it and contain it.
    fn new_block_area(
        &mut self,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        l: &Handle<dyn TopOpeBRepBuildLoop>,
        boundaryloops: &mut LoopList,
    ) -> Result<(), Failure> {
        let mut new_area0: LoopList = vec![l.clone()];
        let mut i = 0;
        while i < boundaryloops.len() {
            let lb = boundaryloops[i].clone();
            let ashapeinside = self.known(lc.compare(&lb, l)?)? == State::In;
            let ablockinside = ashapeinside && self.known(lc.compare(l, &lb)?)? == State::In;
            if ashapeinside && ablockinside {
                new_area0.push(lb);
                boundaryloops.remove(i);
            } else {
                i += 1;
            }
        }
        self.my_area.push(new_area0);
        Ok(())
    }

    /// The areas of the loops of the set, for the base or the 2d flavor.
    fn build_areas(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
        flavor: Flavor,
    ) -> Result<(), Failure> {
        let mut boundaryloops = LoopList::new();
        self.my_area.clear();
        ls.init_loop()?;
        while ls.more_loop() {
            let l = ls.loop_()?;
            let boundary_l = l.get().ok_or_else(null_loop)?.is_shape();
            if boundary_l && !force_class {
                match self.area_containing(lc, &l, TopOpeBRepBuildLoopEnum::Block)? {
                    Some(k) => self.my_area[k].push(l.clone()),
                    None => boundaryloops.push(l.clone()),
                }
            } else {
                match self.area_containing(lc, &l, TopOpeBRepBuildLoopEnum::AnyLoop)? {
                    Some(k) => self.insert_block_in_area(lc, &l, k, &mut boundaryloops)?,
                    None => self.new_block_area(lc, &l, &mut boundaryloops)?,
                }
            }
            ls.next_loop()?;
        }
        if flavor == Flavor::TwoD && !boundaryloops.is_empty() && self.my_area.is_empty() {
            self.my_area.push(boundaryloops);
        }
        self.init_area();
        Ok(())
    }

    /// `InitAreaBuilder(LS, LC, ForceClass)` (virtual natively; ForceClass defaulted to
    /// false): each shape loop (unless ForceClass) added to the first area whose block
    /// loops contain it, else kept as a boundary loop; each block loop inside an area
    /// moves out the loops of the area outside it, else starts an area with the
    /// boundary loops around it; `ab.init_area_builder(&mut ls, &mut lc, false)?`.
    pub fn init_area_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.build_areas(ls, lc, force_class, Flavor::Base)
    }

    /// `InitArea()`: at the first area; the area count; `ab.init_area()`.
    pub fn init_area(&mut self) -> i32 {
        self.my_area_iterator = 0;
        self.init_loop();
        self.my_area.len() as i32
    }

    /// `MoreArea()`; `ab.more_area()`.
    pub fn more_area(&self) -> bool {
        self.my_area_iterator < self.my_area.len()
    }

    /// `NextArea()`; `ab.next_area()`.
    pub fn next_area(&mut self) {
        self.my_area_iterator += 1;
        self.init_loop();
    }

    /// `InitLoop()`: at the first loop of the current area; its loop count;
    /// `ab.init_loop()`.
    pub fn init_loop(&mut self) -> i32 {
        if self.more_area() {
            self.my_loop_iterator = Some((self.my_area_iterator, 0));
            return self.my_area[self.my_area_iterator].len() as i32;
        }
        self.my_loop_iterator = None;
        0
    }

    /// `MoreLoop()`; `ab.more_loop()`.
    pub fn more_loop(&self) -> bool {
        self.my_loop_iterator
            .is_some_and(|(a, l)| l < self.my_area[a].len())
    }

    /// `NextLoop()`; `ab.next_loop()`.
    pub fn next_loop(&mut self) {
        if let Some((_, l)) = self.my_loop_iterator.as_mut() {
            *l += 1;
        }
    }

    /// `Loop()`: the current loop; Standard_NoSuchObject past the end; `ab.loop_()?`.
    pub fn loop_(&self) -> Result<&Handle<dyn TopOpeBRepBuildLoop>, Failure> {
        self.my_loop_iterator
            .and_then(|(a, l)| self.my_area.get(a).and_then(|area| area.get(l)))
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "TopOpeBRepBuild_AreaBuilder::Loop: no current loop; expected MoreLoop()",
                    "TopOpeBRepBuild_AreaBuilder",
                )
            })
    }

    /// `ADD_Loop_TO_LISTOFLoop(L, LOL, s)`: appended; `ab.add_loop_to_list_of_loop(&l, &mut lol)`.
    pub fn add_loop_to_list_of_loop(
        &self,
        l: &Handle<dyn TopOpeBRepBuildLoop>,
        lol: &mut LoopList,
    ) {
        lol.push(l.clone());
    }

    /// `REM_Loop_FROM_LISTOFLoop(ITLOL, LOL, s)`: the loop at the position removed;
    /// `ab.rem_loop_from_list_of_loop(0, &mut lol)`.
    pub fn rem_loop_from_list_of_loop(&self, itlol: usize, lol: &mut LoopList) {
        if itlol < lol.len() {
            lol.remove(itlol);
        }
    }

    /// `ADD_LISTOFLoop_TO_LISTOFLoop(LOL1, LOL2, s, s1, s2)`: the loops of `lol1` moved
    /// to the end of `lol2`; `ab.add_list_of_loop_to_list_of_loop(&mut a, &mut b)`.
    pub fn add_list_of_loop_to_list_of_loop(&self, lol1: &mut LoopList, lol2: &mut LoopList) {
        lol2.append(lol1);
    }
}

/// The 2d area builder (`TopOpeBRepBuild_Area2dBuilder`): the boundary loops become an
/// area when no area is built; `TopOpeBRepBuildArea2dBuilder::with_loops(&mut ls, &mut lc, false)?`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildArea2dBuilder {
    base: TopOpeBRepBuildAreaBuilder,
}

impl Deref for TopOpeBRepBuildArea2dBuilder {
    type Target = TopOpeBRepBuildAreaBuilder;
    fn deref(&self) -> &TopOpeBRepBuildAreaBuilder {
        &self.base
    }
}

impl DerefMut for TopOpeBRepBuildArea2dBuilder {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildAreaBuilder {
        &mut self.base
    }
}

impl TopOpeBRepBuildArea2dBuilder {
    /// `TopOpeBRepBuild_Area2dBuilder()`; `TopOpeBRepBuildArea2dBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_Area2dBuilder(LS, LC, ForceClass)`;
    /// `TopOpeBRepBuildArea2dBuilder::with_loops(&mut ls, &mut lc, false)?`.
    pub fn with_loops(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut ab = Self::new();
        ab.init_area_builder(ls, lc, force_class)?;
        Ok(ab)
    }

    /// `InitAreaBuilder(LS, LC, ForceClass)`: the areas of AreaBuilder, then the
    /// boundary loops as an area when there is none;
    /// `ab.init_area_builder(&mut ls, &mut lc, false)?`.
    pub fn init_area_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.base.build_areas(ls, lc, force_class, Flavor::TwoD)
    }
}

/// The area builder of faces (`TopOpeBRepBuild_FaceAreaBuilder`);
/// `TopOpeBRepBuildFaceAreaBuilder::with_loops(&mut ls, &mut lc, true)?`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildFaceAreaBuilder {
    base: TopOpeBRepBuildArea2dBuilder,
}

impl Deref for TopOpeBRepBuildFaceAreaBuilder {
    type Target = TopOpeBRepBuildArea2dBuilder;
    fn deref(&self) -> &TopOpeBRepBuildArea2dBuilder {
        &self.base
    }
}

impl DerefMut for TopOpeBRepBuildFaceAreaBuilder {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildArea2dBuilder {
        &mut self.base
    }
}

impl TopOpeBRepBuildFaceAreaBuilder {
    /// `TopOpeBRepBuild_FaceAreaBuilder()`; `TopOpeBRepBuildFaceAreaBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_FaceAreaBuilder(LS, LC, ForceClass)`;
    /// `TopOpeBRepBuildFaceAreaBuilder::with_loops(&mut ls, &mut lc, true)?`.
    pub fn with_loops(
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut ab = Self::new();
        ab.init_face_area_builder(ls, lc, force_class)?;
        Ok(ab)
    }

    /// `InitFaceAreaBuilder(LS, LC, ForceClass)`: the 2d areas;
    /// `ab.init_face_area_builder(&mut ls, &mut lc, true)?`.
    pub fn init_face_area_builder(
        &mut self,
        ls: &mut dyn TopOpeBRepBuildLoopSet,
        lc: &mut dyn TopOpeBRepBuildLoopClassifier,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.base.init_area_builder(ls, lc, force_class)
    }
}
