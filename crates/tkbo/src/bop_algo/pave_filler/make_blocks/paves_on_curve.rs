//! `BOPAlgo_PaveFiller_6.cxx`: putting vertices as paves on the section curves.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::{CurveRef, VertexGroups, VertexTolerances};
use crate::bop_ds::{Pave, PaveBlockHandle};
use crate::bop_tools::{d_tolerance, point_on_edge};
use crate::int_tools::Curve;
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

impl BOPAlgoPaveFiller {
    #[allow(clippy::too_many_arguments)]
    /// Puts the EF vertices, then the other On/In vertices (new or common, with a box touching
    /// the curve) on the curve (`PutPavesOnCurve`).
    pub(super) fn put_paves_on_curve(
        &mut self,
        vertices_on_in: &Map<i32>,
        vertices_common: &Map<i32>,
        curve: CurveRef,
        shapes: &Map<i32>,
        vertices_ef: &Map<i32>,
        tolerances: &mut VertexTolerances,
        groups: &mut VertexGroups,
    ) -> Result<(), Failure> {
        let (curve_box, tolerance) = {
            let stored = self.ff_curve(curve)?;
            (
                stored.box_bounds().clone(),
                stored.tolerance().max(stored.tangential_tolerance()),
            )
        };
        for vertex in vertices_ef.iter().copied() {
            self.put_pave_on_curve(vertex, tolerance, curve, shapes, tolerances, groups, 2)?;
        }
        for vertex in vertices_on_in.iter().copied() {
            if vertices_ef.contains(&vertex) {
                continue;
            }
            if !vertices_common.contains(&vertex) {
                let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
                if curve_box.is_out_box(structure.shape_info(vertex)?.box_bounds())
                    || !structure.is_new_shape(vertex)
                {
                    continue;
                }
            }
            self.put_pave_on_curve(vertex, tolerance, curve, shapes, tolerances, groups, 1)?;
        }
        Ok(())
    }

