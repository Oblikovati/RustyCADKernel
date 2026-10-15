//! `SplitEdge` of `TopOpeBRepBuild_Builder1` (Builder1_1.cxx): an edge split by the
//! paves of its point topology (coincident paves merged), each part with the state
//! given by the orientations of its bounding paves.
use super::{UNK_STATE_VER, USED_VERTEX_MAP};
use crate::top_ope_brep_build::tools::{DataMapOfShapeState, PAR_T};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTool, TopOpeBRepBuildLoopSet, TopOpeBRepBuildPave,
    TopOpeBRepBuildPaveSet,
};
use crate::top_ope_brep_ds::TopOpeBRepDSKind;
use crate::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The orientations noted for a merged vertex: the pave taken first (1) or second (0),
/// then the orientation of the other pave (as the native integer list).
type VerOriMap = DataMap<Shape, List<i32>, ShapeMapHasher>;

/// The static `ClassifyEdgeToSolidByOnePoint(E, SC)`: the state in the reference of the
/// sample point of the 3d curve (of the first vertex without curve; UNKNOWN without).
pub(crate) fn classify_edge_to_solid_by_one_point(
    e: &Edge,
    sc: &mut TopOpeBRepToolShapeClassifier,
) -> Result<State, Failure> {
    let (c3d, f2, l2) = BRepTool::curve(e)?;
    let p3d = match c3d.get() {
        Some(c) => c.value(f2 * PAR_T + (1. - PAR_T) * l2)?,
        None => {
            let fv = topexp::first_vertex(e, false);
            if fv.is_null() {
                return Ok(State::Unknown);
            }
            BRepTool::pnt(&fv)?
        }
    };
    sc.state_p3d_reference(p3d)?;
    Ok(sc.state())
}

/// The orientation of a native integer.
fn ori_of(i: i32) -> Orientation {
    Orientation::ALL
        .get(usize::try_from(i).unwrap_or(0))
        .copied()
        .unwrap_or(Orientation::Forward)
}

/// The state of a part by the orientations of its bounding paves: OUT from a FORWARD
/// or INTERNAL pave to a REVERSED or INTERNAL one, IN otherwise.
fn part_state(or1: Orientation, or2: Orientation) -> State {
    let out = matches!(or1, Orientation::Forward | Orientation::Internal)
        && matches!(or2, Orientation::Reversed | Orientation::Internal);
    if out { State::Out } else { State::In }
}

impl TopOpeBRepBuildBuilder {
    /// The pave kept of two coincident paves: the one with a same domain vertex, else
    /// a used vertex, else the first when on a face interference.
    fn take_first_pave(
        &self,
        v1: &Shape,
        v2: &Shape,
        pv1: &TopOpeBRepBuildPave,
    ) -> Result<bool, Failure> {
        let (sd1, sd2) = (
            self.hds()?.has_same_domain(v1, true),
            self.hds()?.has_same_domain(v2, true),
        );
        let (u1, u2) = USED_VERTEX_MAP.with_borrow(|m| (m.contains(v1), m.contains(v2)));
        Ok(if sd1 {
            true
        } else if sd2 {
            false
        } else if u1 {
            true
        } else if u2 {
            false
        } else {
            pv1.interference_type() == TopOpeBRepDSKind::Face
        })
    }

    /// Two coincident paves merged: the kept one listed; for different vertex
    /// orientations, the kept side and the other orientation noted for the vertex,
    /// whose (reference) vertex gets an unknown state.
    fn merge_paves(
        &self,
        pv1: &Handle<TopOpeBRepBuildPave>,
        pv2: &Handle<TopOpeBRepBuildPave>,
        list: &mut Vec<Handle<TopOpeBRepBuildPave>>,
        ori_map: &mut VerOriMap,
    ) -> Result<(), Failure> {
        let (p1, p2) = (pv1.get().expect("pave"), pv2.get().expect("pave"));
        let (v1, v2) = (p1.vertex(), p2.vertex());
        let take_first = self.take_first_pave(&v1, &v2, p1)?;
        let (mut ver, has_sdv, ori_opp) = if take_first {
            list.push(pv1.clone());
            (
                v1.clone(),
                self.hds()?.has_same_domain(&v1, true),
                v2.orientation(),
            )
        } else {
            list.push(pv2.clone());
            (
                v2.clone(),
                self.hds()?.has_same_domain(&v2, true),
                v1.orientation(),
            )
        };
        if v1.orientation() != v2.orientation() {
            if !ori_map.is_bound(&ver) {
                ori_map.bind(ver.clone(), List::default());
            }
            let l = ori_map.change_find(&ver)?;
            l.append(i32::from(take_first));
            l.append(ori_opp as i32);
            if has_sdv {
                ver = self
                    .hds()?
                    .shape(self.hds()?.same_domain_reference(&ver), true);
            }
            UNK_STATE_VER.with_borrow_mut(|m| m.add(ver));
        }
        Ok(())
    }

