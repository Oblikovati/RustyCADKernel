// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_BuilderON
// (.hxx/.cxx) and TopOpeBRepBuild_on.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ON parts of a face (`TopOpeBRepBuild_BuilderON`): the ON splits of the section
//! edges of the face interferences added, oriented, to the wire edge set of the face
//! when they bound its part of the operation.
mod context;
mod on2d;
mod yap0;
mod yap12;
mod yap3;
mod yap6;
mod yapc;

use super::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::export::{fds_data, fds_has_same_domain2d, fds_has_same_domain3d};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSShapeShapeInterference, TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{
    fun_tool_find_pin_e, fun_tool_orient_e_in_f, fun_tool_orient_e_in_f_forward, fun_tool_sta_pin_e,
};
use context::OnContext;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_lprop::{BRepLPropSLProps, BRepLPropSurfaceTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Point2};

/// The ON parts of a face (`TopOpeBRepBuild_BuilderON`): the builder, the face, the
/// operation table, the shapes to classify with and the wire edge set it fills;
/// `TopOpeBRepBuildBuilderON::new().perform(&b, &f, &g, &l, &mut wes)?`.
#[derive(Default)]
pub struct TopOpeBRepBuildBuilderON<'a> {
    my_pb: Option<&'a TopOpeBRepBuildBuilder>,
    my_face: Shape,
    my_pg: Option<TopOpeBRepBuildGTopo>,
    my_plsclass: Option<&'a List<Shape>>,
    my_pwes: Option<&'a mut TopOpeBRepBuildWireEdgeSet>,
}

/// Standard_NullObject of a builder ON not performed.
fn not_performed() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepBuild_BuilderON: not performed",
        "TopOpeBRepBuild_BuilderON",
    )
}

/// `FUN_keepEON(B, sEG, sFOR, sFS, EGBoundFOR, TFE, TB1, TB2)`: the part of the face
/// on the side of the edge given by the transition on the state (all on a closing
/// edge); `fun_keep_eon(&eg, &for_, &tfe, State::In)?`.
pub fn fun_keep_eon(
    eg: &Shape,
    for_: &Shape,
    tfe: &TopOpeBRepDSTransition,
    tb1: State,
) -> Result<bool, Failure> {
    let eg = topods::edge(eg)?;
    let ff = topods::face(&for_.oriented(Orientation::Forward))?;
    if BRepTool::is_closed_on_face(&eg, &ff)? {
        return Ok(true);
    }
    let (_, oegff) = fun_tool_orient_e_in_f(&eg, &ff)?;
    let (tfeb, tfea) = (tfe.before(), tfe.after());
    Ok(match oegff {
        Orientation::Forward => tfea == tb1,
        Orientation::Reversed => tfeb == tb1,
        _ => tfea == tb1 || tfeb == tb1,
    })
}

/// `FUN_build_TB(PB, rank)`: the state of the part kept of the shape of the rank by
/// the operation; `fun_build_tb(&b, 1)`.
pub fn fun_build_tb(pb: &TopOpeBRepBuildBuilder, rank: i32) -> State {
    let mut sta = State::Unknown;
    if pb.opefus() {
        sta = State::Out;
    }
    if pb.opecom() {
        sta = State::In;
    }
    if pb.opec12() {
        sta = if rank == 1 { State::Out } else { State::In };
    }
    if pb.opec21() {
        sta = if rank == 2 { State::Out } else { State::In };
    }
    sta
}

/// `FUN_Kpart0(ST1, ST2)` (static): a plane against a quadric, or a cylinder or cone
/// against a sphere.
fn fun_kpart0(st1: SurfaceType, st2: SurfaceType) -> bool {
    use SurfaceType::*;
    let quad2 = matches!(st2, Cylinder | Cone | Sphere | Torus);
    (st1 == Plane && quad2) || ((st1 == Cylinder || st1 == Cone) && st2 == Sphere)
}

/// `FUN_Kpart(ST1, ST2)` (static): 1 or 2 by the order of a particular pair, else 0.
fn fun_kpart(st1: SurfaceType, st2: SurfaceType) -> i32 {
    if fun_kpart0(st1, st2) {
        1
    } else if fun_kpart0(st2, st1) {
        2
    } else {
        0
    }
}

