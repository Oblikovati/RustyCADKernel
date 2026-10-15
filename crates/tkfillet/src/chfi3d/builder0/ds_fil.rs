// Rust translation of ChFi3d_FilDS and its static helpers (ChFi3d_Orientation, ChFi3d_Contains,
// QueryAddVertexInEdge, CutEdge, findIndexPoint) of ChFi3d_Builder_0.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi3d_FilDS`: the surfaces, curves and points of a stripe of fillets put in the
//! TopOpeBRepDS data structure with their interferences.
use super::ds_tools::{fil_point_in_ds, fil_vertex_in_ds, null};
use crate::chfi_ds::{CommonPoint, Regul, SpineHandle, StripeHandle, SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSDataStructure,
    TopOpeBRepDSKind, TopOpeBRepDSSolidSurfaceInterference, TopOpeBRepDSSurfaceCurveInterference,
    TopOpeBRepDSTransition,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

/// The static `ChFi3d_Orientation(LI, igros, ipetit, Or, isvertex, aprendre)`: the
/// transition (on IN) of the first interference of support `igros` and geometry `ipetit`
/// (of the kind when `aprendre`); None when none.
fn interference_orientation(
    li: &ListOfInterference,
    igros: i32,
    ipetit: i32,
    isvertex: bool,
    aprendre: bool,
) -> Option<Orientation> {
    let typepetit = if isvertex {
        TopOpeBRepDSKind::Vertex
    } else {
        TopOpeBRepDSKind::Point
    };
    li.iter().filter_map(|i| i.get()).find_map(|cur| {
        let (gk, g, _, s) = cur.gkgsks();
        let hit = s == igros && g == ipetit && (!aprendre || gk == typepetit);
        hit.then(|| cur.transition().orientation(State::In, ShapeEnum::Face))
    })
}

/// The static `ChFi3d_Contains(LI, igros, ipetit, isvertex, aprendre)`.
pub(super) fn contains(
    li: &ListOfInterference,
    igros: i32,
    ipetit: i32,
    isvertex: bool,
    aprendre: bool,
) -> bool {
    interference_orientation(li, igros, ipetit, isvertex, aprendre).is_some()
}

/// The static `QueryAddVertexInEdge(LI, IC, IV, par, Or)`: the curve/vertex interference
/// appended unless an equal one is in the list.
fn query_add_vertex_in_edge(
    li: &mut ListOfInterference,
    (ic, iv): (i32, i32),
    par: f64,
    or: Orientation,
) {
    for cur in li.iter() {
        let cpi: Handle<TopOpeBRepDSCurvePointInterference> = cur.clone().cast();
        if let Some(c) = cpi.get() {
            let same = iv == c.geometry()
                && c.geometry_type() == TopOpeBRepDSKind::Vertex
                && or == c.transition().orientation(State::In, ShapeEnum::Face)
                && (par - c.parameter()).abs() < 1.0e-10;
            if same {
                return;
            }
        }
    }
    li.append(fil_vertex_in_ds(or, ic, iv, par));
}

/// The static `CutEdge(V, SD, DStr, isfirst, ons)`: on a fillet along an edge, the vertex
/// added to the edge interferences at its parameter (reversed orientation).
fn cut_edge(
    v: &Vertex,
    sd: &SurfDataHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    ons: i32,
) -> Result<(), Failure> {
    let ic = {
        let g = lock(sd);
        if !g.is_on_curve(ons) {
            return Ok(());
        }
        g.index_of_c(ons)?
    };
    let iv = dstr.add_shape(v.as_ref());
    let e = topods::edge(&dstr.shape(ic, true).oriented(Orientation::Forward))?;
    let mut found = Vec::new();
    for vv in Explorer::new(e.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if vv.is_same(v.as_ref()) {
            found.push((
                top_abs::reverse(vv.orientation()),
                BRepTool::parameter(&topods::vertex(&vv)?, &e)?,
            ));
        }
    }
    let li = dstr.change_shape_interferences_index(ic)?;
    for (or, par) in found {
        query_add_vertex_in_edge(li, (ic, iv), par, or);
    }
    Ok(())
}

/// The static `findIndexPoint(DStr, Fd, OnS, ipoin)`: the point of the structure on a
/// curve of the fillet surface at the last vertex on the face.
pub(super) fn find_index_point(
    dstr: &TopOpeBRepDSDataStructure,
    fd: &SurfDataHandle,
    on_s: i32,
) -> Result<Option<i32>, Failure> {
    let (p, isurf) = {
        let g = lock(fd);
        (g.vertex(false, on_s).point(), g.surf())
    };
    for sci in dstr.surface_interferences(isurf).iter() {
        let sc: Handle<TopOpeBRepDSSurfaceCurveInterference> = sci.clone().cast();
        let Some(sc) = sc.get() else { continue };
        for cpi in dstr.curve_interferences(sc.geometry()).iter() {
            let cp: Handle<TopOpeBRepDSCurvePointInterference> = cpi.clone().cast();
            let Some(cp) = cp.get() else { continue };
            let tp = dstr.point(cp.geometry())?;
            if p.is_equal(&tp.point(), tp.tolerance()) {
                return Ok(Some(cp.geometry()));
            }
        }
    }
    Ok(None)
}

/// The state of `ChFi3d_FilDS` carried from one fillet to the next.
pub(super) struct FilDs {
    pub(super) solid_index: i32,
    pub(super) closed: bool,
    pub(super) spine: Option<SpineHandle>,
    pub(super) seq: Vec<SurfDataHandle>,
    pub(super) ipoin: (i32, i32),
    pub(super) num_edge: i32,
    pub(super) bout_de_vtx: Vertex,
    pub(super) icurv: i32,
    pub(super) iarc: (i32, i32),
    pub(super) trafil: (Orientation, Orientation),
    pub(super) par: (f64, f64),
    pub(super) pcurv: Handle<dyn Geom2dCurve>,
    pub(super) regcout: Regul,
    pub(super) regfilfil: Regul,
    pub(super) v3: CommonPoint,
    pub(super) v4: CommonPoint,
    pub(super) degene: bool,
    pub(super) is_vertex: (bool, bool),
    pub(super) singulier_en_bout: bool,
    pub(super) tol: (f64, f64),
}

/// The degenerated traces of a multi-fillet stripe: their curves nullified, and the
/// common points of the fillets around an end trace merged on its point.
fn nullify_degenerated(
    seq: &[SurfDataHandle],
    dstr: &mut TopOpeBRepDSDataStructure,
) -> Result<(), Failure> {
    let n = seq.len();
    for (k, fd) in seq.iter().enumerate() {
        let j = k + 1;
        for on_s in 1..=2 {
            let (ic, len) = {
                let g = lock(fd);
                let fi = g.interference(on_s);
                (
                    fi.line_index(),
                    (fi.first_parameter() - fi.last_parameter()).abs(),
                )
            };
            if ic == 0 || len > precision::p_confusion() {
                continue;
            }
            *dstr.change_curve(ic).change_curve() = Handle::default();
            if j != 1 && j != n {
                continue;
            }
            let isfirst = j == 1;
            let other = &seq[if isfirst { j } else { j - 2 }];
            let on_arc = lock(fd).vertex(isfirst, on_s).is_on_arc()
                && lock(other).vertex(isfirst, on_s).is_on_arc();
            if on_arc {
                let p2 = lock(fd).change_vertex(!isfirst, on_s).point();
                let mut go = lock(other);
                let cp1 = go.change_vertex(isfirst, on_s);
                cp1.reset();
                cp1.set_point(p2);
                let p1 = cp1.point();
                drop(go);
                let mut gf = lock(fd);
                let cp2 = gf.change_vertex(!isfirst, on_s);
                cp2.reset();
                cp2.set_point(p1);
            }
        }
    }
    Ok(())
}

/// `ChFi3d_FilDS(SolidIndex, CorDat, DStr, reglist, tol3d, tol2d)`: each fillet of the
/// stripe put in the structure: its surface on the solid, its traces on the faces (or
/// fillet surfaces) with their end points, the cuts of the edges at the vertices, the
/// curves between consecutive fillets and at the stripe ends, the regularities noted;
/// `fil_ds(1, &stripe, &mut ds, &mut reg, 1e-4, 1e-5)?`.
pub fn fil_ds(
    solid_index: i32,
    cor_dat: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    reglist: &mut Vec<Regul>,
    tol3d: f64,
    tol2d: f64,
) -> Result<(), Failure> {
    let (spine, seq, ipoin) = {
        let g = lock(cor_dat);
        let seq = g
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| null("ChFi3d_FilDS: no surface data"))?;
        let seq = lock(&seq).clone();
        (
            g.spine().cloned(),
            seq,
            (g.index_first_point_on_s1(), g.index_first_point_on_s2()),
        )
    };
    let closed = spine.as_ref().is_some_and(|s| lock(s).is_periodic());
    if seq.len() > 1 {
        nullify_degenerated(&seq, dstr)?;
    }
    let mut st = FilDs {
        solid_index,
        closed,
        spine,
        seq,
        ipoin,
        num_edge: 1,
        bout_de_vtx: Vertex::default(),
        icurv: 0,
        iarc: (0, 0),
        trafil: (Orientation::Forward, Orientation::Forward),
        par: (0., 0.),
        pcurv: Handle::default(),
        regcout: Regul::new(),
        regfilfil: Regul::new(),
        v3: CommonPoint::new(),
        v4: CommonPoint::new(),
        degene: false,
        is_vertex: (false, false),
        singulier_en_bout: false,
        tol: (tol3d, tol2d),
    };
    for j in 1..=st.seq.len() {
        st.fillet(j, cor_dat, dstr, reglist)?;
    }
    Ok(())
}

