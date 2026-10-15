// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_mkTondgE.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_mkTondgE`: the transitions on a degenerated edge (the apex of a
//! cone or the pole of a sphere) crossed by a face or by its edges, as the angles
//! around the closing edge where the transition is OUT/IN and IN/OUT.
use crate::top_ope_brep_tool::normal::nggeom_f;
use crate::top_ope_brep_tool::project::fun_tool_bounds;
use crate::top_ope_brep_tool::tool::{TOOL_FORWARD, TopOpeBRepToolTOOL};
use crate::top_ope_brep_tool::tool_iso::uviso_edge;
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use crate::top_ope_brep_tool::topology::fun_tool_paron_ef;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Vector3};
use std::f64::consts::PI;

/// No transition.
pub const MKT_NOI: i32 = 0;
/// The transition at the first parameter.
pub const MKT_MKI1: i32 = 1;
/// The transition at the second parameter.
pub const MKT_MKI2: i32 = 2;
/// The transitions at both parameters.
pub const MKT_MKI12: i32 = 3;

/// The transitions of a crossing on a degenerated edge: `(mkT, par1, par2)`.
pub type DgTransition = (i32, f64, f64);

/// `FUN_tola()` (static): Precision::Angular().
fn fun_tola() -> f64 {
    precision::angular()
}

/// The transition computer on a degenerated edge;
/// `let mut mk = TopOpeBRepToolMkTondgE::new(); mk.initialize(&dge, &f, uvi, &fi)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepToolMkTondgE {
    my_dg_e: Edge,
    my_f: Face,
    my_cl_e: Edge,
    my_fi: Face,
    myuvi: Point2,
    is_t2d: bool,
    my_epari: DataMap<Shape, f64, ShapeMapHasher>,
    has_rest: bool,
    myngf: Direction3,
    myngfi: Direction3,
}

/// The first vertex of an edge (Standard_NoSuchObject without vertex).
fn first_vertex(e: &Edge) -> Result<Vertex, Failure> {
    let exv = Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    let v = exv.more().then(|| exv.current().clone()).ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "TopOpeBRepTool_mkTondgE: the degenerated edge has no vertex; expected a vertex",
            "TopOpeBRepTool_mkTondgE",
        )
    })?;
    topods::vertex(&v)
}

/// The geometric normal of a face at UV as a direction; natively the direction of the
/// zero vector is made before the success test (Standard_ConstructionError).
fn face_normal(uv: Point2, f: &Face) -> Result<(bool, Direction3), Failure> {
    let mut tmp = Vector3::default();
    let ok = nggeom_f(uv, f, &mut tmp)?;
    Ok((ok, Direction3::from_xyz(*tmp.xyz())?))
}

/// `FUN_getEc(f, v, cle)` (static): the first closing edge of the face bounded by the
/// vertex.
fn fun_get_ec(f: &Face, v: &Vertex) -> Result<Option<Edge>, Failure> {
    for es in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&es)?;
        if !TopOpeBRepToolTOOL::is_closing_e(&e, f)? {
            continue;
        }
        let mut exv = Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        if exv.any(|c| c.is_same(v)) {
            return Ok(Some(e));
        }
    }
    Ok(None)
}

/// `FUN_MkTonE(faxis, dirINcle, xxi, ngf, par1, par2, outin)` (static): the angles
/// around `faxis` from the closing edge direction where the transition is OUT/IN
/// (`par1`) and IN/OUT (`par2`), and whether the first angle is OUT/IN;
/// `(par1, par2, outin)`.
fn fun_mk_ton_e(
    faxis: Vector3,
    dir_incle: Vector3,
    xxi: Vector3,
) -> Result<(f64, f64, bool), Failure> {
    let tgi = xxi.crossed(faxis);
    let tola = fun_tola();
    let dot1 = dir_incle.dot(xxi);
    let is_oni = dot1.abs() < tola;
    let (ang, outin) = if is_oni {
        (if dir_incle.dot(tgi) > 0. { 0. } else { PI }, true)
    } else {
        let ang = TopOpeBRepToolTOOL::matter_vec(dir_incle, tgi.reversed(), faxis)?;
        let dot = dir_incle.crossed(tgi).dot(faxis);
        (ang, if dot1 < 0. { dot > 0. } else { dot < 0. })
    };
    let cang = if ang > PI { ang - PI } else { ang + PI };
    Ok(if outin {
        (ang, cang, outin)
    } else {
        (cang, ang, outin)
    })
}