/// `FUN_findeSD(BDS, EspON, EG, FOR, oeSD, D)` (static): the 3d (D 3) or 2d (D 2) same
/// domain edge of the face containing a point of the ON split, with its orientation
/// in the face (0 when none).
fn fun_find_esd(
    bds: &TopOpeBRepDSDataStructure,
    esp_on: &Edge,
    eg: &Edge,
    for_: &Face,
    d: i32,
) -> Result<(i32, Orientation), Failure> {
    let (pt_on, _) = fun_tool_find_pin_e(esp_on.as_ref())?;
    let mut lesd = List::default();
    if d == 3 {
        fds_has_same_domain3d(bds, eg.as_ref(), Some(&mut lesd));
    }
    if d == 2 {
        fds_has_same_domain2d(bds, eg.as_ref(), Some(&mut lesd));
    }
    for s in lesd.iter() {
        let esd = topods::edge(s)?;
        let (of, oesd) = fun_tool_orient_e_in_f_forward(&esd, for_)?;
        if !of {
            continue;
        }
        if fun_tool_sta_pin_e(pt_on, &esd)? == State::In {
            return Ok((bds.shape_index(s, true), oesd));
        }
    }
    Ok((0, Orientation::External))
}

/// `ComputeFaceCrvtInSec(aFace, aP2d, aSecDir, aCrvt)` (static): the normal curvature of
/// the face along the direction (the umbilic curvature), None for a surface less than
/// C2 or without curvature.
fn compute_face_crvt_in_sec(
    a_face: &Face,
    p2d: Point2,
    sec_dir: Direction3,
) -> Result<Option<f64>, Failure> {
    let surf = BRepAdaptorSurface::with_face(a_face, true)?;
    if BRepLPropSurfaceTool::continuity(&surf)? < 2 {
        return Ok(None);
    }
    let mut prop =
        BRepLPropSLProps::with_parameters(&surf, p2d.x(), p2d.y(), 2, precision::confusion())?;
    if !prop.is_curvature_defined()? {
        return Ok(None);
    }
    if prop.is_umbilic()? {
        return Ok(Some(prop.max_curvature()?));
    }
    let (max_crv, min_crv) = (prop.max_curvature()?, prop.min_curvature()?);
    let (max_dir, min_dir) = prop.curvature_directions()?;
    Ok(Some(
        max_crv * sec_dir.dot(max_dir) + min_crv * sec_dir.dot(min_dir),
    ))
}

impl<'a> TopOpeBRepBuildBuilderON<'a> {
    /// `TopOpeBRepBuild_BuilderON()`; `TopOpeBRepBuildBuilderON::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_BuilderON(PB, FOR, PG, PLSclass, PWES)`: `Perform`;
    /// `TopOpeBRepBuildBuilderON::with_face(&b, &f, &g, &l, &mut wes)?`.
    pub fn with_face(
        pb: &'a TopOpeBRepBuildBuilder,
        for_: &Shape,
        pg: &TopOpeBRepBuildGTopo,
        plsclass: &'a List<Shape>,
        pwes: &'a mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<Self, Failure> {
        let mut bon = Self::new();
        bon.perform(pb, for_, pg, plsclass, pwes)?;
        Ok(bon)
    }

    /// `Perform(PB, FOR, PG, PLSclass, PWES)`: each interference of the face filled;
    /// `bon.perform(&b, &f, &g, &l, &mut wes)?`.
    pub fn perform(
        &mut self,
        pb: &'a TopOpeBRepBuildBuilder,
        for_: &Shape,
        pg: &TopOpeBRepBuildGTopo,
        plsclass: &'a List<Shape>,
        pwes: &'a mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        self.my_pb = Some(pb);
        self.my_face = for_.clone();
        self.my_pg = Some(*pg);
        self.my_plsclass = Some(plsclass);
        self.my_pwes = Some(pwes);
        let li = pb.hds()?.ds().shape_interferences(for_, true).clone();
        for i in li.iter() {
            self.g_fill_on_parts_wes1(i)?;
        }
        Ok(())
    }

