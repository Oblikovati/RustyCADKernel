// Rust translation of the vertex phase and the line creation of
// IntPatch_ALineToWLine::MakeWLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ALineToWLine.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of the sampled line and the creation of the walking line.
use super::super::IntPatchSpecPntType as Spt;
use super::super::line::IntPatchLine;
use super::super::point::IntPatchPoint;
use super::super::point_line::IntPatchPointLine;
use super::super::wline::{IntPatchWLType, IntPatchWLine};
use super::specials::{PoleOrSeamInput, add_point_into_line, add_vertex_point, is_pole_or_seam};
use super::walk::{Flow, Walk, point_on_quadrics};
use crate::int_surf::{IntSurfPntOn2S, IntSurfTypeTrans};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Vector3};

impl Walk<'_> {
    /// The reference point for the parameters that cannot be made precise: the last
    /// point of the line, or (on an empty line) the point halfway to the next vertex of
    /// greater parameter, or the vertex point itself.
    fn reference_iso(
        &self,
        vertex_number: usize,
        vtx: &IntPatchPoint,
    ) -> Result<IntSurfPntOn2S, Failure> {
        let nb: i32 = self.lin_on_2s.nb_points();
        if nb >= 1 {
            return self.lin_on_2s.value(nb);
        }
        let vparam: f64 = self.vertex_params[vertex_number];
        for &param in &self.vertex_params[vertex_number + 1..] {
            if (param - vparam) > precision::p_confusion() {
                let pnt_3d: Point3 = self.aline.value(0.5 * (param + vparam))?;
                return Ok(point_on_quadrics(self.conv, pnt_3d));
            }
        }
        Ok(vtx.pnt_on_2s())
    }

    /// The vertex phase of the loop body: the vertex goes into the line as a special
    /// point, as its own point or as the current point; the vertices at its place are
    /// checked, then the step is recomputed after a reduced one.
    pub(super) fn add_vertex(
        &mut self,
        vertex_number: usize,
        mut p_on_2s: IntSurfPntOn2S,
        is_point_valid: bool,
    ) -> Result<Flow, Failure> {
        let mut vtx: IntPatchPoint = self.aline.vertex(vertex_number as i32 + 1)?;
        let nb_points_prev: i32 = self.lin_on_2s.nb_points();
        let mut new_vertex_param: f64 = f64::from(nb_points_prev + 1);
        let pref_iso: IntSurfPntOn2S = self.reference_iso(vertex_number, &vtx)?;
        let input = PoleOrSeamInput {
            s1: &self.conv.s1,
            s2: &self.conv.s2,
            arr_periods: &self.periods,
            tol_3d: self.tol,
        };
        let (spt, id) = is_pole_or_seam(&input, &pref_iso, &self.lin_on_2s, &mut vtx)?;
        (self.pre_point_exist, self.singular_surface_id) = (spt, id);
        if spt == Spt::SPntPole || spt == Spt::SPntPoleSeamU {
            // Set the status: to be corrected later.
            if self.lin_on_2s.nb_points() == 1 {
                self.first_degenerated = true;
            } else {
                self.last_degenerated = true;
            }
        }
        let cur_vert_param: f64 = vtx.parameter_on_line();
        if spt != Spt::SPntNone {
            if nb_points_prev == self.lin_on_2s.nb_points() {
                // The vertex coincides with a point of the line and was not added.
                new_vertex_param = f64::from(nb_points_prev);
            }
            (self.prev_param, self.parameter) = (cur_vert_param, cur_vert_param);
        } else {
            if !is_point_valid {
                // Take a farther point of the line (with greater parameter).
                return Ok(Flow::Next);
            }
            if vtx.tolerance() > self.tol {
                vtx.set_pnt_on_2s(&p_on_2s);
                add_point_into_line(&self.lin_on_2s, &self.periods, &mut p_on_2s, None)?;
            } else {
                add_vertex_point(&self.lin_on_2s, &mut vtx, &self.periods)?;
            }
        }
        self.prev_l_point = self.lin_on_2s.value(self.lin_on_2s.nb_points())?;
        self.check_vertices_at(&vtx, cur_vert_param, new_vertex_param)?;
        if spt != Spt::SPntNone && self.lin_on_2s.nb_points() > 1 {
            return Ok(Flow::Break);
        }
        if self.is_step_reduced {
            return Ok(self.recompute_reduced_step());
        }
        Ok(Flow::Next)
    }

    /// Every vertex not checked yet whose point is the vertex one (within the
    /// tolerance) gets its point, tolerance and parameter on the new line (-1 at the
    /// end of a closed curve, set once the number of points is known); the search stops
    /// at the first farther vertex after a found one.
    fn check_vertices_at(
        &mut self,
        vtx: &IntPatchPoint,
        cur_vert_param: f64,
        new_vertex_param: f64,
    ) -> Result<(), Failure> {
        let mut is_found: bool = false;
        let sq_tol: f64 = self.tol * self.tol;
        let p1: Point3 = self.aline.value(cur_vert_param)?;
        let (vert_p2s, vert_toler) = (vtx.pnt_on_2s(), vtx.tolerance());
        for i in 0..self.vertex_params.len() {
            if self.checked[i] {
                continue;
            }
            let p2: Point3 = self.aline.value(self.vertex_params[i])?;
            if p1.square_distance(&p2) < sq_tol {
                let mut l_vtx: IntPatchPoint = self.aline.vertex(i as i32 + 1)?;
                l_vtx.set_pnt_on_2s(&vert_p2s);
                l_vtx.set_tolerance(vert_toler);
                let param: f64 = l_vtx.parameter_on_line();
                if (param - self.l_par).abs() <= precision::p_confusion() {
                    // A closed curve: the number of points is not known yet.
                    l_vtx.set_parameter(-1.0);
                } else {
                    l_vtx.set_parameter(new_vertex_param);
                }
                self.new_vert_id += 1;
                self.store_vertex(l_vtx)?;
                self.checked[i] = true;
                is_found = true;
            } else if is_found {
                break;
            }
        }
        Ok(())
    }

    /// `aSeqVertex(++aNewVertID) = aLVtx`, with the bound check of NCollection_Array1.
    fn store_vertex(&mut self, vtx: IntPatchPoint) -> Result<(), Failure> {
        let len: usize = self.seq_vertex.len();
        let slot = self
            .seq_vertex
            .get_mut(self.new_vert_id - 1)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "IntPatch_ALineToWLine::MakeWLine() - vertex slot {}; expected 1..={len}",
                    self.new_vert_id
                ))
            })?;
        *slot = vtx;
        Ok(())
    }

    /// The step after a vertex reached by a reduced step: the regular step, reduced
    /// again when it passes the next vertex; `Break` under the resolution of `L`.
    fn recompute_reduced_step(&mut self) -> Flow {
        self.is_step_reduced = false;
        self.step = (self.l_par - self.parameter) / f64::from(self.conv.nb_points_in_wline - 1);
        if self.step < epsilon(self.l_par) {
            return Flow::Break;
        }
        let mut l_par: f64 = *self.vertex_params.last().unwrap_or(&self.l_par);
        for (i, &param) in self.vertex_params.iter().enumerate() {
            if self.checked[i] {
                continue;
            }
            l_par = param;
            if (l_par - self.parameter).abs() < self.prm_tol {
                continue;
            }
            break;
        }
        self.reduce_step_to(l_par);
        (self.step_min, self.step_max) = (0.1 * self.step, 10.0 * self.step);
        Flow::Next
    }

    /// The transitions of the new line from the analytic one: TOUCH and undecided are
    /// copied; IN/OUT come from the sign of the mixed product of a chord of the line
    /// and the two quadric normals.
    fn new_wline(&self) -> Result<IntPatchWLine, Failure> {
        let (points, tang) = (&self.lin_on_2s, self.aline.is_tangent());
        match self.aline.transition_on_s1() {
            IntSurfTypeTrans::Touch => {
                let (situ1, situ2) = (self.aline.situation_s1()?, self.aline.situation_s2()?);
                Ok(IntPatchWLine::new_touch(points, tang, situ1, situ2))
            }
            IntSurfTypeTrans::Undecided => Ok(IntPatchWLine::new_undecided(points, tang)),
            _ => {
                let (trans1, trans2) = self.crossing_transitions()?;
                Ok(IntPatchWLine::new(points, tang, trans1, trans2))
            }
        }
    }

    fn crossing_transitions(&self) -> Result<(IntSurfTypeTrans, IntSurfTypeTrans), Failure> {
        let indice1: i32 = std::cmp::max(self.lin_on_2s.nb_points() / 3, 2);
        let pp0: Point3 = self.lin_on_2s.value(indice1 - 1)?.value();
        let pp1: Point3 = self.lin_on_2s.value(indice1)?.value();
        let tgvalid = Vector3::between(pp0, pp1);
        let nq1: Vector3 = self.conv.quad1.normale_at(pp0)?;
        let nq2: Vector3 = self.conv.quad2.normale_at(pp0)?;
        let dotcross: f64 = tgvalid.dot_cross(nq2, nq1);
        let tol: f64 = self.conv.tol_transition;
        if dotcross > tol {
            return Ok((IntSurfTypeTrans::Out, IntSurfTypeTrans::In));
        }
        if dotcross < -tol {
            return Ok((IntSurfTypeTrans::In, IntSurfTypeTrans::Out));
        }
        Ok((IntSurfTypeTrans::Undecided, IntSurfTypeTrans::Undecided))
    }

    /// The end of one line: the degenerated end points are corrected, the walking line
    /// gets its vertices and periods, and is kept when it still has 2 points after
    /// `ComputeVertexParameters`.
    pub(super) fn build_wline(&mut self) -> Result<Option<Handle<dyn IntPatchLine>>, Failure> {
        if self.lin_on_2s.nb_points() >= 3 {
            if self.first_degenerated {
                self.conv.correct_end_point(&self.lin_on_2s, 1)?;
            }
            if self.last_degenerated {
                self.conv
                    .correct_end_point(&self.lin_on_2s, self.lin_on_2s.nb_points())?;
            }
        }
        let wline: IntPatchWLine = self.new_wline()?;
        wline.set_creating_way_info(IntPatchWLType::ImpImp);
        for vtx in &self.seq_vertex[..self.new_vert_id] {
            let mut vtx: IntPatchPoint = vtx.clone();
            if vtx.parameter_on_line() == -1.0 {
                // A closed curve: the last parameter.
                vtx.set_parameter(f64::from(wline.nb_pnts()?));
            }
            wline.add_vertex(&vtx, false);
        }
        let [pu1, pv1, pu2, pv2] = self.periods;
        wline.set_period(pu1, pv1, pu2, pv2);
        // ComputeVertexParameters can reduce the number of points of the line.
        wline.compute_vertex_parameters(self.conv.tol_3d)?;
        if wline.nb_pnts()? <= 1 {
            return Ok(None);
        }
        wline.enable_purging(false);
        Ok(Some(Handle::new(wline).cast()))
    }
}