impl FilDs {
    /// Whether the ends of the fillet `j` are already in the structure.
    fn in_ds(&self, j: usize, cor_dat: &StripeHandle) -> (bool, bool) {
        let (n, j) = (self.seq.len() as i32, j as i32);
        let g = lock(cor_dat);
        let (first, last) = (g.is_in_ds(true), g.is_in_ds(false));
        let (mut d1, mut d2) = (false, false);
        if j <= first {
            d1 = true;
            d2 = j < first;
        }
        if n - j < last {
            d2 = true;
            d1 = d1 || n - j + 1 < last;
        }
        (d1, d2)
    }

    /// One fillet of the stripe.
    fn fillet(
        &mut self,
        j: usize,
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        reglist: &mut Vec<Regul>,
    ) -> Result<(), Failure> {
        let fd = self.seq[j - 1].clone();
        let (isurf, or_fd) = {
            let g = lock(&fd);
            (g.surf(), g.orientation())
        };
        let (in_ds1, in_ds2) = self.in_ds(j, cor_dat);
        let ssi = TopOpeBRepDSSolidSurfaceInterference::new(
            &TopOpeBRepDSTransition::from_orientation(or_fd),
            TopOpeBRepDSKind::Solid,
            self.solid_index,
            TopOpeBRepDSKind::Surface,
            isurf,
        );
        dstr.change_shape_interferences_index(self.solid_index)?
            .append(Handle::new(ssi).cast());
        let (v1, v2) = {
            let g = lock(&fd);
            (
                g.vertex_first_on_s1().clone(),
                g.vertex_first_on_s2().clone(),
            )
        };
        if j > 1 {
            self.check_arcs((&v1, &v2), dstr)?;
        }
        (self.v3, self.v4) = {
            let g = lock(&fd);
            (g.vertex_last_on_s1().clone(), g.vertex_last_on_s2().clone())
        };
        self.transitions(&fd, dstr);
        let mut et1 = top_abs::reverse(self.trafil.0);
        self.cut_edges(&fd, (&v1, &v2), dstr)?;
        if j == 1 {
            self.first_fillet(&fd, (&v1, &v2), cor_dat, dstr, &mut et1, in_ds1)?;
        } else if !in_ds1 {
            self.between_fillets(&fd, dstr, reglist, et1)?;
        }
        self.face_trace(&fd, 1, j, cor_dat, dstr, reglist, in_ds2)?;
        self.face_trace(&fd, 2, j, cor_dat, dstr, reglist, in_ds2)?;
        let et1 = self.trafil.0;
        if j == self.seq.len() {
            if !in_ds2 {
                self.last_fillet(isurf, cor_dat, dstr, reglist, et1)?;
            }
        } else if !in_ds2 {
            self.arete_to_next(&fd, dstr, et1)?;
        }
        self.end_points(&fd, j, cor_dat, dstr)
    }