/// `xxi.Crossed(faxis)` of the members: the vector `faxis` taken as a direction, so
/// the result is a unit direction (Standard_ConstructionError for parallel ones).
fn dir_tangent(xxi: Direction3, faxis: Vector3) -> Result<Vector3, Failure> {
    let f = Direction3::from_xyz(*faxis.xyz())?;
    Ok(Vector3::from(xxi.crossed(f)?))
}

/// The transition code from the side of the restriction and the OUT/IN order.
fn restricted_code(dot: f64, outin: bool) -> i32 {
    let keepang = dot > 0.;
    match (keepang, outin) {
        (true, true) | (false, false) => MKT_MKI1,
        _ => MKT_MKI2,
    }
}

impl TopOpeBRepToolMkTondgE {
    /// `TopOpeBRepTool_mkTondgE()`; `TopOpeBRepToolMkTondgE::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Initialize(dgE, F, uvi, Fi)`: the degenerated edge of `F` and the crossing face
    /// `Fi` at `uvi`; the transition is 2d when both faces have parallel normals;
    /// `mk.initialize(&dge, &f, uvi, &fi)?`.
    pub fn initialize(
        &mut self,
        dg_e: &Edge,
        f: &Face,
        uvi: Point2,
        fi: &Face,
    ) -> Result<bool, Failure> {
        (self.is_t2d, self.has_rest) = (false, false);
        self.my_cl_e = Edge::default();
        self.my_epari.clear(false);
        (self.my_dg_e, self.my_f) = (dg_e.clone(), f.clone());
        let v = first_vertex(&self.my_dg_e)?;
        let par = BRepTool::parameter(&v, &self.my_dg_e)?;
        let mut uv = Point2::default();
        if !fun_tool_paron_ef(&self.my_dg_e, par, &self.my_f, &mut uv)? {
            return Ok(false);
        }
        let (ok, ngf) = face_normal(uv, &self.my_f)?;
        self.myngf = ngf;
        if !ok {
            return Ok(false);
        }
        (self.myuvi, self.my_fi) = (uvi, fi.clone());
        let (oki, ngfi) = face_normal(self.myuvi, &self.my_fi)?;
        self.myngfi = ngfi;
        if !oki {
            return Ok(false);
        }
        let dot = self.myngf.dot(self.myngfi);
        self.is_t2d = (1. - dot.abs()).abs() < fun_tola();
        Ok(true)
    }

    /// `SetclE(clE)`; `mk.setcl_e(&cle)`.
    pub fn setcl_e(&mut self, cl_e: &Edge) -> bool {
        self.my_cl_e = cl_e.clone();
        true
    }

    /// `IsT2d()`; `mk.is_t2d()`.
    pub fn is_t2d(&self) -> bool {
        self.is_t2d
    }

    /// `SetRest(pari, Ei)`: the restriction of the crossing to the edge of `Fi` at its
    /// parameter; false (and no restriction) for a closing edge;
    /// `mk.set_rest(0.5, &ei)?`.
    pub fn set_rest(&mut self, pari: f64, ei: &Edge) -> Result<bool, Failure> {
        self.has_rest = true;
        if TopOpeBRepToolTOOL::is_closing_e(ei, &self.my_fi)? {
            self.has_rest = false;
            return Ok(false);
        }
        self.my_epari.bind(ei.clone().into_shape(), pari);
        Ok(true)
    }

    /// The parameter of `myuvi` on a non-closing iso edge of `Fi` through it.
    fn iso_through_uvi(&self, ei: &Edge, (tolu, tolv): (f64, f64)) -> Result<Option<f64>, Failure> {
        let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
        let (iso, isou, isov) = uviso_edge(ei, &self.my_fi, &mut d2d, &mut o2d)?;
        if !iso {
            return Ok(None);
        }
        let mut ok = false;
        if isou {
            ok = (o2d.x() - self.myuvi.x()).abs() < tolu;
        }
        if isov {
            ok = (o2d.y() - self.myuvi.y()).abs() < tolv;
        }
        if !ok {
            return Ok(None);
        }
        let mut parei: f64 = 0.;
        TopOpeBRepToolTOOL::par_iso(self.myuvi, ei, &self.my_fi, &mut parei)?;
        Ok(Some(parei))
    }

