//! The tangent particular case of `TopOpeBRepBuild_Builder` with planar and cylindrical
//! faces (iskoletge of TopOpeBRepBuild_KPart.cxx): two solids touching on a face whose
//! outer wire edges are all section edges, rebuilt into one solid.
use super::TopOpeBRepBuildBuilder;
use super::kpart::kpresu::*;
use crate::top_ope_brep_ds::export::fun_ds_getoov_hds;
use crate::top_ope_brep_ds::{
    TopOpeBRepDSBuildTool, TopOpeBRepDSConfig, TopOpeBRepDSHDataStructure,
    fdscnx_face_edge_connex_faces,
};
use crate::top_ope_brep_tool::{
    correct_on_uviso, fun_tool_comparebndkole, fun_tool_cylinder_face, fun_tool_pcurve_on_f,
    fun_tool_plane_face, fun_tool_same_ori, fun_tool_shapes,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::outer_wire;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidExplorer;

/// The new edges by edge of the rebuilt faces.
type EdgeMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `FUN_sortplcy(lof, lopl, locy)` (static): the planar and cylindrical faces.
pub(super) fn fun_sortplcy(lof: &List<Shape>) -> Result<(List<Shape>, List<Shape>), Failure> {
    let (mut lopl, mut locy) = (List::default(), List::default());
    for f in lof.iter() {
        if fun_tool_plane_face(f)? {
            lopl.append(f.clone());
        }
        if fun_tool_cylinder_face(f)? {
            locy.append(f.clone());
        }
    }
    Ok((lopl, locy))
}

/// `FUN_addf(sta, ftoadd, map)` (static): the face (complemented IN) added.
fn fun_addf(sta: State, ftoadd: &Shape, map: &mut DataMap<Shape, Shape, ShapeMapHasher>) {
    let mut fori = ftoadd.clone();
    if sta == State::In {
        fori.complement();
    }
    map.bind(fori.clone(), fori);
}

/// `FUN_changev(HDS, v)` (static): a vertex of the second shape with same domain.
fn fun_changev(hds: &TopOpeBRepDSHDataStructure, v: &Shape) -> bool {
    hds.has_shape(v, true) && hds.has_same_domain(v, true) && hds.ds().ancestor_rank(v) == 2
}

/// `FUN_updatev(HDS, newed, v, oriv, parv, changev)` (static): the vertex (or its same
/// domain vertex) added to the new edge at its parameter; false without one.
fn fun_updatev(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    newed: &mut Shape,
    v: &Vertex,
    oriv: Orientation,
    parv: f64,
    changev: bool,
) -> Result<bool, Failure> {
    let (bt, bb) = (TopOpeBRepDSBuildTool::new(), BRepBuilder::default());
    let ov = if changev {
        let Some(oov) = fun_ds_getoov_hds(v.as_ref(), hds)? else {
            return Ok(false);
        };
        oov.oriented(oriv)
    } else {
        v.as_ref().oriented(oriv)
    };
    bb.add(newed, &ov)?;
    bt.parameter(newed, &ov, parv)?;
    Ok(true)
}

impl TopOpeBRepBuildBuilder {
    /// The copy of an edge with the same domain vertices of its vertices of the second
    /// shape (None when none changes, Err-free false when a vertex has no same domain).
    fn edge_with_changed_vertices(&self, ed: &Edge) -> Result<Option<Option<Shape>>, Failure> {
        let hds = self.hds()?;
        let (vf, vl) = topexp::vertices(ed, false);
        let (cvf, cvl) = (fun_changev(hds, vf.as_ref()), fun_changev(hds, vl.as_ref()));
        if !(cvf || cvl) {
            return Ok(Some(None));
        }
        let (ff, l) = (BRepTool::parameter(&vf, ed)?, BRepTool::parameter(&vl, ed)?);
        let mut newed = Shape::default();
        TopOpeBRepDSBuildTool::new()
            .copy_edge(&ed.as_ref().oriented(Orientation::Forward), &mut newed)?;
        if !fun_updatev(
            &self.my_data_structure,
            &mut newed,
            &vf,
            Orientation::Forward,
            ff,
            cvf,
        )? || !fun_updatev(
            &self.my_data_structure,
            &mut newed,
            &vl,
            Orientation::Reversed,
            l,
            cvl,
        )? {
            return Ok(None);
        }
        newed.set_orientation(ed.orientation());
        Ok(Some(Some(newed)))
    }

    /// The edges of the outer wire edge in the rebuilt face: its ON splits (with
    /// p-curves on the face), its new edges, or itself; None to fail.
    fn makefaces_edge_parts(
        &self,
        ed: &Edge,
        f: &Shape,
        enew_e: &mut EdgeMap,
        loe: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let oe = ed.orientation();
        let issplit = self.is_split(ed.as_ref(), State::On);
        let mut isbound = enew_e.is_bound(ed.as_ref());
        if !isbound && !issplit {
            match self.edge_with_changed_vertices(ed)? {
                None => return Ok(false),
                Some(Some(newed)) => {
                    let mut led = List::default();
                    led.append(newed);
                    enew_e.bind(ed.clone().into_shape(), led);
                    isbound = true;
                }
                Some(None) => {}
            }
        }
        if issplit {
            let speds = self.splits(ed.as_ref(), State::On).clone();
            let Some(spe) = speds.first() else {
                return Ok(false);
            };
            let spe = topods::edge(spe)?;
            let mut orisp = spe.orientation();
            if !fun_tool_same_ori(ed, &spe)? {
                orisp = top_abs::complement(orisp);
            }
            for e in speds.iter() {
                let esp = topods::edge(e)?;
                if !fun_tool_pcurve_on_f(&topods::face(f)?, &esp)? {
                    return Ok(false);
                }
                loe.append(esp.as_ref().oriented(orisp));
            }
        } else if isbound {
            for ne in enew_e.find(ed.as_ref())?.iter() {
                loe.append(ne.oriented(oe));
            }
        } else {
            loe.append(ed.clone().into_shape());
        }
        Ok(true)
    }

    /// `FUN_makefaces(BU, HDS, EnewE, f, ...)` (static): the face rebuilt on its outer
    /// wire made of the splits or new edges of its edges (and its other wires),
    /// corrected on the UV isos; false to fail.
    fn kp_makefaces(
        &self,
        enew_e: &mut EdgeMap,
        f: &Shape,
        newfaces: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let ff = topods::face(&f.oriented(Orientation::Forward))?;
        let of = f.orientation();
        let mut loe = List::default();
        let outerwf = outer_wire(&ff)?;
        for e in Explorer::new(outerwf.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !self.makefaces_edge_parts(&topods::edge(&e)?, f, enew_e, &mut loe)? {
                return Ok(false);
            }
        }
        let bb = BRepBuilder::default();
        let mut new_w = Wire::default();
        bb.make_wire(&mut new_w);
        let mut new_w = new_w.into_shape();
        for e in loe.iter() {
            bb.add(&mut new_w, e)?;
        }
        let mut newf = ff.as_ref().empty_copied()?;
        bb.add(&mut newf, &new_w)?;
        for w in Explorer::new(ff.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)? {
            if !w.is_same(outerwf.as_ref()) {
                bb.add(&mut newf, &w)?;
            }
        }
        if of == Orientation::Reversed {
            newf.set_orientation(Orientation::Reversed);
        }
        let mut corrected = topods::face(&newf)?;
        if correct_on_uviso(&topods::face(f)?, &mut corrected)? {
            newfaces.append(corrected.into_shape());
        } else {
            newfaces.append(newf);
        }
        Ok(true)
    }

    /// `FUN_rebuildfc(BU, HDS, ..., Fk, EnewE, fcnewfc)` (static): each face connected
    /// to the face by an edge of its outer wire rebuilt; false to fail.
    fn kp_rebuildfc(
        &self,
        fk: &Shape,
        enew_e: &mut EdgeMap,
        fcnewfc: &mut IndexedDataMapOfShapeListOfShape,
    ) -> Result<bool, Failure> {
        let owk = outer_wire(&topods::face(fk)?)?;
        let mut eds = List::default();
        fun_tool_shapes(owk.as_ref(), ShapeEnum::Edge, &mut eds)?;
        for ed in eds.iter() {
            let lfc = fdscnx_face_edge_connex_faces(fk, ed, &self.my_data_structure)?;
            if lfc.len() != 1 {
                return Ok(false);
            }
            let fc = lfc.first().cloned().unwrap_or_default();
            let mut newfc = List::default();
            if !self.kp_makefaces(enew_e, &fc, &mut newfc)? {
                return Ok(false);
            }
            fcnewfc.add(fc, newfc);
        }
        Ok(true)
    }

    /// `FUN_comparekoletgesh(SC, sh1, sh2, fkole1, ...)` (static): 2 when a point of a
    /// face of `sh1` (not the tangent one) is IN `sh2`, 1 when OUT, else 0.
    fn kp_comparekoletgesh(
        &mut self,
        sh1: &Shape,
        sh2: &Shape,
        fkole1: &Shape,
    ) -> Result<i32, Failure> {
        self.my_shape_classifier.set_reference(sh2)?;
        for f1 in Explorer::new(sh1, ShapeEnum::Face, ShapeEnum::Shape)? {
            if f1.is_same(fkole1) {
                continue;
            }
            let mut pnt1 = Point3::default();
            BRepClass3dSolidExplorer::find_a_point_in_the_face_point(
                &topods::face(&f1)?,
                &mut pnt1,
            )?;
            self.my_shape_classifier.state_p3d_reference(pnt1)?;
            match self.my_shape_classifier.state() {
                State::In => return Ok(2),
                State::Out => return Ok(1),
                _ => {}
            }
        }
        Ok(0)
    }

    /// The faces of the shells of the tangent faces and of the other faces of each
    /// shape added (complemented IN), the rebuilt connected faces merged.
    #[allow(clippy::too_many_arguments)]
    fn koletge_new_shell(
        &mut self,
        pf: (&Shape, &Shape),
        sta: (State, State),
        sha: (&Shape, &Shape),
    ) -> Result<Option<Shape>, Failure> {
        let mut addedfaces = DataMap::default();
        let mut enew_e = EdgeMap::default();
        let (mut fcsma, mut fcgre) = (
            IndexedDataMapOfShapeListOfShape::default(),
            IndexedDataMapOfShapeListOfShape::default(),
        );
        for (pfx, stx, fc) in [(pf.0, sta.0, &mut fcsma), (pf.1, sta.1, &mut fcgre)] {
            if !self.kp_rebuildfc(pfx, &mut enew_e, fc)? {
                return Ok(None);
            }
            for i in 1..=fc.len() {
                let f = fc.find_key(i)?.clone();
                for ff in fc.find_from_index(i)?.clone().iter() {
                    fun_addf(stx, ff, &mut addedfaces);
                    self.change_merged(&f, stx).append(ff.clone());
                }
            }
        }
        for (shax, pfx, stx, fc) in [(sha.0, pf.0, sta.0, &fcsma), (sha.1, pf.1, sta.1, &fcgre)] {
            for ff in Explorer::new(shax, ShapeEnum::Face, ShapeEnum::Shape)? {
                if !fc.contains(&ff) && !ff.is_same(pfx) {
                    fun_addf(stx, &ff, &mut addedfaces);
                }
            }
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

    /// The shell of the shape containing the face.
    fn shell_of_face(sha: &Shape, f: &Shape) -> Result<Shape, Failure> {
        let mut m = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(sha, ShapeEnum::Face, ShapeEnum::Shell, &mut m)?;
        Ok(m.find_from_key(f)?.first().cloned().unwrap_or_default())
    }

    /// `MergeKPartiskoletge()`: the tangent solids merged: the shell of one or the
    /// other, or a new solid of the rebuilt faces; `b.merge_k_part_iskoletge()?`.
    pub fn merge_k_part_iskoletge(&mut self) -> Result<(), Failure> {
        if self.my_is_k_part != 5 {
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
        self.kp_iskoletgesh(&s1, &mut lshsd1, &mut lfhsd1)?;
        self.kp_iskoletgesh(&s2, &mut lshsd2, &mut lfhsd2)?;
        let sol1: Shape = lshsd1.first().cloned().unwrap_or_default();
        let sol2: Shape = lshsd2.first().cloned().unwrap_or_default();
        self.change_merged(&sol1, st1);
        self.change_merged(&sol2, st2);
        let fac1 = fun_sortplcy(&lfhsd1)?
            .0
            .first()
            .cloned()
            .unwrap_or_default();
        let fac2 = fun_sortplcy(&lfhsd2)?
            .0
            .first()
            .cloned()
            .unwrap_or_default();
        let config2 = self.hds()?.ds().same_domain_ori(&fac2);
        let mut rgre = 1;
        if config2 == TopOpeBRepDSConfig::SameOriented {
            rgre = self.kp_comparekoletgesh(&s1, &s2, &fac1)?;
            if rgre == 0 {
                rgre = fun_tool_comparebndkole(&s1, &s2)?;
            }
        }
        if rgre == 0 {
            return Ok(());
        }
        let rsma = if rgre == 1 { 2 } else { 1 };
        let pick = |r: i32| {
            if r == 1 {
                (&fac1, st1, &s1)
            } else {
                (&fac2, st2, &s2)
            }
        };
        let ((pf_sma, sta_sma, sha_sma), (pf_gre, sta_gre, sha_gre)) = (pick(rsma), pick(rgre));
        let ires = self.kp_iskoletgeanalyse(config2, sta_sma, sta_gre);
        let she_sma = Self::shell_of_face(sha_sma, pf_sma)?;
        let she_gre = Self::shell_of_face(sha_gre, pf_gre)?;
        self.change_merged(&she_sma, sta_sma);
        self.change_merged(&she_gre, sta_gre);
        let newshe = match ires {
            RESNULL => return Ok(()),
            RESSHAPE1 => she_sma,
            RESSHAPE2 => she_gre,
            RESNEWSOL => match self.koletge_new_shell(
                (pf_sma, pf_gre),
                (sta_sma, sta_gre),
                (sha_sma, sha_gre),
            )? {
                Some(s) => s,
                None => return Ok(()),
            },
            _ => Shape::default(),
        };
        if !newshe.is_null() {
            let mut newsol = Shape::default();
            self.my_build_tool.make_solid(&mut newsol)?;
            self.my_build_tool.add_solid_shell(&mut newsol, &newshe)?;
            self.change_merged(&s1, st1).append(newsol);
        }
        Ok(())
    }
}
