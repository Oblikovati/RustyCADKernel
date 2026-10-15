//! The detection and the merge of the disjoint particular case (TopOpeBRepBuild_KPart.cxx).
use super::TopOpeBRepBuildBuilder;
use super::kpart::kpresu::*;
use super::kpart_disj::KPartOperation;
use super::kpart_disj::SolidMap;
use super::kpart_disj::build_new_solid;
use super::kpart_disj::disj_perform_common;
use super::kpart_disj::disj_perform_cut;
use super::kpart_disj::disj_perform_fuse;
use super::kpart_disj::local_kp_isdisjanalyse;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Solid};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3d;

impl TopOpeBRepBuildBuilder {
    /// `KPisdisjanalyse(Stsol1, Stsol2, ires, icla1, icla2)`: the result and shell
    /// selections of the operation for two disjoint solids;
    /// `let (ires, icla1, icla2) = b.kp_isdisjanalyse(State::Out, State::Out);`.
    pub fn kp_isdisjanalyse(&self, stsol1: State, stsol2: State) -> (i32, i32, i32) {
        let op = if self.opefus() {
            KPartOperation::Fuse
        } else if self.opec12() {
            KPartOperation::Cut12
        } else if self.opec21() {
            KPartOperation::Cut21
        } else if self.opecom() {
            KPartOperation::Common
        } else {
            return (RESUNDEF, SHEUNDEF, SHEUNDEF);
        };
        local_kp_isdisjanalyse(stsol1, stsol2, op)
    }

    /// Whether both shapes are solids only (no shell, face, edge or vertex out of a solid).
    fn only_solids(&self) -> Result<bool, Failure> {
        for t in [
            ShapeEnum::Shell,
            ShapeEnum::Face,
            ShapeEnum::Wire,
            ShapeEnum::Edge,
            ShapeEnum::Vertex,
        ] {
            let n1 = Explorer::new(&self.my_shape1, t, ShapeEnum::Solid)?.more();
            let n2 = Explorer::new(&self.my_shape2, t, ShapeEnum::Solid)?.more();
            if n1 || n2 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The several solids case of the disjoint merge: the result solids of the
    /// operation appended (nothing when it fails).
    fn merge_disj_solids(&mut self, m1: &SolidMap, m2: &SolidMap) -> Result<(), Failure> {
        let mut result = SolidMap::default();
        let ok = if self.opefus() {
            disj_perform_fuse(m1, m2, &mut result)?
        } else if self.opec12() {
            disj_perform_cut(m1, m2, &mut result)?
        } else if self.opec21() {
            disj_perform_cut(m2, m1, &mut result)?
        } else if self.opecom() {
            disj_perform_common(m1, m2, &mut result)?
        } else {
            true
        };
        if !ok {
            return Ok(());
        }
        let (s1, st1) = (self.my_shape1.clone(), self.my_state1);
        for i in 1..=result.extent() {
            let r = result.find_key(i)?.clone();
            self.change_merged(&s1, st1).append(r);
        }
        Ok(())
    }

    /// The single solids case of the disjoint merge.
    fn merge_disj_solid(&mut self, sol1: &Solid, sol2: &Solid) -> Result<(), Failure> {
        self.change_merged(sol1.as_ref(), self.my_state1);
        let outsha1 = BRepClass3d::outer_shell(sol1)?.into_shape();
        self.change_merged(sol2.as_ref(), self.my_state2);
        let outsha2 = BRepClass3d::outer_shell(sol2)?.into_shape();
        let stsol1 = self.kp_clas_ss(&outsha1, sol2.as_ref())?;
        let stsol2 = self.kp_clas_ss(&outsha2, sol1.as_ref())?;
        let (ires, icla1, icla2) = self.kp_isdisjanalyse(stsol1, stsol2);
        if ires == RESUNDEF || icla1 == SHEUNDEF || icla2 == SHEUNDEF || ires == RESNULL {
            return Ok(());
        }
        let (s1, s2, st1) = (
            self.my_shape1.clone(),
            self.my_shape2.clone(),
            self.my_state1,
        );
        let l = self.change_merged(&s1, st1);
        match ires {
            RESSHAPE12 => {
                l.append(s1);
                l.append(s2);
            }
            RESSHAPE1 => {
                l.append(s1);
            }
            RESSHAPE2 => {
                l.append(s2);
            }
            RESNEWSHA1 | RESNEWSHA2 => {
                let states = (self.my_state1, self.my_state2);
                let newsol =
                    build_new_solid(sol1, sol2, (stsol1, stsol2), ires, (icla1, icla2), states)?;
                self.change_merged(&s1, st1).append(newsol.into_shape());
            }
            _ => {}
        }
        Ok(())
    }

    /// `MergeKPartisdisj()`: the disjoint shapes combined by the operation: the solids
    /// classified by their outer shells, or the shapes themselves when not solids only;
    /// `b.merge_k_part_isdisj()?`.
    pub fn merge_k_part_isdisj(&mut self) -> Result<(), Failure> {
        if self.my_is_k_part != 2 {
            return Ok(());
        }
        let (s1, s2, st1, st2) = (
            self.my_shape1.clone(),
            self.my_shape2.clone(),
            self.my_state1,
            self.my_state2,
        );
        self.change_merged(&s1, st1);
        self.change_merged(&s2, st2);
        if !self.only_solids()? {
            let (c12, c21, com, fus) = (self.opec12(), self.opec21(), self.opecom(), self.opefus());
            let l = self.change_merged(&s1, st1);
            if c12 {
                l.append(s1.clone());
            } else if c21 {
                l.append(s2.clone());
            } else if com {
                l.clear();
            } else if fus {
                l.append(s1.clone());
                l.append(s2.clone());
            }
            return Ok(());
        }
        let (mut m1, mut m2) = (SolidMap::default(), SolidMap::default());
        topexp::map_shapes_of_type(&s1, ShapeEnum::Solid, &mut m1)?;
        topexp::map_shapes_of_type(&s2, ShapeEnum::Solid, &mut m2)?;
        if m1.extent() > 1 || m2.extent() > 1 {
            return self.merge_disj_solids(&m1, &m2);
        }
        let sol1 = topods::solid(
            &Iterator::next(&mut Explorer::new(&s1, ShapeEnum::Solid, ShapeEnum::Shape)?)
                .unwrap_or_default(),
        )?;
        let sol2 = topods::solid(
            &Iterator::next(&mut Explorer::new(&s2, ShapeEnum::Solid, ShapeEnum::Shape)?)
                .unwrap_or_default(),
        )?;
        self.merge_disj_solid(&sol1, &sol2)
    }
}