    /// `GetAllRest(lEi)`: the non-closing edges of `Fi` restricting the crossing (set
    /// ones and iso edges through `uvi`, their parameters stored) and their count;
    /// `let (n, lei) = mk.get_all_rest()?;`.
    pub fn get_all_rest(&mut self) -> Result<(i32, List<Shape>), Failure> {
        let mut l_ei: List<Shape> = List::default();
        let bs = BRepAdaptorSurface::with_face(&self.my_fi, true)?;
        let tol3d = bs.tolerance()?;
        let tols = (bs.u_resolution(tol3d)?, bs.v_resolution(tol3d)?);
        for es in Explorer::new(&self.my_fi, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let ei = topods::edge(&es)?;
            if TopOpeBRepToolTOOL::is_closing_e(&ei, &self.my_fi)? {
                continue;
            }
            if self.my_epari.is_bound(&es) {
                l_ei.append(es);
                continue;
            }
            if let Some(parei) = self.iso_through_uvi(&ei, tols)? {
                self.my_epari.bind(es.clone(), parei);
                l_ei.append(es);
            }
        }
        Ok((l_ei.extent(), l_ei))
    }

    /// The vertex of the degenerated edge and the direction inside the closing edge of
    /// `F` at it with the orientation code of the vertex; `None` without closing edge
    /// or tangent.
    fn closing_direction(&mut self) -> Result<Option<(Vector3, i32)>, Failure> {
        let v = first_vertex(&self.my_dg_e)?;
        if self.my_cl_e.is_null() {
            match fun_get_ec(&self.my_f, &v)? {
                Some(cle) => self.my_cl_e = cle,
                None => return Ok(None),
            }
        }
        {
            let mut ovcle = 0;
            let tg = TopOpeBRepToolTOOL::tg_inside(&v, &self.my_cl_e, &mut ovcle)?;
            Ok(tg.map(|t| (t, ovcle)))
        }
    }

    /// The axis of `F` for the parametrization of the degenerated edge.
    fn faxis(&self, ovcle: i32) -> Vector3 {
        let faxis = Vector3::from(self.myngf);
        if ovcle == TOOL_FORWARD {
            faxis.reversed()
        } else {
            faxis
        }
    }

    /// The normal of `Fi` at `uvi` oriented inside its matter.
    fn inner_normal_fi(&self) -> Result<Option<Vector3>, Failure> {
        let mut xxi = Vector3::default();
        if !nggeom_f(self.myuvi, &self.my_fi, &mut xxi)? {
            return Ok(None);
        }
        if self.my_fi.orientation() == Orientation::Forward {
            xxi.reverse();
        }
        Ok(Some(xxi))
    }

    /// `MkTonE(mkT, par1, par2)`: for a 3d crossing by `Fi`, the transitions at both
    /// angles (MKI12); false when they cannot be computed. As natively, `out` is left
    /// untouched for a 2d crossing and reset to (NOI, 1e7, 1e7) otherwise before the
    /// computation; `let ok = mk.mk_ton_e(&mut out)?;`.
    pub fn mk_ton_e(&mut self, out: &mut DgTransition) -> Result<bool, Failure> {
        if self.is_t2d {
            return Ok(false);
        }
        *out = (MKT_NOI, 1.0e7, 1.0e7);
        let Some((dir_incle, ovcle)) = self.closing_direction()? else {
            return Ok(false);
        };
        let faxis = self.faxis(ovcle);
        let Some(xxi) = self.inner_normal_fi()? else {
            return Ok(false);
        };
        let (par1, par2, _) = fun_mk_ton_e(faxis, dir_incle, xxi)?;
        *out = (MKT_MKI12, par1, par2);
        Ok(true)
    }