    /// Puts one vertex on the curve: reuses a pave at the same parameter (grouping the
    /// vertices for later fusion) or adds an extra pave, growing the vertex tolerance to reach
    /// the curve (`PutPaveOnCurve`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn put_pave_on_curve(
        &mut self,
        vertex: i32,
        curve_tolerance: f64,
        curve: CurveRef,
        shapes: &Map<i32>,
        tolerances: &mut VertexTolerances,
        groups: &mut VertexGroups,
        check_extend: i32,
    ) -> Result<(), Failure> {
        let (block, section) = self.curve_first_block(curve)?;
        let shape = self.vertex_shape(vertex)?;
        let mut vertex_tolerance = match tolerances.seek(&vertex) {
            Some(saved) => *saved,
            None => BRepTool::vertex_tolerance(&shape)?,
        };
        let Some(parameter) = self.vertex_on_section(
            vertex,
            &shape,
            &mut vertex_tolerance,
            &section,
            curve_tolerance,
            shapes,
            check_extend,
        )?
        else {
            return Ok(());
        };
        let adaptor = GeomAdaptorCurve::with_curve(section.curve())?;
        let parameter_tolerance = adaptor.resolution(curve_tolerance.max(vertex_tolerance))?;
        let mut used = -1;
        let found = block.lock().expect("pave-block lock").contains_parameter(
            parameter,
            parameter_tolerance,
            &mut used,
        );
        if found {
            return self.group_with_used_pave(vertex, &shape, used, tolerances, groups);
        }
        block
            .lock()
            .expect("pave-block lock")
            .append_ext_pave(Pave::new(vertex, parameter));
        let on_curve = adaptor.value(parameter)?;
        let tolerance = BRepTool::vertex_tolerance(&shape)?;
        let distance = on_curve.distance(&BRepTool::pnt(&shape)?);
        if tolerance < distance + d_tolerance() {
            BRepBuilder::default().update_vertex_tolerance(&shape, distance + d_tolerance())?;
            if !tolerances.is_bound(&vertex) {
                tolerances.bind(vertex, tolerance);
            }
            self.refresh_vertex_box(vertex, &shape)?;
        }
        Ok(())
    }

    /// The vertex parameter on the section when the vertex lies on it, possibly using the
    /// extended tolerance from its EE/EF origin; updates `vertex_tolerance` in that case.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn vertex_on_section(
        &self,
        vertex: i32,
        shape: &Vertex,
        vertex_tolerance: &mut f64,
        section: &Curve,
        curve_tolerance: f64,
        shapes: &Map<i32>,
        check_extend: i32,
    ) -> Result<Option<f64>, Failure> {
        let context = self.context_ref()?;
        let line_tolerance = curve_tolerance + self.options.fuzzy_value();
        let mut parameter = 0.;
        if context.is_vertex_on_line_tolerance(
            shape,
            *vertex_tolerance,
            section,
            line_tolerance,
            &mut parameter,
        )? {
            return Ok(Some(parameter));
        }
        if check_extend == 0 || self.vertices_to_avoid_extension.contains(&vertex) {
            return Ok(None);
        }
        let mut extra = *vertex_tolerance;
        if !self.extended_tolerance(vertex, shapes, &mut extra, check_extend)? {
            return Ok(None);
        }
        if !context.is_vertex_on_line_tolerance(
            shape,
            extra,
            section,
            line_tolerance,
            &mut parameter,
        )? {
            return Ok(None);
        }
        let mut on_curve = Point3::default();
        section.d0(parameter, &mut on_curve)?;
        *vertex_tolerance = on_curve.distance(&BRepTool::pnt(shape)?);
        Ok(Some(parameter))
    }

    /// Records that `vertex` coincides with the pave `used` already on the curve.
    pub(super) fn group_with_used_pave(
        &self,
        vertex: i32,
        shape: &Vertex,
        used: i32,
        tolerances: &mut VertexTolerances,
        groups: &mut VertexGroups,
    ) -> Result<(), Failure> {
        if groups.seek(&used).is_none() {
            groups.bind(used, vec![used]);
            if !tolerances.is_bound(&used) {
                let used_tolerance = BRepTool::vertex_tolerance(&self.vertex_shape(used)?)?;
                tolerances.bind(used, used_tolerance);
            }
        }
        let list = groups.change_seek(&used).expect("bound just above");
        if !list.contains(&vertex) {
            list.push(vertex);
        }
        if !tolerances.is_bound(&vertex) {
            tolerances.bind(vertex, BRepTool::vertex_tolerance(shape)?);
        }
        Ok(())
    }

    /// The vertex tolerance widened to cover its EE (type 1) or EF (type 2) common range
    /// between shapes of `shapes`; true when such an interference exists (`ExtendedTolerance`).
    pub(super) fn extended_tolerance(
        &self,
        vertex: i32,
        shapes: &Map<i32>,
        tolerance: &mut f64,
        kind: i32,
    ) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        if !structure.is_new_shape(vertex) {
            return Ok(false);
        }
        let point = BRepTool::pnt(&self.vertex_shape(vertex)?)?;
        let (start, end) = match kind {
            1 => (0, 1),
            2 => (1, 2),
            _ => (0, 2),
        };
        for k in start..end {
            let parts: Vec<_> = if k == 0 {
                structure
                    .interf_ee_records()
                    .iter()
                    .map(|r| (r.index_new(), r.indices(), r.common_part().clone()))
                    .collect()
            } else {
                structure
                    .interf_ef_records()
                    .iter()
                    .map(|r| (r.index_new(), r.indices(), r.common_part().clone()))
                    .collect()
            };
            for (new_index, (first, second), part) in parts {
                if new_index != vertex || !shapes.contains(&first) || !shapes.contains(&second) {
                    continue;
                }
                let (t1, t2) = part.range1().range();
                let (mut p1, mut p2) = (Point3::default(), Point3::default());
                point_on_edge(part.edge1(), t1, &mut p1)?;
                point_on_edge(part.edge1(), t2, &mut p2)?;
                let (d1, d2) = (point.distance(&p1), point.distance(&p2));
                let distance = if d1 > d2 { d1 } else { d2 };
                if distance > *tolerance {
                    *tolerance = distance;
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The vertices at the first and last extra pave of the curve's first block, kept only
    /// when they lie at the corresponding curve end (native static `getBoundPaves`).
    pub(super) fn bound_paves(&self, curve: CurveRef) -> Result<[i32; 2], Failure> {
        let mut bounds = [-1, -1];
        let (block, section) = self.curve_first_block(curve)?;
        let paves: Vec<Pave> = block.lock().expect("pave-block lock").ext_paves().to_vec();
        if paves.is_empty() {
            return Ok(bounds);
        }
        let (mut minimum, mut maximum) = (f64::MAX, -f64::MAX);
        for pave in &paves {
            if pave.parameter() < minimum {
                bounds[0] = pave.index();
                minimum = pave.parameter();
            }
            if pave.parameter() > maximum {
                bounds[1] = pave.index();
                maximum = pave.parameter();
            }
        }
        let stored = self.ff_curve(curve)?;
        let tolerance =
            stored.tolerance().max(stored.tangential_tolerance()) + precision::confusion();
        let (mut t1, mut t2, mut p1, mut p2) = (0., 0., Point3::default(), Point3::default());
        section.bounds(&mut t1, &mut t2, &mut p1, &mut p2)?;
        for (end, point) in [(0, p1), (1, p2)] {
            let shape = self.vertex_shape(bounds[end])?;
            if crate::bop_tools::compute_vertex_point(&shape, point, tolerance)? != 0 {
                bounds[end] = -1;
            }
        }
        Ok(bounds)
    }

    /// Re-adds the vertex box with its current tolerance plus Confusion to the stored box.
    pub(super) fn refresh_vertex_box(
        &mut self,
        vertex: i32,
        shape: &Vertex,
    ) -> Result<(), Failure> {
        let bounds = self
            .structure_mut()?
            .change_shape_info(vertex)?
            .change_box();
        BRepBndLib::add(shape, bounds, true)?;
        bounds.set_gap(bounds.gap() + precision::confusion());
        Ok(())
    }

    /// Sets the stored vertex tolerance directly, as `BRep_TVertex::Tolerance(T)` does.
    pub(super) fn set_vertex_tolerance(shape: &Vertex, tolerance: f64) -> Result<(), Failure> {
        vertex_tshape(shape, "BRep_TVertex::Tolerance")?.set_tolerance(tolerance);
        Ok(())
    }

    pub(super) fn vertex_shape(&self, vertex: i32) -> Result<Vertex, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        Ok(Vertex::unchecked(structure.shape(vertex)?.clone()))
    }

    /// The first pave block of the curve and a copy of its section curve.
    pub(super) fn curve_first_block(
        &self,
        curve: CurveRef,
    ) -> Result<(PaveBlockHandle, Curve), Failure> {
        let stored = self.ff_curve(curve)?;
        let block = stored.pave_blocks().first().cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!("BOPDS_Curve {curve:?} has no pave block; expected InitPaveBlock1"),
                "",
            )
        })?;
        Ok((block, stored.curve().clone()))
    }
}
