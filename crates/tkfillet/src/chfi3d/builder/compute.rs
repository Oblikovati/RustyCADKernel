// Rust translation of ChFi3d_Builder::Compute, ExtentAnalyse, PerformFilletOnVertex,
// PerformSingularCorner and the file static CompleteDS (ChFi3d_Builder.cxx) and of
// ChFi3d_Builder::UpdateTolesp (ChFi3d_Builder_C2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The computation of all the fillets: the contours, the corners at their vertices, the
//! data structure and the rebuilt solid.
use super::{ChFi3dBuilder, ChFi3dBuilderOps};
use crate::chfi_ds::{ErrorStatus, Map, SpineHandle, State, StripeHandle, lock};
use crate::chfi3d::{fil_ds, number_of_sharp_edges, stripe_edge_inter};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurveExplorer, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSDataStructure,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSKind, TopOpeBRepDSPointIterator,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::Map as IntMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State as TopState};
use rustycad_tkshhealing::shape_fix::ShapeFix;
use rustycad_tktopalgo::brep_lib::same_parameter_shape;

/// The spine of a contour (Standard_NullObject without one).
pub(crate) fn spine_of(st: &StripeHandle) -> Result<SpineHandle, Failure> {
    lock(st).spine().cloned().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi3d_Builder: contour without spine",
        )
    })
}

/// The ancestors of the shapes of `s` of type `kind` having geometry, added to the data
/// structure.
pub(super) fn add_ancestors_with_geometry(
    dstr: &mut TopOpeBRepDSDataStructure,
    s: &Shape,
    kind: ShapeEnum,
    ancestor: ShapeEnum,
) -> Result<(), Failure> {
    let mut map = Map::new();
    map.fill(s, kind, ancestor)?;
    let mut ex = Explorer::new(s, kind, ShapeEnum::Shape)?;
    while ex.more() {
        let cur = ex.current().clone();
        if dstr.has_geometry(&cur) {
            for anc in map.find_from_key(&cur)?.iter() {
                dstr.add_shape(anc);
            }
        }
        ex.next()?;
    }
    Ok(())
}

/// `CompleteDS(DStr, S)`: the wires and shells of the shapes with geometry added to the
/// data structure, and the range of each curve set to its points.
pub(super) fn complete_ds(dstr: &mut TopOpeBRepDSDataStructure, s: &Shape) -> Result<(), Failure> {
    add_ancestors_with_geometry(dstr, s, ShapeEnum::Edge, ShapeEnum::Wire)?;
    add_ancestors_with_geometry(dstr, s, ShapeEnum::Face, ShapeEnum::Shell)?;
    for ic in 1..=dstr.nb_curves() {
        let mut it = TopOpeBRepDSPointIterator::new(dstr.curve_interferences(ic))?;
        if !it.more() {
            continue;
        }
        let mut parmin = it.parameter()?;
        let mut parmax = parmin;
        it.next()?;
        while it.more() {
            let par = it.parameter()?;
            parmin = parmin.min(par);
            parmax = parmax.max(par);
            it.next()?;
        }
        dstr.change_curve(ic).set_range(parmin, parmax);
    }
    Ok(())
}

