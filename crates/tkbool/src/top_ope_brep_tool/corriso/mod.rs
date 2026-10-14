// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_CORRISO.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_CORRISO`: the UV representations of the edges of a shape on a
//! closed reference face, the detection of edges whose UV ends are not connected and
//! the translation of their p-curves by a period.
mod bounds;
mod faulty;

use crate::top_ope_brep_tool::c2df::TopOpeBRepToolC2DF;
use crate::top_ope_brep_tool::tool::TopOpeBRepToolTOOL;
use crate::top_ope_brep_tool::topology::fun_tool_closed_s;
use crate::top_ope_brep_tool::two_d::{
    fc2d_add_new_curve_on_surface, fc2d_editable_curve_on_surface, fc2d_has_old_curve_on_surface,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::Vector2;

/// A map of edges to integer flags (`TopTools_DataMapOfOrientedShapeInteger`).
pub type EdgeFlagMap = DataMap<Shape, i32>;

/// The `EdgeOUTofBoundsUV` codes: the edge crosses a bound and must be split.
pub const CORRISO_SPLITEDGE: i32 = 0;
/// The edge must be translated by a period forwards.
pub const CORRISO_INCREASE: i32 = 1;
/// The edge must be translated by a period backwards.
pub const CORRISO_DECREASE: i32 = -1;

/// The UV corrector of the edges of a shape on a reference face;
/// `let mut c = TopOpeBRepToolCORRISO::with_face(&f)?; c.init(&w)?;`.
#[derive(Default)]
pub struct TopOpeBRepToolCORRISO {
    my_fref: Face,
    my_gas: GeomAdaptorSurface,
    my_uclosed: bool,
    my_vclosed: bool,
    my_uper: f64,
    my_vper: f64,
    my_s: Shape,
    my_eds: List<Shape>,
    my_e_rep2d: DataMap<Shape, TopOpeBRepToolC2DF>,
    my_v_eds: DataMap<Shape, List<Shape>, ShapeMapHasher>,
}

impl TopOpeBRepToolCORRISO {
    /// `TopOpeBRepTool_CORRISO()`; `TopOpeBRepToolCORRISO::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepTool_CORRISO(Fref)`: the reference face with the closures of its basis
    /// surface and the adaptor of its surface; `TopOpeBRepToolCORRISO::with_face(&f)?`.
    pub fn with_face(fref: &Face) -> Result<Self, Failure> {
        let (uclosed, uper, vclosed, vper) = fun_tool_closed_s(fref)?.unwrap_or_default();
        let su = BRepTool::surface(fref)?;
        Ok(Self {
            my_fref: fref.clone(),
            my_gas: GeomAdaptorSurface::with_surface(&su)?,
            my_uclosed: uclosed,
            my_vclosed: vclosed,
            my_uper: uper,
            my_vper: vper,
            ..Self::default()
        })
    }

    /// `Fref()`; `c.fref()`.
    pub fn fref(&self) -> &Face {
        &self.my_fref
    }

    /// `GASref()`; `c.gas_ref()`.
    pub fn gas_ref(&self) -> &GeomAdaptorSurface {
        &self.my_gas
    }

    /// `Refclosed(x, xperiod)`: whether the reference face is closed in U (`x` 1) or V
    /// (2) with its period; `None` for another `x`; `c.refclosed(1)`.
    pub fn refclosed(&self, x: i32) -> Option<(bool, f64)> {
        match x {
            1 => Some((self.my_uclosed, self.my_uper)),
            2 => Some((self.my_vclosed, self.my_vper)),
            _ => None,
        }
    }

    /// The p-curve of the edge on the reference face (kept as new when not stored);
    /// `None` without p-curve.
    fn edge_c2df(&self, e: &Edge) -> Result<Option<TopOpeBRepToolC2DF>, Failure> {
        let hasold = fc2d_has_old_curve_on_surface(e, &self.my_fref)?.is_some();
        let (pc, f, l, tol) = fc2d_editable_curve_on_surface(e, &self.my_fref, false)?;
        if !hasold {
            fc2d_add_new_curve_on_surface(&pc, e, &self.my_fref, f, l, tol);
        }
        if pc.is_null() {
            return Ok(None);
        }
        Ok(Some(TopOpeBRepToolC2DF::new(&pc, f, l, tol, &self.my_fref)))
    }

    /// The edge appended to the connexity of each of its vertices.
    fn connect_vertices(&mut self, e: &Shape) -> Result<(), Failure> {
        for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            match self.my_v_eds.change_seek(&v) {
                Some(loe) => {
                    loe.append(e.clone());
                }
                None => {
                    let mut loe: List<Shape> = List::default();
                    loe.append(e.clone());
                    self.my_v_eds.bind(v, loe);
                }
            }
        }
        Ok(())
    }

    /// `Init(S)`: the edges of the shape with their UV representations and the edges
    /// at each vertex; false for a null shape or an edge without p-curve;
    /// `c.init(&w)?`.
    pub fn init(&mut self, s: &Shape) -> Result<bool, Failure> {
        self.my_e_rep2d.clear(false);
        self.my_eds.clear();
        self.my_v_eds.clear(false);
        if s.is_null() {
            return Ok(false);
        }
        self.my_s = s.clone();
        for es in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let e = topods::edge(&es)?;
            self.my_eds.append(es.clone());
            let Some(c2df) = self.edge_c2df(&e)? else {
                return Ok(false);
            };
            self.my_e_rep2d.bind(es.clone(), c2df);
            self.connect_vertices(&es)?;
        }
        Ok(true)
    }

    /// `S()`; `c.s()`.
    pub fn s(&self) -> &Shape {
        &self.my_s
    }

    /// `Eds()`; `c.eds()`.
    pub fn eds(&self) -> &List<Shape> {
        &self.my_eds
    }

    /// `UVRep(E, C2DF)`: a copy of the UV representation of the edge;
    /// `c.uv_rep(&e)`.
    pub fn uv_rep(&self, e: &Shape) -> Option<TopOpeBRepToolC2DF> {
        self.my_e_rep2d.seek(e).cloned()
    }

    /// `SetUVRep(E, C2DF)`: the UV representation of a known edge replaced;
    /// `c.set_uv_rep(&e, &c2df)`.
    pub fn set_uv_rep(&mut self, e: &Shape, c2df: &TopOpeBRepToolC2DF) -> bool {
        match self.my_e_rep2d.change_seek(e) {
            Some(rep) => {
                *rep = c2df.clone();
                true
            }
            None => false,
        }
    }

    /// `Connexity(V, Eds)`: a copy of the edges at the vertex; `c.connexity(&v)`.
    pub fn connexity(&self, v: &Shape) -> Option<List<Shape>> {
        self.my_v_eds.seek(v).cloned()
    }

    /// `SetConnexity(V, Eds)`: the edges of a known vertex replaced;
    /// `c.set_connexity(&v, &l)`.
    pub fn set_connexity(&mut self, v: &Shape, eds: &List<Shape>) -> bool {
        match self.my_v_eds.change_seek(v) {
            Some(loe) => {
                *loe = eds.clone();
                true
            }
            None => false,
        }
    }

    /// `UVClosed()`: whether no edge has an unconnected UV end; `c.uv_closed()?`.
    pub fn uv_closed(&self) -> Result<bool, Failure> {
        let mut lfy_e = EdgeFlagMap::default();
        Ok(!self.edges_with_faulty_uv(&self.my_eds, 3, &mut lfy_e, true)?)
    }

    /// `Tol(I, tol3d)`: the U (`I` 1) or V resolution of a 3d tolerance on the
    /// reference surface; `c.tol(1, 1e-7)?`.
    pub fn tol(&self, i: i32, tol3d: f64) -> Result<f64, Failure> {
        if i == 1 {
            self.my_gas.u_resolution(tol3d)
        } else {
            self.my_gas.v_resolution(tol3d)
        }
    }

    /// The greater of the U and V resolutions of a 3d tolerance.
    pub(super) fn tol_uv(&self, tol3d: f64) -> Result<f64, Failure> {
        Ok(std_max(self.tol(1, tol3d)?, self.tol(2, tol3d)?))
    }

    /// `TrslUV(onU, FyEds)`: the UV representations of the edges translated by a period
    /// (forwards for INCREASE, backwards for DECREASE); false for a direction without
    /// period, an unknown edge or another code; `c.trsl_uv(true, &fy)?`.
    pub fn trsl_uv(&mut self, on_u: bool, fy_eds: &EdgeFlagMap) -> Result<bool, Failure> {
        let (_, per) = self.refclosed(if on_u { 1 } else { 2 }).unwrap_or_default();
        if per == 0. {
            return Ok(false);
        }
        let tt2d = if on_u {
            Vector2::new(per, 0.)
        } else {
            Vector2::new(0., per)
        };
        for (e, &itt) in fy_eds.items() {
            let Some(mut c2df) = self.uv_rep(e) else {
                return Ok(false);
            };
            match itt {
                CORRISO_INCREASE => TopOpeBRepToolTOOL::trsl_uv(tt2d, &mut c2df)?,
                CORRISO_DECREASE => TopOpeBRepToolTOOL::trsl_uv(tt2d * -1., &mut c2df)?,
                _ => return Ok(false),
            }
            self.set_uv_rep(e, &c2df);
        }
        Ok(true)
    }

    /// A trimmed p-curve of a UV representation.
    fn trimmed(c2df: &TopOpeBRepToolC2DF) -> Result<(Handle<dyn Geom2dCurve>, f64), Failure> {
        let (pc, f, l, tol) = c2df.pc();
        let cu: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dTrimmedCurve::new(&pc, f, l, true, true)?).cast();
        Ok((cu, tol))
    }

    /// `GetnewS(newS)`: the face of the shape with the UV representations stored on its
    /// edges (both p-curves on a closing edge, from its FORWARD occurrence); `None` when
    /// the shape is not a face or an edge has no representation; `c.getnew_s()?`.
    pub fn getnew_s(&self) -> Result<Option<Face>, Failure> {
        if self.my_s.shape_type()? != ShapeEnum::Face {
            return Ok(None);
        }
        let new_s = topods::face(&self.my_s)?;
        let bb = BRepBuilder::default();
        for es in self.my_eds.iter() {
            let e = topods::edge(es)?;
            let ori_e = e.orientation();
            let Some(c2df) = self.uv_rep(es) else {
                return Ok(None);
            };
            let (cu, tol) = Self::trimmed(&c2df)?;
            let err = e.oriented(top_abs::complement(ori_e));
            match self.uv_rep(&err) {
                Some(c2dfrr) => {
                    let (curr, _) = Self::trimmed(&c2dfrr)?;
                    if ori_e == Orientation::Forward {
                        bb.update_edge_pcurves_on_face(&e, (&cu, &curr), &new_s, tol)?;
                    }
                }
                None => bb.update_edge_pcurve_on_face(&e, &cu, &new_s, tol)?,
            }
        }
        Ok(Some(new_s))
    }

    /// `AddNewConnexity(V, E)`: the edge (with its UV representation when new) added to
    /// the edges and to the connexity of its vertices; false without p-curve;
    /// `c.add_new_connexity(&v, &e)?`.
    pub fn add_new_connexity(&mut self, _v: &Vertex, e: &Edge) -> Result<bool, Failure> {
        let isb = self.my_e_rep2d.is_bound(e);
        if !isb {
            let Some(c2df) = self.edge_c2df(e)? else {
                return Ok(false);
            };
            self.my_e_rep2d.bind(e.clone().into_shape(), c2df);
            self.my_eds.append(e.clone().into_shape());
        }
        self.connect_vertices(e)?;
        Ok(true)
    }

    /// `RemoveOldConnexity(V, E)`: the edge removed from the representations, the edges
    /// and the connexity of its vertices; false when a vertex is unknown or the edge was
    /// in no connexity; `c.remove_old_connexity(&v, &e)?`.
    pub fn remove_old_connexity(&mut self, _v: &Vertex, e: &Edge) -> Result<bool, Failure> {
        if self.my_e_rep2d.is_bound(e) {
            self.my_e_rep2d.unbind(e);
            remove_first_equal(&mut self.my_eds, e);
        }
        let mut done = false;
        for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let Some(loe) = self.my_v_eds.change_seek(&v) else {
                return Ok(false);
            };
            if remove_first_equal(loe, e) {
                done = true;
            }
        }
        Ok(done)
    }
}

/// The first shape of the list equal to `s` removed; whether there was one.
pub(crate) fn remove_first_equal(l: &mut List<Shape>, s: &Shape) -> bool {
    let Some(pos) = l.iter().position(|x| x.is_equal(s)) else {
        return false;
    };
    let mut index = 0;
    l.retain(|_| {
        index += 1;
        index != pos + 1
    });
    true
}
