//! HLRBRep_Hider.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{Data, EdgeIList, EdgeInterferenceTool};
use crate::hlr_algo::Interference;
use crate::hlr_topo_brep::MapOfShapeTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, State};

/// HLRBRep_Hider: hides the edges of a data structure by one face: for each edge
/// the face may hide, the interferences with the face boundary are sorted into the
/// hidden (IN) and on-face (ON) lists, merged and resolved, and the parts of the
/// edge they bound are hidden in its status; an edge without interference is
/// classified as a whole. A failure on one edge leaves it as it is and the next edge
/// follows (the native try/catch).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_brep::{Data, Hider};
/// let hider = Hider::new(&Handle::new(Data::new(0, 0, 0)?));
/// hider.own_hiding(1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Hider {
    ds: Handle<Data>,
}

// The interference lists of one edge (`ILHidden`, `ILOn`).
pub(super) struct EdgeLists {
    pub(super) hidden: Vec<Interference>,
    pub(super) on: Vec<Interference>,
}

impl Hider {
    /// HLRBRep_Hider(DS); `Hider::new(&ds)`.
    pub fn new(ds: &Handle<Data>) -> Self {
        Self { ds: ds.clone() }
    }

    /// OwnHiding(I): nothing, as natively; `hider.own_hiding(1)`.
    pub fn own_hiding(&self, _i: i32) {}

    pub(super) fn data(&self) -> Result<&Data, Failure> {
        self.ds.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_Hider: no data structure; expected a HLRBRep_Data handle",
            )
        })
    }

    /// Hide(FI, MST): hides the edges of the data structure by the face `fi`;
    /// `hider.hide(1, &mut tools)?`.
    pub fn hide(&self, fi: i32, mst: &mut MapOfShapeTool) -> Result<(), Failure> {
        let ds = self.data()?;
        ds.init_edge(fi, mst)?;
        if !ds.more_edge()? {
            return Ok(()); // there is nothing to do
        }
        if ds.is_bad_face()? {
            return Ok(());
        }
        let mut eit = EdgeInterferenceTool::new(&self.ds);
        while ds.more_edge()? {
            let e = ds.edge()?;
            // The native code catches a Standard_Failure of one edge and goes on.
            let _ = self.hide_edge(ds, &mut eit, e);
            ds.next_edge(true)?;
        }
        Ok(())
    }

    fn hide_edge(&self, ds: &Data, eit: &mut EdgeInterferenceTool, e: i32) -> Result<(), Failure> {
        eit.load_edge()?;
        let (mut lists, has_out) = collect_interferences(ds, eit)?;
        merge_segments(&mut lists.hidden);
        if !lists.on.is_empty() {
            EdgeIList::process_complex_in(&mut lists.on, eit)?;
            resolve_on(&mut lists, eit);
        }
        if lists.hidden.is_empty() && lists.on.is_empty() && !has_out {
            let st = ds.compare(e)?; // Classification
            if st == State::In || st == State::On {
                hide_all(ds, e)?;
            }
            return Ok(());
        }
        self.hide_parts(ds, eit, e, &mut lists)
    }
}

pub(super) fn hide_all(ds: &Data, e: i32) -> Result<(), Failure> {
    let mut ed = ds.e_data_array();
    ed.change_value(e)
        .map_err(|x| x.into_failure("HLRBRep_Hider::Hide"))?
        .status_mut()
        .hide_all();
    Ok(())
}

// The IN and ON interferences of the current edge with the face, sorted, and
// whether a rejected one was above a simple hiding face.
fn collect_interferences(
    ds: &Data,
    eit: &EdgeInterferenceTool,
) -> Result<(EdgeLists, bool), Failure> {
    let mut lists = EdgeLists {
        hidden: Vec::new(),
        on: Vec::new(),
    };
    let mut has_out = false;
    ds.init_interference()?;
    while ds.more_interference() {
        if ds.rejected_interference()? {
            if ds.above_interference() && ds.simple_hiding_face() {
                has_out = true;
            }
        } else {
            let int = ds.interference();
            match int.intersection().state() {
                State::In => EdgeIList::add_interference_in(&mut lists.hidden, &int, eit),
                State::On => EdgeIList::add_interference_in(&mut lists.on, &int, eit),
                State::Out | State::Unknown => {}
            }
        }
        ds.next_interference()?;
    }
    Ok((lists, has_out))
}

// The vertex index of two segment ends that become one: the common non zero index,
// 0 when both are 0, -1 when they differ.
fn merged_index(i1: i32, i2: i32) -> i32 {
    if i1 == 0 && i2 == 0 {
        return 0;
    }
    match (i1, i2) {
        (0, b) => b,
        (a, 0) => a,
        (a, b) if a == b => a,
        _ => -1,
    }
}