/// The tolerance of a curve of the data structure made at least that of its vertices
/// and points (raised for the vertices of a tolerance above 1e-4), the tolerance of the
/// vertices and points raised to that of the curve.
pub(super) fn harmonize_curve_tolerance(
    dstr: &mut TopOpeBRepDSDataStructure,
    ic: i32,
) -> Result<(), Failure> {
    let degen = dstr.curve(ic).curve().is_null();
    let mut tolc = if degen {
        0.
    } else {
        dstr.curve(ic).tolerance()
    };
    let list = dstr.curve_interferences(ic).clone();
    let mut it = TopOpeBRepDSPointIterator::new(&list)?;
    let b1 = BRepBuilder::default();
    while it.more() {
        let ii = it.value()?.clone();
        it.next()?;
        let cpi: Handle<TopOpeBRepDSCurvePointInterference> = ii.cast();
        if cpi.is_null() {
            continue;
        }
        let i = ii
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "CurvePoints"))?;
        let (gk, gi) = (i.geometry_type(), i.geometry());
        if gk == TopOpeBRepDSKind::Vertex {
            let v = topods::vertex(dstr.shape(gi, true))?;
            let mut tolv = BRepTool::vertex_tolerance(&v)?;
            if tolv > 0.0001 {
                tolv += 0.0003;
                if tolc < tolv {
                    tolc = tolv + 0.00001;
                }
            }
            if degen && tolc < tolv {
                tolc = tolv;
            } else if tolc > tolv {
                b1.update_vertex_tolerance(&v, tolc)?;
            }
        } else if gk == TopOpeBRepDSKind::Point {
            let p = dstr.change_point(gi)?;
            let tolp = p.tolerance();
            if degen && tolc < tolp {
                tolc = tolp;
            } else if tolc > tolp {
                p.set_tolerance(tolc);
            }
        }
    }
    if degen {
        dstr.change_curve(ic).set_tolerance(tolc);
    }
    Ok(())
}

impl ChFi3dBuilder {
    /// `UpdateTolesp()`: the 3d tolerance lowered to the smallest tolerance of the
    /// spines; `b.update_tolesp()?`.
    pub fn update_tolesp(&mut self) -> Result<(), Failure> {
        for st in &self.my_list_stripe {
            let current = lock(&spine_of(st)?).get_tolesp();
            if self.tolesp > current {
                self.tolesp = current;
            }
        }
        Ok(())
    }

    /// The solids and the shells outside solids of the shape added to the data
    /// structure: their indices, in a native integer map.
    pub(super) fn solids_and_shells(&mut self) -> Result<IntMap<i32>, Failure> {
        let mut map = IntMap::default();
        let mut expso = Explorer::new(&self.my_shape, ShapeEnum::Solid, ShapeEnum::Shape)?;
        while expso.more() {
            map.add(self.ds.add_shape(expso.current()));
            expso.next()?;
        }
        let mut expsh = Explorer::new(&self.my_shape, ShapeEnum::Shell, ShapeEnum::Solid)?;
        while expsh.more() {
            map.add(self.ds.add_shape(expsh.current()));
            expsh.next()?;
        }
        Ok(map)
    }