    /// The 2d crossing of `MkTonE(Ei, ...)`.
    fn mk_ton_e_2d(
        &self,
        (ei, pari): (&Edge, f64),
        (faxis, dir_incle): (Vector3, Vector3),
        (onfi, onli, closedi, tgin1di): (bool, bool, bool, Vector3),
        out: &mut DgTransition,
    ) -> Result<bool, Failure> {
        let Some(xxi) = TopOpeBRepToolTOOL::xx(self.myuvi, &self.my_fi, pari, ei)? else {
            return Ok(false);
        };
        let (par1, par2, outin) = fun_mk_ton_e(faxis, dir_incle, Vector3::from(xxi))?;
        (out.1, out.2) = (par1, par2);
        if (!onfi && !onli) || closedi {
            out.0 = MKT_MKI12;
            return Ok(true);
        }
        let xxri = Direction3::from_xyz(*tgin1di.xyz())?;
        let dot = dir_tangent(xxi, faxis)?.dot(Vector3::from(xxri));
        out.0 = restricted_code(dot, outin);
        Ok(true)
    }

    /// The 3d crossing of `MkTonE(Ei, ...)` (BUC61053, jgv 21.11.01); the angles are
    /// written before the restriction vector, which may fail.
    fn mk_ton_e_3d(
        &self,
        (ei, pari): (&Edge, f64),
        (faxis, dir_incle): (Vector3, Vector3),
        (onfi, onli, closedi, tgin1di): (bool, bool, bool, Vector3),
        out: &mut DgTransition,
    ) -> Result<bool, Failure> {
        let (ok, xxi_dir) = face_normal(self.myuvi, &self.my_fi)?;
        if !ok {
            return Ok(false);
        }
        let mut xxi = xxi_dir;
        if self.my_fi.orientation() == Orientation::Forward {
            xxi.reverse();
        }
        let (par1, par2, outin) = fun_mk_ton_e(faxis, dir_incle, Vector3::from(xxi))?;
        (out.1, out.2) = (par1, par2);
        let Some(xxri) = TopOpeBRepToolTOOL::xx(self.myuvi, &self.my_fi, pari, ei)? else {
            return Ok(false);
        };
        out.0 = MKT_MKI12;
        let tgi = dir_tangent(xxi, faxis)?;
        let mut dot = tgi.dot(Vector3::from(xxri));
        if dot.abs() < fun_tola() {
            if (!onfi && !onli) || closedi {
                return Ok(true);
            }
            dot = tgi.dot(tgin1di);
        }
        out.0 = restricted_code(dot, outin);
        Ok(true)
    }

    /// `MkTonE(Ei, mkT, par1, par2)`: the transitions of the crossing restricted to the
    /// edge of `Fi` (set by `SetRest` or `GetAllRest`): both angles (MKI12) when the
    /// restriction does not limit them, else the angle where the edge lies (MKI1 or
    /// MKI2); false without restriction or when they cannot be computed. `out` is reset
    /// to (NOI, 1e7, 1e7) first and keeps the values written natively before a failure;
    /// `let ok = mk.mk_ton_e_on(&ei, &mut out)?;`.
    pub fn mk_ton_e_on(&mut self, ei: &Edge, out: &mut DgTransition) -> Result<bool, Failure> {
        *out = (MKT_NOI, 1.0e7, 1.0e7);
        self.has_rest = self.my_epari.is_bound(ei);
        let Some(&pari) = self.my_epari.seek(ei) else {
            return Ok(false);
        };
        let (pfi, pli) = fun_tool_bounds(ei)?;
        let tolpi = TopOpeBRepToolTOOL::tol_p(ei, &self.my_fi)?;
        let (onfi, onli) = ((pari - pfi).abs() < tolpi, (pari - pli).abs() < tolpi);
        let Some(mut tgin1di) = tggeom_e(pari, ei)? else {
            return Ok(false);
        };
        if onli {
            tgin1di.reverse();
        }
        let Some((dir_incle, ovcle)) = self.closing_direction()? else {
            return Ok(false);
        };
        let faxis = self.faxis(ovcle);
        let closedi = TopOpeBRepToolTOOL::closed_e(ei)?.is_some();
        let rest = (onfi, onli, closedi, tgin1di);
        if self.is_t2d {
            return self.mk_ton_e_2d((ei, pari), (faxis, dir_incle), rest, out);
        }
        self.mk_ton_e_3d((ei, pari), (faxis, dir_incle), rest, out)
    }
}