// Whether the opposite ends `a` and `b` of one segment have the same orientations
// and 3D states (none ON), so that they merge into one interference.
fn mergeable(a: &Interference, b: &Interference) -> bool {
    let (stbef1, staft1) = a.boundary().state_3d();
    let (stbef2, staft2) = b.boundary().state_3d();
    a.orientation() == b.orientation()
        && a.transition() == b.transition()
        && stbef1 == stbef2
        && staft1 == staft2
        && stbef1 != State::On
        && staft1 != State::On
}

// The two ends of a hidden segment of the face boundary (segment indices n and
// -n) merge into the first one, at the middle parameter; repeated until none does.
fn merge_segments(il: &mut Vec<Interference>) {
    'restart: loop {
        for i1 in 0..il.len() {
            let numseg1 = il[i1].intersection().seg_index();
            if numseg1 == 0 {
                continue;
            }
            for i2 in 0..il.len() {
                if numseg1 + il[i2].intersection().seg_index() != 0 || !mergeable(&il[i1], &il[i2])
                {
                    continue;
                }
                let nind =
                    merged_index(il[i1].intersection().index(), il[i2].intersection().index());
                if nind == -1 {
                    continue;
                }
                let p2 = il[i2].intersection().parameter();
                let int1 = &mut il[i1];
                let p1 = int1.intersection().parameter();
                let inter = int1.change_intersection();
                inter.set_seg_index(nind);
                inter.set_parameter((p1 + p2) * 0.5);
                int1.set_boundary_transition(Orientation::External);
                il.remove(i2);
                continue 'restart;
            }
        }
        return;
    }
}

// What becomes of one ON interference: kept (moved on), removed, or moved to the
// hidden list, possibly with a copy of other transition added to the hidden list.
enum OnFate {
    Keep,
    Remove,
    ToHidden,
    KeepAndHide(Orientation),
}

fn on_fate(int: &mut Interference) -> OnFate {
    let (stbef, staft) = int.boundary().state_3d();
    match int.transition() {
        Orientation::Forward => match staft {
            State::Out => OnFate::Remove,
            State::In => OnFate::ToHidden,
            State::Unknown | State::On => OnFate::Keep,
        },
        Orientation::Reversed => match stbef {
            State::Out => OnFate::Remove,
            State::In => OnFate::ToHidden,
            State::Unknown | State::On => OnFate::Keep,
        },
        Orientation::External => OnFate::Remove,
        Orientation::Internal => internal_on_fate(int, stbef, staft),
    }
}

fn internal_on_fate(int: &mut Interference, stbef: State, staft: State) -> OnFate {
    match (stbef, staft) {
        (State::In, State::In) => OnFate::ToHidden,
        (State::In, State::On) => {
            int.set_transition(Orientation::Forward); // FORWARD in ILOn,
            OnFate::KeepAndHide(Orientation::Reversed) // REVERSED in ILHidden
        }
        (State::In, State::Out) | (State::On, State::Out) => {
            int.set_transition(Orientation::Reversed);
            if stbef == State::In {
                OnFate::ToHidden
            } else {
                OnFate::Keep
            }
        }
        (State::On, State::In) => {
            int.set_transition(Orientation::Reversed); // REVERSED in ILOn,
            OnFate::KeepAndHide(Orientation::Forward) // FORWARD in ILHidden
        }
        (State::Out, State::In) => {
            int.set_transition(Orientation::Forward);
            OnFate::ToHidden
        }
        (State::Out, State::On) => {
            int.set_transition(Orientation::Forward);
            OnFate::Keep
        }
        (State::Out, State::Out) => OnFate::Remove,
        // Natively an UNKNOWN state before the point does not move the iterator (an
        // endless loop); the states of an ON interference are never UNKNOWN.
        _ => OnFate::Keep,
    }
}

// The ON interferences on the face: each one stays ON, is dropped (OUT) or moves
// to the hidden list (IN), by its transition and 3D states.
fn resolve_on(lists: &mut EdgeLists, eit: &EdgeInterferenceTool) {
    let mut i = 0;
    while i < lists.on.len() {
        match on_fate(&mut lists.on[i]) {
            OnFate::Keep => i += 1,
            OnFate::Remove => {
                lists.on.remove(i);
            }
            OnFate::ToHidden => {
                let int = lists.on.remove(i);
                EdgeIList::add_interference_in(&mut lists.hidden, &int, eit);
            }
            OnFate::KeepAndHide(transition) => {
                let int = &lists.on[i];
                let copy = Interference::with_values(
                    *int.intersection(),
                    *int.boundary(),
                    int.orientation(),
                    transition,
                    int.boundary_transition(),
                );
                EdgeIList::add_interference_in(&mut lists.hidden, &copy, eit);
                i += 1;
            }
        }
    }
}

mod parts;
