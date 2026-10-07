//! `BOPAlgo_PaveFiller_6.cxx`: EF and stick vertices put on section curves, and the
//! collection of stick vertices.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::{CurveRef, VertexGroups, VertexTolerances};
use crate::bop_tools::algo_tools3d::get_normal_to_surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::Failure;

use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction3, Point3};

impl BOPAlgoPaveFiller {
    /// Puts the EF vertices not yet used on a Bezier/BSpline section by projection
    /// (`PutEFPavesOnCurve`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn put_ef_paves_on_curve(
        &mut self,
        curve: CurveRef,
        shapes: &Map<i32>,
        vertices_ef: &Map<i32>,
        tolerances: &mut VertexTolerances,
        groups: &mut VertexGroups,
    ) -> Result<(), Failure> {
        if vertices_ef.extent() == 0 {
            return Ok(());
        }
        let section = self.ff_curve(curve)?.curve().clone();
        if !matches!(
            section.curve_type()?,
            CurveType::BezierCurve | CurveType::BSplineCurve
        ) {
            return Ok(());
        }
        let mut unused: Map<i32> = Map::new(1);
        unused.assign(vertices_ef);
        self.remove_used_vertices(curve.0, &mut unused)?;
        if unused.extent() == 0 {
            return Ok(());
        }
        let projector = self.context_ref()?.proj_pt(section.curve())?;
        for vertex in unused.iter().copied() {
            let point = BRepTool::pnt(&self.vertex_shape(vertex)?)?;
            let distance = {
                let mut projection = projector.write();
                projection.perform(point)?;
                if projection.nb_points() == 0 {
                    continue;
                }
                projection.lower_distance()?
            };
            self.put_pave_on_curve(vertex, distance, curve, shapes, tolerances, groups, 0)?;
        }
        Ok(())
    }
    /// Puts unused stick vertices at a free curve end where the faces are tangent (a crease)
    /// (`PutStickPavesOnCurve`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn put_stick_paves_on_curve(
        &mut self,
        faces: &[Face; 2],
        shapes: &Map<i32>,
        curve: CurveRef,
        vertices_stick: &Map<i32>,
        tolerances: &mut VertexTolerances,
        groups: &mut VertexGroups,
    ) -> Result<(), Failure> {
        let bounds = self.bound_paves(curve)?;
        if bounds[0] >= 0 && bounds[1] >= 0 {
            return Ok(());
        }
        let mut unused: Map<i32> = Map::new(1);
        unused.assign(vertices_stick);
        self.remove_used_vertices(curve.0, &mut unused)?;
        if unused.extent() == 0 {
            return Ok(());
        }
        let section = self.ff_curve(curve)?.curve().clone();
        let pcurves = [
            section.first_curve2d().clone(),
            section.second_curve2d().clone(),
        ];
        if pcurves[0].is_null() || pcurves[1].is_null() {
            return Ok(());
        }
        let surfaces = [BRepTool::surface(&faces[0])?, BRepTool::surface(&faces[1])?];
        let (mut t1, mut t2, mut p1, mut p2) = (0., 0., Point3::default(), Point3::default());
        section.bounds(&mut t1, &mut t2, &mut p1, &mut p2)?;
        let (ends, end_points) = ([t1, t2], [p1, p2]);
        for vertex in unused.iter().copied() {
            let point = BRepTool::pnt(&self.vertex_shape(vertex)?)?;
            for end in 0..2 {
                if bounds[end] >= 0 {
                    continue;
                }
                let square = end_points[end].square_distance(&point);
                if square > 2.0e-7 {
                    continue;
                }
                let mut normals = [Direction3::default(); 2];
                for side in 0..2 {
                    let uv = pcurves[side].value(ends[end])?;
                    get_normal_to_surface(&surfaces[side], uv.x(), uv.y(), &mut normals[side])?;
                }
                let mut product = normals[0].dot(normals[1]);
                if product < 0. {
                    product = -product;
                }
                if 1. - product > 5.0e-9 {
                    continue;
                }
                // The section is a vanishing curve (a crease) at this end.
                self.put_pave_on_curve(
                    vertex,
                    square.sqrt(),
                    curve,
                    shapes,
                    tolerances,
                    groups,
                    0,
                )?;
            }
        }
        Ok(())
    }
    /// The new vertices of VV, VE, EE, VF (stick) and EF (also `vertices_ef`) interferences
    /// between sub-shapes of the two faces, collected in `shapes` (`GetStickVertices`).
    pub(super) fn get_stick_vertices(
        &self,
        faces: (i32, i32),
        vertices_stick: &mut Map<i32>,
        vertices_ef: &mut Map<i32>,
        shapes: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        shapes.clear(false);
        for face in [faces.0, faces.1] {
            shapes.add(face);
            for sub_shape in structure.shape_info(face)?.sub_shapes() {
                shapes.add(*sub_shape);
            }
        }
        let mut stick: Vec<(i32, (i32, i32))> = Vec::new();
        stick.extend(
            structure
                .interf_vv_records()
                .iter()
                .map(|r| (r.index_new(), r.indices())),
        );
        stick.extend(
            structure
                .interf_ve_records()
                .iter()
                .map(|r| (r.index_new(), r.indices())),
        );
        stick.extend(
            structure
                .interf_ee_records()
                .iter()
                .map(|r| (r.index_new(), r.indices())),
        );
        stick.extend(
            structure
                .interf_vf_records()
                .iter()
                .map(|r| (r.index_new(), r.indices())),
        );
        for (new_index, (first, second)) in stick {
            if new_index >= 0 && shapes.contains(&first) && shapes.contains(&second) {
                vertices_stick.add(structure.same_domain_index(new_index));
            }
        }
        for record in structure.interf_ef_records().iter() {
            let (first, second) = record.indices();
            if record.has_index_new() && shapes.contains(&first) && shapes.contains(&second) {
                let vertex = structure.same_domain_index(record.index_new());
                vertices_stick.add(vertex);
                vertices_ef.add(vertex);
            }
        }
        Ok(())
    }
    /// Removes from `vertices` every vertex already used by the pave blocks of the curves of
    /// the face/face interference `ff` (`RemoveUsedVertices`).
    pub(super) fn remove_used_vertices(
        &self,
        ff: usize,
        vertices: &mut Map<i32>,
    ) -> Result<(), Failure> {
        if vertices.extent() == 0 {
            return Ok(());
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        for curve in structure.interf_ff_records()[ff].curves().iter() {
            for block in curve.pave_blocks() {
                let locked = block.lock().expect("pave-block lock");
                for pave in locked.ext_paves() {
                    vertices.remove(&pave.index());
                }
                vertices.remove(&locked.pave1().index());
                vertices.remove(&locked.pave2().index());
            }
        }
        Ok(())
    }
}