    /// The paves of the edge with the coincident consecutive paves merged.
    fn merged_paves(
        &self,
        an_edge: &Shape,
        pvs: &mut TopOpeBRepBuildPaveSet,
        ori_map: &mut VerOriMap,
    ) -> Result<Vec<Handle<TopOpeBRepBuildPave>>, Failure> {
        let e = topods::edge(an_edge)?;
        let ca = BRepAdaptorCurve::with_edge(&e)?;
        let tol_edge = BRepTool::edge_tolerance(&e)?;
        let mut list = Vec::new();
        pvs.init_loop()?;
        while pvs.more_loop() {
            let pv1: Handle<TopOpeBRepBuildPave> = pvs.loop_()?.cast();
            pvs.next_loop()?;
            if !pvs.more_loop() {
                list.push(pv1);
                break;
            }
            let pv2: Handle<TopOpeBRepBuildPave> = pvs.loop_()?.cast();
            let (p1, p2) = (pv1.get().expect("pave"), pv2.get().expect("pave"));
            let (v1, v2) = (topods::vertex(&p1.vertex())?, topods::vertex(&p2.vertex())?);
            let tol_max = tol_edge
                .max(BRepTool::vertex_tolerance(&v1)?.max(BRepTool::vertex_tolerance(&v2)?));
            let delta = (p1.parameter() - p2.parameter()).abs();
            if delta < ca.resolution(tol_max)? {
                let dist = BRepTool::pnt(&v1)?.distance(&BRepTool::pnt(&v2)?);
                if dist < tol_max || delta < precision::p_confusion() {
                    self.merge_paves(&pv1, &pv2, &mut list, ori_map)?;
                    pvs.next_loop()?;
                    continue;
                }
            }
            list.push(pv1);
        }
        Ok(list)
    }

    /// The orientation of a pave vertex, replaced by the noted one of a merged vertex
    /// (when the side matches; the note consumed for a first vertex).
    fn noted_orientation(
        ori_map: &mut VerOriMap,
        v: &Shape,
        first: bool,
    ) -> Result<Option<Orientation>, Failure> {
        if !ori_map.is_bound(v) {
            return Ok(None);
        }
        let l = ori_map.change_find(v)?;
        let items: Vec<i32> = l.iter().copied().collect();
        if items.is_empty() {
            return Ok(None);
        }
        let side = items[0] != 0;
        let noted = (side == first)
            .then(|| items.get(1).copied().map(ori_of))
            .flatten();
        if first {
            l.remove_first()?;
            l.remove_first()?;
        }
        Ok(noted)
    }

    /// The (reference, for a same domain vertex) vertex of a pave, oriented.
    fn pave_vertex(&self, v: &Shape, ori: Orientation) -> Result<Shape, Failure> {
        let mut v = v.oriented(ori);
        if self.hds()?.has_same_domain(&v, true) {
            v = self
                .hds()?
                .shape(self.hds()?.same_domain_reference(&v), true)
                .oriented(ori);
        }
        Ok(v)
    }

    /// A part of the forward edge between two vertices at their parameters.
    fn edge_part(
        &self,
        edge_f: &Shape,
        (v1, par1): (&Shape, f64),
        (v2, par2): (&Shape, f64),
    ) -> Result<Shape, Failure> {
        let mut new_e = Shape::default();
        self.my_build_tool.copy_edge(edge_f, &mut new_e)?;
        self.my_build_tool.add_edge_vertex(&mut new_e, v1)?;
        self.my_build_tool.parameter(&new_e, v1, par1)?;
        self.my_build_tool.add_edge_vertex(&mut new_e, v2)?;
        self.my_build_tool.parameter(&new_e, v2, par2)?;
        Ok(new_e)
    }

    /// `SplitEdge(anEdge, aLNew, aDataMapOfShapeState)` (Builder1): the parts of the
    /// edge between its consecutive (merged) paves, with their states by the pave
    /// orientations; `b.split_edge_with_states(&e, &mut l, &mut m)?`.
    pub fn split_edge_with_states(
        &mut self,
        an_edge: &Shape,
        l_new: &mut List<Shape>,
        map_state: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        let edge_f = an_edge.oriented(Orientation::Forward);
        let mut pvs = TopOpeBRepBuildPaveSet::new(&edge_f)?;
        let g1 = TopOpeBRepBuildGTool::g_fus_same(ShapeEnum::Face, ShapeEnum::Face);
        self.my_edge_reference = topods::edge(&edge_f)?;
        self.g_fill_point_topology_pvs(&edge_f, &g1, &mut pvs)?;
        let mut ori_map = VerOriMap::default();
        let list = self.merged_paves(an_edge, &mut pvs, &mut ori_map)?;
        let mut i = 0;
        while i < list.len() {
            let p1 = list[i].get().expect("pave");
            let v1 = p1.vertex().oriented(Orientation::Forward);
            let or1 = Self::noted_orientation(&mut ori_map, &v1, true)?
                .unwrap_or(p1.vertex().orientation());
            i += 1;
            let Some(pv2) = list.get(i) else { break };
            let p2 = pv2.get().expect("pave");
            let v2 = p2.vertex().oriented(Orientation::Reversed);
            let or2 = Self::noted_orientation(&mut ori_map, &v2, false)?
                .unwrap_or(p2.vertex().orientation());
            if p1.parameter() > p2.parameter() {
                continue;
            }
            let v1 = self.pave_vertex(&v1, Orientation::Forward)?;
            let v2 = self.pave_vertex(&v2, Orientation::Reversed)?;
            let new_e = self
                .edge_part(&edge_f, (&v1, p1.parameter()), (&v2, p2.parameter()))?
                .oriented(an_edge.orientation());
            l_new.append(new_e.clone());
            map_state.bind(new_e, part_state(or1, or2));
        }
        Ok(())
    }
}
