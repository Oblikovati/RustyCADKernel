//! The members of `TopOpeBRepDS_FaceInterferenceTool`.
use super::*;

/// The UV of the point of the edge on the face: on its pcurve when `on_face`, else
/// (or failing) by projection; Standard_ProgramError when both fail.
fn uv_on(
    e: &topods::Edge,
    (p, par): (Point3, f64),
    f: &topods::Face,
    on_face: bool,
    project_on_failure: bool,
) -> Result<Point2, Failure> {
    let mut uv = Point2::default();
    let mut ok = on_face && fun_tool_paron_ef(e, par, f, &mut uv)?;
    if !ok && (project_on_failure || !on_face) {
        let mut d = 0.;
        ok = fun_tool_proj_ponf(p, f, &mut uv, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?;
    }
    if !ok {
        return Err(raise());
    }
    Ok(uv)
}

impl<'a> TopOpeBRepDSFaceInterferenceTool<'a> {
    /// `TopOpeBRepDS_FaceInterferenceTool(PBDS)`;
    /// `TopOpeBRepDSFaceInterferenceTool::new(&ds)`.
    pub fn new(pbds: &'a TopOpeBRepDSDataStructure) -> Self {
        Self {
            my_pbds: pbds,
            myrefdef: false,
            my_face_orientation: Orientation::Forward,
            my_face_oriented: 0,
            my_tool: SurfaceTransition::new(),
            my_edge: Shape::default(),
            my_pnt_on_ed: Point3::default(),
            my_par_on_ed: 0.,
            my_on_ed_def: false,
            my_tole: 0.,
        }
    }

    /// `Init(FI, E, Eisnew, I)`: the reference face `fi` along the edge `e` (opposite
    /// to its geometry edge when new and DIFFORIENTED), its frame at the point of the
    /// edge; nothing for no shape/shape interference; Standard_ProgramError when a
    /// parameter or tangent is missing; `t.init(&fi, &e, false, &i)?`.
    pub fn init(
        &mut self,
        ffi: &Shape,
        ee: &Shape,
        ee_is_new: bool,
        iin: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<(), Failure> {
        let i = ssi(iin);
        let Some(ii) = i.get() else {
            return Ok(());
        };
        let (fi, e) = (topods::face(ffi)?, topods::edge(ee)?);
        STATIC_TOREVERSE.with(|c| c.set(false));
        if ee_is_new {
            let eg = topods::edge(self.my_pbds.shape(ii.geometry(), true))?;
            let cf = fds_config3d(&e, &eg)?.ok_or_else(raise)?;
            if cf == TopOpeBRepDSConfig::DiffOriented {
                STATIC_TOREVERSE.with(|c| c.set(true));
            }
        }
        self.my_face_orientation = fi.orientation();
        self.my_face_oriented = ii.support();
        self.my_edge = e.clone().into_shape();
        let (edonfa, _) = fun_tool_orient_e_in_f_forward(&e, &fi)?;
        if !self.my_on_ed_def {
            (self.my_pnt_on_ed, self.my_par_on_ed) = fun_tool_find_pin_e(&e)?;
        }
        self.my_tole = precision::angular();
        let uv = uv_on(
            &e,
            (self.my_pnt_on_ed, self.my_par_on_ed),
            &fi,
            edonfa,
            false,
        )?;
        let tgt = dir(tggeom_e(self.my_par_on_ed, &e)?.ok_or_else(raise)?)?;
        let (norm, d1, d2, cur1, cur2) = fun_compute_geom_data(&fi, uv)?;
        self.my_tool.reset(tgt, norm, d1, d2, cur1, cur2)?;
        self.myrefdef = true;
        Ok(())
    }

    /// The orientation of the geometry edge `eg` in `ft` (else in `fi`), complemented
    /// for a new edge opposite to it; Standard_ProgramError when in neither.
    fn oritan(
        &self,
        eg: &topods::Edge,
        (fi, ft): (&topods::Face, &topods::Face),
        ee_is_new: bool,
        same: bool,
    ) -> Result<(bool, Orientation), Failure> {
        let (egofft, ori_eft) = fun_tool_orient_e_in_f_forward(eg, ft)?;
        let (egoffi, ori_efi) = fun_tool_orient_e_in_f_forward(eg, fi)?;
        let ori = if egofft {
            ori_eft
        } else if egoffi {
            ori_efi
        } else {
            return Err(raise());
        };
        let rev = ee_is_new && !same;
        Ok((egofft, if rev { top_abs::complement(ori) } else { ori }))
    }

    /// `Add(FI, FT, E, Eisnew, I)`: the face `ft` along the edge (the reference edge,
    /// Standard_ProgramError otherwise) compared (`Init` with `fi` when the reference
    /// is undefined); `t.add(&fi, &ft, &e, false, &i)?`.
    pub fn add(
        &mut self,
        ffi: &Shape,
        fft: &Shape,
        ee: &Shape,
        ee_is_new: bool,
        iin: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<(), Failure> {
        let i = ssi(iin);
        if i.is_null() {
            return Ok(());
        }
        let (fi, ft, e) = (topods::face(ffi)?, topods::face(fft)?, topods::edge(ee)?);
        self.my_pbds.shape_index(&fi, true);
        if !e.is_same(&self.my_edge) {
            return Err(raise());
        }
        let i: Handle<TopOpeBRepDSInterference> = i.cast();
        if !self.myrefdef {
            return self.init(&fi, &e, ee_is_new, &i);
        }
        let (_, g, _, _) = fds_data(&i);
        let eg = topods::edge(self.my_pbds.shape(g, true))?;
        fds_has_same_domain3d(self.my_pbds, &eg, None);
        let same = !STATIC_TOREVERSE.with(|c| c.get());
        let mut oriloc = iref(&i)?
            .transition()
            .orientation(State::In, ShapeEnum::Face);
        if !same && (oriloc == Orientation::Forward || oriloc == Orientation::Reversed) {
            oriloc = top_abs::complement(oriloc);
        }
        let (egofft, oritan) = self.oritan(&eg, (&fi, &ft), ee_is_new, same)?;
        let uv = uv_on(
            &e,
            (self.my_pnt_on_ed, self.my_par_on_ed),
            &ft,
            egofft,
            true,
        )?;
        let (norm, d1, d2, cur1, cur2) = fun_compute_geom_data(&ft, uv)?;
        self.my_tool
            .compare(self.my_tole, norm, d1, d2, cur1, cur2, oriloc, oritan)
    }

    /// `Add(E, C, I)`: nothing; `t.add_curve(&e, &c, &i)`.
    pub fn add_curve(
        &mut self,
        _e: &Shape,
        _c: &TopOpeBRepDSCurve,
        _i: &Handle<TopOpeBRepDSInterference>,
    ) {
    }

    /// `Transition(I)`: the transition of `i` set to IN/IN (INTERNAL face), OUT/OUT
    /// (EXTERNAL) or the accumulated states with the reference face as support
    /// (complemented after an opposite `Init`); `t.transition(&i)?`.
    pub fn transition(&self, i: &Handle<TopOpeBRepDSInterference>) -> Result<(), Failure> {
        let ii = iref(i)?;
        let face = ShapeEnum::Face;
        match self.my_face_orientation {
            Orientation::Internal => {
                ii.change_transition(|t| t.set(State::In, State::In, face, face))
            }
            Orientation::External => {
                ii.change_transition(|t| t.set(State::Out, State::Out, face, face))
            }
            _ => {
                ii.set_support(self.my_face_oriented);
                let (stb, sta) = (self.my_tool.state_before(), self.my_tool.state_after());
                let toreverse = STATIC_TOREVERSE.with(|c| c.get());
                ii.change_transition(|t| {
                    t.set(stb, sta, face, face);
                    let mut o = t.orientation(State::In, face);
                    if toreverse && (o == Orientation::Forward || o == Orientation::Reversed) {
                        o = top_abs::complement(o);
                    }
                    t.set_orientation(o);
                });
            }
        }
        Ok(())
    }

    /// `SetEdgePntPar(P, p)`: the point and parameter on the edge used by `Init`;
    /// `t.set_edge_pnt_par(p, 0.5)`.
    pub fn set_edge_pnt_par(&mut self, p: Point3, par: f64) {
        (self.my_pnt_on_ed, self.my_par_on_ed, self.my_on_ed_def) = (p, par, true);
    }

    /// `GetEdgePntPar(P, p)`: Standard_ProgramError when not set;
    /// `let (p, par) = t.get_edge_pnt_par()?;`.
    pub fn get_edge_pnt_par(&self) -> Result<(Point3, f64), Failure> {
        if !self.my_on_ed_def {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "GetEdgePntPar",
                "TopOpeBRepDS_FaceInterferenceTool",
            ));
        }
        Ok((self.my_pnt_on_ed, self.my_par_on_ed))
    }

    /// `IsEdgePntParDef()`; `t.is_edge_pnt_par_def()`.
    pub fn is_edge_pnt_par_def(&self) -> bool {
        self.my_on_ed_def
    }
}
