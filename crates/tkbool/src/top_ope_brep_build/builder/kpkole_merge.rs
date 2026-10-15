//! The merge of the kole particular case (TopOpeBRepBuild_kpkole.cxx).
use super::TopOpeBRepBuildBuilder;
use super::kpart::kpresu::*;
use super::kpkole::KoleContext;
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// `MergeKPartiskole()`: the tangent solids merged by the analysis of their states:
    /// a shell, the tangent face, or a new shell of the faces made of the tangent faces
    /// and of the other faces of the shells, with the other shells kept or classified;
    /// `b.merge_k_part_iskole()?`.
    pub fn merge_k_part_iskole(&mut self) -> Result<(), Failure> {
        if self.my_is_k_part != 1 {
            return Ok(());
        }
        let (s1, s2, st1, st2) = (
            self.my_shape1.clone(),
            self.my_shape2.clone(),
            self.my_state1,
            self.my_state2,
        );
        self.g_map_shapes(&s1, &s2)?;
        self.change_merged(&s1, st1);
        self.change_merged(&s2, st2);
        let (mut lshsd1, mut lshsd2, mut lfhsd1, mut lfhsd2) = Default::default();
        self.kp_iskolesh(&s1, &mut lshsd1, &mut lfhsd1)?;
        self.kp_iskolesh(&s2, &mut lshsd2, &mut lfhsd2)?;
        let sol1: Shape = lshsd1.first().cloned().unwrap_or_default();
        let sol2: Shape = lshsd2.first().cloned().unwrap_or_default();
        self.change_merged(&sol1, st1);
        self.change_merged(&sol2, st2);
        if self.my_kp_map_f1f2.is_empty() {
            return Ok(());
        }
        let Some(kf) = self.kole_faces() else {
            return Ok(());
        };
        let (rank_ou, rank_in) = (self.g_shape_rank(&kf.f_ou), self.g_shape_rank(&kf.f_in));
        if !(rank_ou == 1 || rank_ou == 2) || !(rank_in == 1 || rank_in == 2) {
            return Ok(());
        }
        let solf_ou = if rank_ou == 1 {
            sol1.clone()
        } else {
            sol2.clone()
        };
        let solf_in = if rank_in == 1 {
            sol1.clone()
        } else {
            sol2.clone()
        };
        let stsolf_ou = self.kp_clas_ss_avoiding(&solf_ou, &kf.f_ou, &solf_in)?;
        let stsolf_in = self.kp_clas_ss_list(&solf_in, &kf.lfin, &solf_ou)?;
        let (stsol1, stfac1, fac1, stsol2, stfac2, fac2) = if rank_ou == 1 {
            (
                stsolf_ou,
                State::Out,
                kf.f_ou.clone(),
                stsolf_in,
                State::In,
                kf.f_in.clone(),
            )
        } else {
            (
                stsolf_in,
                State::In,
                kf.f_in.clone(),
                stsolf_ou,
                State::Out,
                kf.f_ou.clone(),
            )
        };
        let (ires, icla1, icla2) = self.kp_iskoleanalyse(stfac1, stfac2, stsol1, stsol2);
        if ires == RESUNDEF || icla1 == SHEUNDEF || icla2 == SHEUNDEF {
            return Ok(());
        }
        let she1 = Self::shell_of_face_in(&sol1, &fac1)?;
        let she2 = Self::shell_of_face_in(&sol2, &fac2)?;
        self.change_merged(&she1, st1);
        self.change_merged(&she2, st2);
        let ctx = KoleContext {
            sol: (sol1, sol2),
            she: (she1, she2),
            fac: (fac1, fac2),
            stsol: (stsol1, stsol2),
            rank_ou,
            kf,
        };
        let newshe = match ires {
            RESNULL => return Ok(()),
            RESSHAPE1 => ctx.she.0.clone(),
            RESSHAPE2 => ctx.she.1.clone(),
            RESFACE1 | RESFACE2 => return self.kole_face_result(ires, &ctx),
            RESNEWSHE => match self.kole_new_shell(&ctx)? {
                Some(s) => s,
                None => return Ok(()),
            },
            _ => Shape::default(),
        };
        self.kole_new_solid(&newshe, (icla1, icla2), &ctx)
    }

    /// The tangent face result: the OUT face (or the IN faces) merged.
    fn kole_face_result(&mut self, ires: i32, ctx: &KoleContext) -> Result<(), Failure> {
        let (s1, st1, st2) = (self.my_shape1.clone(), self.my_state1, self.my_state2);
        let (own_rank, other_fac, other_state) = if ires == RESFACE1 {
            (1, &ctx.fac.1, st2)
        } else {
            (2, &ctx.fac.0, st1)
        };
        if ctx.rank_ou == own_rank {
            self.change_merged(&s1, st1).append(ctx.kf.f_ou.clone());
            self.change_merged(other_fac, other_state)
                .append(ctx.kf.f_ou.clone());
        } else {
            let l = ctx.kf.lf_in_first.clone();
            Self::g_copy_list(&l, self.change_merged(&s1, st1));
            Self::g_copy_list(&l, self.change_merged(other_fac, other_state));
        }
        Ok(())
    }

    /// The faces of a shell not tangent and not added yet added (complemented IN).
    fn kole_add_shell_faces(
        &self,
        she: &Shape,
        stsol: State,
        addedfaces: &mut DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let mut fex = TopOpeBRepToolShapeExplorer::new(she, ShapeEnum::Face, ShapeEnum::Shape)?;
        while fex.more() {
            let facur = fex.current().clone();
            fex.next()?;
            if !self.my_kp_map_f1f2.is_bound(&facur) && !addedfaces.is_bound(&facur) {
                let mut fori = facur;
                if stsol == State::In {
                    fori.complement();
                }
                addedfaces.bind(fori.clone(), fori);
            }
        }
        Ok(())
    }

    /// The new shell of the faces made of each pair of tangent faces and of the other
    /// faces of the two shells (None when the map is empty).
    fn kole_new_shell(&mut self, ctx: &KoleContext) -> Result<Option<Shape>, Failure> {
        let (stsol1, stsol2) = ctx.stsol;
        let (st1, st2) = (self.my_state1, self.my_state2);
        let mut addedfaces = DataMap::default();
        let entries: Vec<(Shape, List<Shape>)> = self
            .my_kp_map_f1f2
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if entries.is_empty() {
            return Ok(None);
        }
        let mut ires = RESNEWSHE;
        for (f1, lf2) in entries {
            let Some(f2) = lf2.first().cloned() else {
                continue;
            };
            let (rankf1, rankf2) = (self.g_shape_rank(&f1), self.g_shape_rank(&f2));
            if rankf1 == 0 || rankf2 == 0 {
                continue;
            }
            let (stf1, stf2) = self.kp_class_ff(&f1, &f2)?;
            if rankf1 == 1 {
                ires = self.kp_iskoleanalyse(stf1, stf2, stsol1, stsol2).0;
            }
            if rankf1 == 2 {
                ires = self.kp_iskoleanalyse(stf2, stf1, stsol2, stsol1).0;
            }
            if ires == RESUNDEF {
                continue;
            }
            let (r1, r2) = (stsol1 == State::In, stsol2 == State::In);
            let fac = if rankf1 == 1 {
                self.kp_makeface(&f1, &lf2, stf1, stf2, r1, r2)?
            } else {
                self.kp_makeface(&f1, &lf2, stf1, stf2, r2, r1)?
            };
            if fac.is_null() {
                continue;
            }
            addedfaces.bind(fac.clone(), fac.clone());
            let sm1 = if rankf1 == 1 { st1 } else { st2 };
            let sm2 = if rankf2 == 2 { st2 } else { st1 };
            self.change_merged(&f1, sm1).append(fac.clone());
            for f in lf2.iter() {
                self.change_merged(f, sm2).append(fac.clone());
            }
            self.kole_add_shell_faces(&ctx.she.0, stsol1, &mut addedfaces)?;
            self.kole_add_shell_faces(&ctx.she.1, stsol2, &mut addedfaces)?;
        }
        if addedfaces.is_empty() {
            return Ok(Some(Shape::default()));
        }
        let mut newshe = Shape::default();
        self.my_build_tool.make_shell(&mut newshe)?;
        self.my_build_tool.closed(&mut newshe, true)?;
        for (f, _) in addedfaces.items() {
            self.my_build_tool.add_shell_face(&mut newshe, f)?;
        }
        Ok(Some(newshe))
    }

    /// The other shells of a solid kept or classified (against the other solid avoiding
    /// the tangent face) on the state of the operation.
    fn kole_other_shells(
        &mut self,
        sol: &Shape,
        she: &Shape,
        icla: i32,
        fac: &Shape,
        other_sol: &Shape,
        sta: State,
    ) -> Result<List<Shape>, Failure> {
        let mut loshe = List::default();
        let mut ex = TopOpeBRepToolShapeExplorer::new(sol, ShapeEnum::Shell, ShapeEnum::Shape)?;
        while ex.more() {
            let shecur = ex.current().clone();
            ex.next()?;
            if she.is_equal(&shecur) {
                continue;
            }
            if icla == SHECLASAUTR {
                if self.kp_clas_ss_avoiding(&shecur, fac, other_sol)? == sta {
                    loshe.append(shecur);
                }
            } else if icla == SHEGARDAUTR {
                loshe.append(shecur);
            }
        }
        Ok(loshe)
    }

    /// The new solid of the shell with the other shells of the solids, merged.
    fn kole_new_solid(
        &mut self,
        newshe: &Shape,
        icla: (i32, i32),
        ctx: &KoleContext,
    ) -> Result<(), Failure> {
        let mut newsol = Shape::default();
        if !newshe.is_null() {
            self.my_build_tool.make_solid(&mut newsol)?;
            self.my_build_tool.add_solid_shell(&mut newsol, newshe)?;
        }
        let (st1, st2) = (self.my_state1, self.my_state2);
        for (k, (sol, she, fac, other, sta)) in [
            (
                icla.0,
                (&ctx.sol.0, &ctx.she.0, &ctx.fac.0, &ctx.sol.1, st1),
            ),
            (
                icla.1,
                (&ctx.sol.1, &ctx.she.1, &ctx.fac.1, &ctx.sol.0, st2),
            ),
        ] {
            if k == SHECLASAUTR || k == SHEGARDAUTR {
                for sh in self.kole_other_shells(sol, she, k, fac, other, sta)?.iter() {
                    self.my_build_tool.add_solid_shell(&mut newsol, sh)?;
                }
            }
        }
        if !newsol.is_null() {
            let s1 = self.my_shape1.clone();
            self.change_merged(&s1, st1).append(newsol);
        }
        Ok(())
    }
}