    /// The arcs of the first points of the fillet on the same arcs as the last points of
    /// the previous one, with a different transition, added to the arc interferences.
    fn check_arcs(
        &mut self,
        (v1, v2): (&CommonPoint, &CommonPoint),
        dstr: &mut TopOpeBRepDSDataStructure,
    ) -> Result<(), Failure> {
        for (v, vp, iarc, ipoin) in [
            (v1, self.v3.clone(), self.iarc.0, self.ipoin.0),
            (v2, self.v4.clone(), self.iarc.1, self.ipoin.1),
        ] {
            if !(v.is_on_arc() && vp.is_on_arc() && v.arc()?.is_same(vp.arc()?)) {
                continue;
            }
            if contains(
                dstr.shape_interferences_index(iarc, true)?,
                iarc,
                ipoin,
                false,
                false,
            ) && v.transition_on_arc()? != vp.transition_on_arc()?
            {
                let i = fil_point_in_ds(
                    v.transition_on_arc()?,
                    iarc,
                    ipoin,
                    v.parameter_on_arc()?,
                    v.is_vertex(),
                );
                dstr.change_shape_interferences(v.arc()?.as_ref()).append(i);
            }
        }
        Ok(())
    }

    /// The transitions of the traces on the faces by the orientation of the face (or of
    /// the fillet surface in the solid) and of the fillet.
    fn transitions(&mut self, fd: &SurfDataHandle, dstr: &TopOpeBRepDSDataStructure) {
        let g = lock(fd);
        let (ishape1, ishape2, or_fd) = (g.index_of_s1(), g.index_of_s2(), g.orientation());
        let (t1, t2) = (
            g.interference_on_s1().transition(),
            g.interference_on_s2().transition(),
        );
        drop(g);
        let solid_li = dstr
            .shape_interferences_index(self.solid_index, true)
            .cloned()
            .unwrap_or_default();
        let face_or = |ishape: i32, current: Orientation| -> Orientation {
            if ishape > 0 {
                dstr.shape(ishape, true).orientation()
            } else {
                interference_orientation(&solid_li, self.solid_index, -ishape, false, false)
                    .unwrap_or(current)
            }
        };
        if ishape1 != 0 {
            let t = top_abs::compose(face_or(ishape1, self.trafil.0), or_fd);
            let t = top_abs::compose(top_abs::reverse(t1), t);
            self.trafil = (t, top_abs::reverse(t));
        } else {
            let t = top_abs::compose(face_or(ishape2, self.trafil.1), or_fd);
            let t = top_abs::compose(top_abs::reverse(t2), t);
            self.trafil = (top_abs::reverse(t), t);
        }
    }