    /// The tolerances of the curves and their points, then the data structure moved into
    /// its handle and the solid rebuilt: the solids merged, the tolerances of the vertices
    /// of the split edges raised to those of the edges.
    pub(super) fn rebuild(&mut self, map_ind_so: &IntMap<i32>) -> Result<(), Failure> {
        complete_ds(&mut self.ds, &self.my_shape)?;
        let mut indices = Vec::new();
        let mut cex = TopOpeBRepDSCurveExplorer::new(&self.ds, true);
        while cex.more() {
            indices.push(cex.index());
            cex.next();
        }
        for ic in indices {
            harmonize_curve_tolerance(&mut self.ds, ic)?;
        }
        self.move_ds();
        self.my_coup.perform(&self.my_ds)?;
        for indsol in map_ind_so.iter() {
            let curshape = self.my_ds.shape(*indsol, true);
            self.my_coup.merge_solid(&curshape, TopState::In)?;
        }
        let b1 = BRepBuilder::default();
        for i in 1..=self.my_ds.nb_shapes() {
            let s = self.my_ds.shape(i, true);
            if s.shape_type()? != ShapeEnum::Edge || !self.my_coup.is_split(&s, TopState::In) {
                continue;
            }
            for new_e in self.my_coup.splits(&s, TopState::In).iter() {
                let new_e = topods::edge(new_e)?;
                let tole = BRepTool::edge_tolerance(&new_e)?;
                let mut exv = Explorer::new(new_e.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
                while exv.more() {
                    let v = topods::vertex(exv.current())?;
                    if tole > BRepTool::vertex_tolerance(&v)? {
                        b1.update_vertex_tolerance(&v, tole)?;
                    }
                    exv.next()?;
                }
            }
        }
        Ok(())
    }

    /// The data structure moved into its handle (`myDS`) for the builder of the result.
    pub(crate) fn move_ds(&mut self) {
        self.ds_moved = true;
        let ds = std::mem::replace(&mut self.ds, TopOpeBRepDSDataStructure::new());
        *self.my_ds.change_ds() = ds;
    }

    /// The compound of the merged solids (or of the solids when unchanged; the shells of
    /// the merged shells).
    pub(super) fn result_compound(
        &self,
        map_ind_so: &IntMap<i32>,
        shells_of_merged: bool,
    ) -> Result<Shape, Failure> {
        let mut c = Compound::default();
        Builder.make_compound(&mut c);
        let mut res: Shape = c.into();
        for indsol in map_ind_so.iter() {
            let curshape = self.my_ds.shape(*indsol, true);
            let merged = self.my_coup.merged(&curshape, TopState::In);
            if merged.is_empty() {
                Builder.add(&mut res, &curshape)?;
                continue;
            }
            for m in merged.iter() {
                if shells_of_merged && curshape.shape_type()? == ShapeEnum::Shell {
                    let expsh2 = Explorer::new(m, ShapeEnum::Shell, ShapeEnum::Shape)?;
                    Builder.add(&mut res, expsh2.current())?;
                } else {
                    Builder.add(&mut res, m)?;
                }
            }
        }
        Ok(res)
    }

    /// The new faces of the fillets made same parameter.
    pub(super) fn same_parameter_new_faces(&self) -> Result<(), Failure> {
        let same_par_tol = precision::confusion();
        for i_f in 1..=self.my_ds.nb_surfaces() {
            for a_f in self.my_coup.new_faces(i_f)?.iter() {
                same_parameter_shape(a_f, same_par_tol, true)?;
                ShapeFix::same_parameter(
                    a_f,
                    false,
                    same_par_tol,
                    &ProgressRange::default(),
                    &Handle::default(),
                )?;
            }
        }
        Ok(())
    }
}

impl dyn ChFi3dBuilderOps + '_ {
    /// `ExtentAnalyse()`: the extensions of the contours at the vertices where one, two
    /// or three of them meet (two or three when at most three sharp edges meet there).
    pub(crate) fn extent_analyse(&mut self) -> Result<(), Failure> {
        for iv in 1..=self.base().my_vdata_map.extent() {
            let (vtx, list, nbedges) = {
                let b = self.base();
                let vtx = b.my_vdata_map.find_key(iv)?;
                let list = b.my_vdata_map.find_from_index(iv)?.to_vec();
                let nbedges = number_of_sharp_edges(&vtx, &b.my_ve_map, &b.my_ef_map)?;
                (vtx, list, nbedges)
            };
            match list.len() {
                1 => self.extent_one_corner(&vtx, &list[0])?,
                2 if nbedges <= 3 => self.extent_two_corner(&vtx, &list)?,
                3 if nbedges <= 3 => self.extent_three_corner(&vtx, &list)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// The vertices of the contour ends at break points registered with their contours,
    /// the ends on free boundaries extended.
    pub(super) fn register_ends(&mut self, stripes: &[StripeHandle]) -> Result<(), Failure> {
        for st in stripes {
            let sp = spine_of(st)?;
            for first in [true, false] {
                let (status, v) = {
                    let s = lock(&sp);
                    if first {
                        (s.first_status(), s.first_vertex()?)
                    } else {
                        (s.last_status(), s.last_vertex()?)
                    }
                };
                if (status as i32) <= State::BreakPoint as i32 {
                    self.base_mut().my_vdata_map.add(&v, st);
                } else if status == State::FreeBoundary {
                    self.extent_one_corner(&v, st)?;
                }
            }
        }
        Ok(())
    }

    /// The fillets of every contour, the failing ones listed as bad stripes.
    pub(super) fn perform_contours(&mut self, stripes: &[StripeHandle]) -> Result<(), Failure> {
        for st in stripes {
            let sp = spine_of(st)?;
            lock(&sp).set_error_status(ErrorStatus::Ok);
            if self.perform_set_of_surf(st, false).is_err() {
                self.base_mut().badstripes.push(st.clone());
                self.base_mut().done = true;
                if lock(&sp).error_status() == ErrorStatus::Ok {
                    lock(&sp).set_error_status(ErrorStatus::Error);
                }
            }
            if !self.base().done {
                self.base_mut().badstripes.push(st.clone());
            }
            self.base_mut().done = true;
        }
        Ok(())
    }

    /// The corners at every vertex, the failing ones listed as bad vertices.
    pub(super) fn perform_corners(&mut self) -> Result<(), Failure> {
        for j in 1..=self.base().my_vdata_map.extent() {
            if self.perform_fillet_on_vertex(j).is_err() {
                let key = self.base().my_vdata_map.find_key(j)?;
                let b = self.base_mut();
                b.badvertices.push(key.into());
                b.hasresult = false;
                b.done = true;
            }
            if !self.base().done {
                let key = self.base().my_vdata_map.find_key(j)?;
                self.base_mut().badvertices.push(key.into());
            }
            self.base_mut().done = true;
        }
        if !self.base().hasresult {
            let b = self.base_mut();
            b.done = b.badvertices.is_empty();
        }
        Ok(())
    }

    /// The intersections of the contours with the edges of the others, and the fillets
    /// written into the data structure.
    pub(super) fn fill_ds(&mut self, stripes: &[StripeHandle]) -> Result<(), Failure> {
        for (i1, st) in stripes.iter().enumerate() {
            for check in &stripes[i1 + 1..] {
                let b = self.base_mut();
                if stripe_edge_inter(st, check, &mut b.ds, b.tol2d).is_err() {
                    b.badstripes.push(st.clone());
                    b.hasresult = false;
                    b.done = false;
                    break;
                }
            }
            let solidindex = lock(st).solid_index();
            let b = self.base_mut();
            fil_ds(
                solidindex,
                st,
                &mut b.ds,
                &mut b.my_regul,
                b.tolapp3d,
                b.tol2d,
            )?;
            if !self.base().done {
                break;
            }
        }
        Ok(())
    }

    /// `Compute()`: the fillets of all the contours, the corners and the result
    /// (Standard_Failure without contour); `(&mut fb as &mut dyn ChFi3dBuilderOps).compute()?`.
    pub fn compute(&mut self) -> Result<(), Failure> {
        self.base_mut().update_tolesp()?;
        if self.base().my_list_stripe.is_empty() {
            return Err(Failure::new(
                "There are no suitable edges for chamfer or fillet",
            ));
        }
        self.base_mut().reset()?;
        {
            let b = self.base_mut();
            b.my_ds = Handle::new(TopOpeBRepDSHDataStructure::new());
            b.ds = TopOpeBRepDSDataStructure::new();
            b.ds_moved = false;
            b.done = true;
            b.hasresult = false;
        }
        let stripes = self.base().my_list_stripe.clone();
        self.register_ends(&stripes)?;
        self.extent_analyse()?;
        self.perform_contours(&stripes)?;
        let done = self.base().badstripes.is_empty();
        self.base_mut().done = done;
        if done {
            self.perform_corners()?;
        }
        let map_ind_so = self.base_mut().solids_and_shells()?;
        if self.base().done {
            self.fill_ds(&stripes)?;
            if self.base().done {
                self.base_mut().rebuild(&map_ind_so)?;
                if !self.base().hasresult {
                    let res = self.base().result_compound(&map_ind_so, true)?;
                    self.base_mut().my_shape_result = res;
                } else {
                    self.base_mut().done = false;
                    let bad = self.base().result_compound(&map_ind_so, false)?;
                    self.base_mut().bad_shape = bad;
                }
                self.set_regul()?;
            }
        }
        if !self.base().ds_moved {
            self.base_mut().move_ds();
        }
        if self.base().is_done() {
            self.base().same_parameter_new_faces()?;
        }
        Ok(())
    }
}