    /// The builder of the parts.
    fn pb(&self) -> Result<&'a TopOpeBRepBuildBuilder, Failure> {
        self.my_pb.ok_or_else(not_performed)
    }

    /// An edge added as a start element of the wire edge set.
    fn add_start(&mut self, e: &Shape) -> Result<(), Failure> {
        self.my_pwes
            .as_deref_mut()
            .ok_or_else(not_performed)?
            .add_start_element(e)
    }

    /// `GFillONCheckI(I)`: a shape/shape EDGE/FACE interference of FACE transitions on a
    /// split section edge, between shapes of a rank; `bon.g_fill_on_check_i(&i)?`.
    pub fn g_fill_on_check_i(&self, i: &Handle<TopOpeBRepDSInterference>) -> Result<bool, Failure> {
        let pb = self.pb()?;
        let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
        let Some(s) = ssi.get() else {
            return Ok(false);
        };
        let (gt, gi, st, si) = fds_data(i);
        if gt != TopOpeBRepDSKind::Edge || st != TopOpeBRepDSKind::Face {
            return Ok(false);
        }
        let (eg, fs) = {
            let bds = pb.hds()?.ds();
            (
                topods::edge(bds.shape(gi, false))?,
                bds.shape(si, false).clone(),
            )
        };
        let tfe = s.transition();
        if tfe.shape_after() != ShapeEnum::Face || tfe.shape_before() != ShapeEnum::Face {
            return Ok(false);
        }
        let isrest = pb.hds()?.ds().is_section_edge(&eg, true);
        if !(isrest && pb.is_split(eg.as_ref(), State::On))
            || pb.splits(eg.as_ref(), State::On).is_empty()
        {
            return Ok(false);
        }
        Ok(pb.g_shape_rank(&fs) != 0 && pb.g_shape_rank(&self.my_face) != 0)
    }

    /// `GFillONPartsWES1(I)`: each ON split of the section edge of a checked
    /// interference filled; `bon.g_fill_on_parts_wes1(&i)?`.
    pub fn g_fill_on_parts_wes1(
        &mut self,
        i: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<(), Failure> {
        if !self.g_fill_on_check_i(i)? {
            return Ok(());
        }
        let pb = self.pb()?;
        let (_, gi, _, _) = fds_data(i);
        let eg = pb.hds()?.shape(gi, true);
        for esp_on in pb.splits(&eg, State::On).clone().iter() {
            self.g_fill_on_parts_wes2(i, esp_on)?;
        }
        Ok(())
    }

    /// `GFillONPartsWES2(I, EspON)`: the ON split of the section edge of the interference
    /// added, oriented, when it bounds the part of the face by the configuration of
    /// the face, the edge and its same domain edges (the cases yap00 .. yap7);
    /// `bon.g_fill_on_parts_wes2(&i, &esp)?`.
    pub fn g_fill_on_parts_wes2(
        &mut self,
        i: &Handle<TopOpeBRepDSInterference>,
        esp_on: &Shape,
    ) -> Result<(), Failure> {
        let pb = self.pb()?;
        let pg = self.my_pg.ok_or_else(not_performed)?;
        let Some(c) = OnContext::new(pb, &self.my_face, &pg, i, esp_on)? else {
            return Ok(());
        };
        if c.yap00 {
            return self.yap00(&c);
        }
        if c.yap0 {
            return self.yap0(&c);
        }
        if c.yapc1 {
            return self.yapc1(&c);
        }
        if c.yapc2 {
            return self.yapc2(&c);
        }
        if c.yapc3 {
            return self.yapc3(&c);
        }
        self.after_closed_cases(c)
    }

    /// The cases on the face connected to the edge in the other face (yap1 .. yap7).
    fn after_closed_cases(&mut self, c: OnContext) -> Result<(), Failure> {
        let pb = self.pb()?;
        let Some(cx) = c.fcx_context(pb)? else {
            return Ok(());
        };
        if cx.yap1 {
            return self.yap1(&c, &cx);
        }
        if cx.yap2 {
            return self.yap2(&c, &cx);
        }
        if cx.yap6 {
            return self.yap6(&c, &cx);
        }
        if cx.yap3 {
            return self.yap3(&c);
        }
        if cx.yap5 {
            return self.yap5(&c);
        }
        if cx.yap4 {
            let e = c.esp_on.oriented(c.tfe.orientation(c.tb1, ShapeEnum::Face));
            return self.add_start(&e);
        }
        if cx.yap7 {
            return self.yap7(&c);
        }
        Ok(())
    }
}

impl TopOpeBRepBuildBuilder {
    /// `GFillONPartsWES(FOR, G, LSclass, WES)` (TopOpeBRepBuild_on.cxx): the ON parts of
    /// the face (BuilderON, Perform2d in the 2d case) in the wire edge set;
    /// `b.g_fill_on_parts_wes(&f, &g, &l, &mut wes)?`.
    pub fn g_fill_on_parts_wes(
        &mut self,
        for_: &Shape,
        g: &TopOpeBRepBuildGTopo,
        lsclass: &List<Shape>,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let mut bon = TopOpeBRepBuildBuilderON::new();
        if crate::top_ope_brep_build::globals::GLOBAL_FACES2D.get() {
            bon.perform2d(self, for_, g, lsclass, wes)
        } else {
            bon.perform(self, for_, g, lsclass, wes)
        }
    }
}