    /// The edges of the fillet along edges cut at their end vertices.
    fn cut_edges(
        &self,
        fd: &SurfDataHandle,
        (v1, v2): (&CommonPoint, &CommonPoint),
        dstr: &mut TopOpeBRepDSDataStructure,
    ) -> Result<(), Failure> {
        let (c1, c2) = {
            let g = lock(fd);
            (g.is_on_curve1(), g.is_on_curve2())
        };
        for (v, on_curve, ons) in [
            (v1, c1, 1),
            (v2, c2, 2),
            (&self.v3, c1, 1),
            (&self.v4, c2, 2),
        ] {
            if v.is_vertex() && on_curve {
                cut_edge(v.vertex()?, fd, dstr, ons)?;
            }
        }
        Ok(())
    }

    /// The vertex `bout` of the spine edge `arcspine` added to the edge interferences.
    pub(super) fn spine_vertex_interference(
        &self,
        dstr: &mut TopOpeBRepDSDataStructure,
        arcspine: &Edge,
        ivtx: i32,
        ovtx: Option<Orientation>,
    ) -> Result<(), Failure> {
        let iarc = dstr.add_shape(arcspine.as_ref());
        let mut ovtx_first = Orientation::Forward;
        for v in Explorer::new(
            &arcspine.oriented(Orientation::Forward),
            ShapeEnum::Vertex,
            ShapeEnum::Shape,
        )? {
            if self.bout_de_vtx.is_same(&v) {
                ovtx_first = v.orientation();
                break;
            }
        }
        let ovtx = ovtx.unwrap_or(ovtx_first);
        let par = BRepTool::parameter(&self.bout_de_vtx, arcspine)?;
        dstr.change_shape_interferences_index(iarc)?
            .append(fil_vertex_in_ds(top_abs::reverse(ovtx), iarc, ivtx, par));
        Ok(())
    }

    /// The edge `i` of the spine.
    pub(super) fn spine_edge(&self, i: i32) -> Result<Edge, Failure> {
        lock(
            self.spine
                .as_ref()
                .ok_or_else(|| null("ChFi3d_FilDS: no spine"))?,
        )
        .edges(i)
        .cloned()
    }
}
