//! The tangent planar particular case of `TopOpeBRepBuild_Builder`
//! (TopOpeBRepBuild_kpkole.cxx): two solids touching on planar faces, one IN the other,
//! merged by the shells of these faces.
use super::TopOpeBRepBuildBuilder;
use super::kpart::kpresu::*;
use crate::top_ope_brep_build::TopOpeBRepBuildWireToFace;
use crate::top_ope_brep_ds::TopOpeBRepDSShapeShapeInterference;
use crate::top_ope_brep_tool::{TopOpeBRepToolShapeExplorer, fun_tool_plane_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// The faces of the tangent map: the first face OUT, the first face IN and all the
/// faces IN.
pub(super) struct KoleFaces {
    pub(super) f_ou: Shape,
    pub(super) f_in: Shape,
    pub(super) lf_in_first: List<Shape>,
    pub(super) lfin: List<Shape>,
}

impl TopOpeBRepBuildBuilder {
    /// `FUNKP_KPiskolesh(BU, BDS, Sarg, lShsd, lfhsd)`: the shape has solids with same
    /// domain solids, whose first one has only shape/shape interferences on its faces
    /// with geometry; `b.funkp_kpiskolesh(&s, &mut lsd)?`.
    pub fn funkp_kpiskolesh(&self, sarg: &Shape, lshsd: &mut List<Shape>) -> Result<bool, Failure> {
        if sarg.is_null() || self.kp_lhsd_list(sarg, ShapeEnum::Solid, lshsd)? == 0 {
            return Ok(false);
        }
        let sol = lshsd.first().cloned().unwrap_or_default();
        let mut lfhg = List::default();
        if self.kp_lhg_list(&sol, ShapeEnum::Face, &mut lfhg)? != 0 {
            let bds = self.hds()?.ds();
            for f in lfhg.iter() {
                for i in bds.shape_interferences(f, true).iter() {
                    let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
                    if ssi.get().is_none() {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    /// `KPiskolesh(Sarg, lShsd, lfhsd)`: a tangent shape whose same domain faces are
    /// planar with one wire and no edge with geometry; `b.kp_iskolesh(&s, &mut lsd, &mut lfsd)?`.
    pub fn kp_iskolesh(
        &self,
        sarg: &Shape,
        lshsd: &mut List<Shape>,
        lfhsd: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        if !self.funkp_kpiskolesh(sarg, lshsd)? {
            return Ok(false);
        }
        self.kp_lhsd_list(sarg, ShapeEnum::Face, lfhsd)?;
        for fac in lfhsd.iter() {
            if !fun_tool_plane_face(fac)?
                || Self::kp_ls(fac, ShapeEnum::Wire)? > 1
                || self.kp_lhg(fac, ShapeEnum::Edge)? != 0
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The face `f2` recorded IN (or OUT) of `f1` in the tangent map.
    fn kp_record(&mut self, fout: &Shape, fin: &Shape) {
        if !self.my_kp_map_f1f2.is_bound(fout) {
            self.my_kp_map_f1f2.bind(fout.clone(), List::default());
        }
        if let Ok(l) = self.my_kp_map_f1f2.change_find(fout) {
            l.append(fin.clone());
        }
        if !self.my_kp_map_f1f2.is_bound(fin) {
            self.my_kp_map_f1f2.bind(fin.clone(), List::default());
        }
    }

    /// `KPiskole()`: the shapes are single tangent solids whose same domain faces are
    /// IN/OUT of each other, recorded in the tangent map; the edges of the faces IN set
    /// as the section edges; `b.kp_iskole()?`.
    pub fn kp_iskole(&mut self) -> Result<i32, Failure> {
        let (mut lshsd1, mut lshsd2, mut lfhsd1, mut lfhsd2) = Default::default();
        if !self.kp_iskolesh(&self.my_shape1.clone(), &mut lshsd1, &mut lfhsd1)?
            || lfhsd1.is_empty()
        {
            return Ok(0);
        }
        if !self.kp_iskolesh(&self.my_shape2.clone(), &mut lshsd2, &mut lfhsd2)?
            || lfhsd2.is_empty()
        {
            return Ok(0);
        }
        if lshsd1.len() > 1 || lshsd2.len() > 1 {
            return Ok(0);
        }
        let mut les = List::default();
        for f1 in lfhsd1.iter() {
            let (mut lf1, mut lf2) = (List::default(), List::default());
            lf1.append(f1.clone());
            self.kp_same_domain(&mut lf1, &mut lf2)?;
            for f2 in lf2.iter() {
                let (ok, st1, st2) = self.kp_iskole_ff(f1, f2)?;
                if !ok {
                    return Ok(0);
                }
                if st1 == State::Out && st2 == State::In {
                    self.kp_record(f1, f2);
                } else if st2 == State::Out && st1 == State::In {
                    self.kp_record(f2, f1);
                }
                let fw = if st1 == State::In {
                    f1
                } else if st2 == State::In {
                    f2
                } else {
                    continue;
                };
                let mut ex =
                    TopOpeBRepToolShapeExplorer::new(fw, ShapeEnum::Edge, ShapeEnum::Shape)?;
                while ex.more() {
                    les.append(ex.current().clone());
                    ex.next()?;
                }
            }
        }
        let hds = self.hds()?;
        let mut ds = hds.change_ds();
        ds.init_section_edges();
        for e in les.iter() {
            ds.add_section_edge(&topods::edge(e)?);
        }
        Ok(1)
    }

    /// `KPiskoleanalyse(Stfac1, Stfac2, Stsol1, Stsol2, ires, icla1, icla2)`: the result
    /// and shell selections of the operation for two tangent solids;
    /// `let (ires, c1, c2) = b.kp_iskoleanalyse(State::Out, State::In, State::Out, State::Out);`.
    pub fn kp_iskoleanalyse(
        &self,
        stfac1: State,
        stfac2: State,
        stsol1: State,
        stsol2: State,
    ) -> (i32, i32, i32) {
        use State::{In, Out};
        let mut r = (RESUNDEF, SHEUNDEF, SHEUNDEF);
        if self.opefus() {
            r = match (stsol1, stsol2) {
                (Out, Out) => (RESNEWSHE, SHEGARDAUTR, SHEGARDAUTR),
                (Out, In) => (RESSHAPE1, SHECLASAUTR, SHEAUCU),
                (In, Out) => (RESSHAPE2, SHEAUCU, SHECLASAUTR),
                _ => r,
            };
        } else if self.opec12() {
            r = match (stsol1, stsol2) {
                (Out, Out) => (RESSHAPE1, SHEGARDAUTR, SHEAUCU),
                (Out, In) => (RESNEWSHE, SHECLASAUTR, SHEAUCU),
                (In, Out) => (RESNULL, SHEAUCU, SHEAUCU),
                _ => r,
            };
        } else if self.opec21() {
            r = match (stsol1, stsol2) {
                (Out, Out) => (RESSHAPE2, SHEAUCU, SHEGARDAUTR),
                (Out, In) => (RESNULL, SHEAUCU, SHEAUCU),
                (In, Out) => (RESNEWSHE, SHEAUCU, SHECLASAUTR),
                _ => r,
            };
        } else if self.opecom() {
            r = match (stsol1, stsol2) {
                (Out, Out) => {
                    let mut x = r;
                    if stfac1 == In {
                        x = (RESFACE1, SHEAUCU, SHEAUCU);
                    }
                    if stfac2 == In {
                        x = (RESFACE2, SHEAUCU, SHEAUCU);
                    }
                    x
                }
                (Out, In) => (RESSHAPE2, SHECLASAUTR, SHEGARDAUTR),
                (In, Out) => (RESSHAPE1, SHEGARDAUTR, SHECLASAUTR),
                _ => r,
            };
        }
        r
    }

    /// `FUNKP_KPmakefaces(BU, Fac1, LF2, Stfac1, Stfac2, R1, R2, Lres)`: the faces of the
    /// wires of the OUT face and of the faces IN (complemented when reversed);
    /// Standard_ProgramError for a face not OUT or without other rank;
    /// `b.funkp_kpmakefaces(&f1, &lf2, State::Out, State::In, false, false, &mut l)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn funkp_kpmakefaces(
        &self,
        fac1: &Shape,
        lf2: &List<Shape>,
        stfac1: State,
        _stfac2: State,
        r1: bool,
        r2: bool,
        lres: &mut List<Shape>,
    ) -> Result<(), Failure> {
        self.kp_makeface_check(fac1, stfac1)?;
        let mut wtof = TopOpeBRepBuildWireToFace::new();
        for (f, r) in std::iter::once((fac1, r1)).chain(lf2.iter().map(|f| (f, r2))) {
            for w in Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)? {
                let mut wori = w;
                if r {
                    wori.complement();
                }
                wtof.add_wire(&topods::wire(&wori)?);
            }
        }
        wtof.make_faces(&topods::face(fac1)?, lres)
    }

    /// The checks of KPmakeface: the face OUT and of a rank (the same domain lists of
    /// the other rank computed, as natively).
    fn kp_makeface_check(&self, fac1: &Shape, stfac1: State) -> Result<(), Failure> {
        if stfac1 != State::Out {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "KPmakeface Stfac1 != OUT",
                "TopOpeBRepBuild_Builder",
            ));
        }
        let rank_f = self.g_shape_rank(fac1);
        let rank_in = match rank_f {
            0 => 0,
            1 => 2,
            _ => 1,
        };
        let (mut lfso, mut lfdo) = (List::default(), List::default());
        self.g_find_sam_dom_sodo_of(fac1, &mut lfso, &mut lfdo)?;
        if rank_in == 0 {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "KPmakeface rankIN = 0",
                "TopOpeBRepBuild_Builder",
            ));
        }
        let mut lfin = List::default();
        self.g_find_same_rank(&lfso, rank_in, &mut lfin);
        self.g_find_same_rank(&lfdo, rank_in, &mut lfin);
        Ok(())
    }

    /// `KPmakeface(Fac1, LF2, Stfac1, Stfac2, R1, R2)`: an empty copy of the OUT face
    /// with its wires and those of the faces IN (complemented when reversed);
    /// `b.kp_makeface(&f1, &lf2, State::Out, State::In, false, false)?`.
    pub fn kp_makeface(
        &self,
        fac1: &Shape,
        lf2: &List<Shape>,
        stfac1: State,
        _stfac2: State,
        r1: bool,
        r2: bool,
    ) -> Result<Shape, Failure> {
        let mut fac = Face::default();
        BRepBuilder::default().make_face(&mut fac);
        let mut fac = fac.into_shape();
        if stfac1 == State::Out {
            fac = fac1.empty_copied()?;
        }
        self.kp_makeface_check(fac1, stfac1)?;
        for (f, r) in std::iter::once((fac1, r1)).chain(lf2.iter().map(|f| (f, r2))) {
            let mut wex = TopOpeBRepToolShapeExplorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)?;
            while wex.more() {
                let mut wori = wex.current().clone();
                if r {
                    wori.complement();
                }
                self.my_build_tool.add_face_wire(&mut fac, &wori)?;
                wex.next()?;
            }
        }
        Ok(fac)
    }

    /// The faces of the tangent map: the first OUT face, its first IN face and all the
    /// IN faces (None when the map has none).
    pub(super) fn kole_faces(&self) -> Option<KoleFaces> {
        let mut r: Option<KoleFaces> = None;
        let mut lfin = List::default();
        for (k, los) in self.my_kp_map_f1f2.items() {
            if los.is_empty() {
                continue;
            }
            if r.is_none() {
                r = Some(KoleFaces {
                    f_ou: k.clone(),
                    f_in: los.first().cloned().unwrap_or_default(),
                    lf_in_first: los.clone(),
                    lfin: List::default(),
                });
            }
            for f in los.iter() {
                lfin.append(f.clone());
            }
        }
        r.map(|mut r| {
            r.lfin = lfin;
            r
        })
    }

    /// The shell of the solid containing the face.
    pub(super) fn shell_of_face_in(sol: &Shape, fac: &Shape) -> Result<Shape, Failure> {
        let mut m = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(sol, ShapeEnum::Face, ShapeEnum::Shell, &mut m)?;
        Ok(m.find_from_key(fac)?.first().cloned().unwrap_or_default())
    }
}

/// The data of a tangent merge: the solids, the shells and the tangent faces of each
/// shape, the solid states, the rank of the OUT face and the tangent map faces.
pub(super) struct KoleContext {
    pub(super) sol: (Shape, Shape),
    pub(super) she: (Shape, Shape),
    pub(super) fac: (Shape, Shape),
    pub(super) stsol: (State, State),
    pub(super) rank_ou: i32,
    pub(super) kf: KoleFaces,
}
