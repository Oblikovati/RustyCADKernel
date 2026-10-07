//! `BOPAlgo_PaveFiller_8.cxx`: splitting of degenerated edges (`ProcessDE`).
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_ds::{Pave, PaveBlockHandle, ShapeInfo};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve as ProjectPointOnCurve2d;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Splits each degenerated edge of a face at the 2D crossings of the blocks through its
    /// vertex, or replaces a degenerated edge of an edge argument (`ProcessDE`).
    /// `filler.process_de()?`.
    pub fn process_de(&mut self) -> Result<(), Failure> {
        let count = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .nb_source_shapes();
        for edge in 0..count {
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let info = structure.shape_info(edge)?;
            let mut owner = 0;
            if info.shape_type() != ShapeEnum::Edge || !info.has_flag_into(&mut owner) {
                continue;
            }
            let vertex = structure.same_domain_index(info.sub_shapes()[0]);
            match structure.shape_info(owner)?.shape_type() {
                ShapeEnum::Face => self.process_face_de(edge, owner, vertex)?,
                ShapeEnum::Edge => self.process_edge_de(edge, vertex)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn process_face_de(&mut self, edge: i32, face: i32, vertex: i32) -> Result<(), Failure> {
        let passing = self.find_pave_blocks(vertex, face)?;
        if !passing.is_empty() {
            let Some(degenerated) = self
                .structure_mut()?
                .change_pave_blocks(edge)?
                .first()
                .cloned()
            else {
                return Ok(());
            };
            self.fill_paves(vertex, edge, face, &passing, &degenerated)?;
            self.structure_mut()?.update_pave_block(&degenerated)?;
        }
        self.make_split_edge_de(edge, face)
    }

    /// A degenerated edge of an edge argument gets a new degenerated edge on its vertex.
    fn process_edge_de(&mut self, edge: i32, vertex: i32) -> Result<(), Failure> {
        let structure = self.structure_mut()?;
        let original = Edge::unchecked(structure.shape(edge)?.clone());
        let vertex = structure.shape(vertex)?.clone();
        let builder = BRepBuilder::default();
        let mut copy = original.empty_copied()?;
        builder.add(&mut copy, &vertex)?;
        let copy = Edge::unchecked(copy);
        builder.degenerated(&copy, true)?;
        builder.update_edge_tolerance(&copy, precision::confusion())?;
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Edge);
        info.set_shape(&copy.into_shape());
        let index = structure.append_shape_info(&info);
        let block = Arc::clone(&structure.change_pave_blocks(edge)?[0]);
        block.lock().expect("pave-block lock").set_edge(index);
        Ok(())
    }

    /// The In, On and Section blocks of the face passing through the vertex (`FindPaveBlocks`).
    fn find_pave_blocks(
        &mut self,
        vertex: i32,
        face: i32,
    ) -> Result<Vec<PaveBlockHandle>, Failure> {
        let info = self.structure_mut()?.change_face_info(face)?;
        let mut found = Vec::new();
        for blocks in [
            info.pave_blocks_in(),
            info.pave_blocks_on(),
            info.pave_blocks_section(),
        ] {
            for block in blocks.iter() {
                let (first, last) = block.lock().expect("pave-block lock").indices();
                if vertex == first || vertex == last {
                    found.push(Arc::clone(block));
                }
            }
        }
        Ok(found)
    }

    /// Splits the degenerated edge's blocks into new degenerated edges, or drops its pool
    /// entry when it stays whole (`MakeSplitEdge`).
    fn make_split_edge_de(&mut self, edge: i32, face: i32) -> Result<(), Failure> {
        let structure = self.structure_mut()?;
        let degenerated = Edge::unchecked(structure.shape(edge)?.oriented(Orientation::Forward));
        let face = Face::unchecked(structure.shape(face)?.clone());
        let blocks = structure.change_pave_blocks(edge)?.clone();
        let count = blocks.len();
        for block in blocks {
            let structure = self.structure_mut()?;
            let (first, last) = {
                let locked = block.lock().expect("pave-block lock");
                (locked.pave1(), locked.pave2())
            };
            if !(structure.is_new_shape(first.index()) || count > 1) {
                let reference = structure.shape_info(edge)?.reference() as usize;
                structure.change_shape_info(edge)?.set_reference(-1);
                structure.change_pave_blocks_pool()[reference].clear();
                break;
            }
            let v1 = Vertex::unchecked(
                structure
                    .shape(first.index())?
                    .oriented(Orientation::Forward),
            );
            let v2 = Vertex::unchecked(
                structure
                    .shape(last.index())?
                    .oriented(Orientation::Reversed),
            );
            let split = make_split_edge1(
                &degenerated,
                &face,
                (&v1, first.parameter()),
                (&v2, last.parameter()),
            )?;
            let mut info = ShapeInfo::default();
            info.set_shape_type(ShapeEnum::Edge);
            info.set_shape(&split.into_shape());
            let index = structure.append_shape_info(&info);
            block.lock().expect("pave-block lock").set_edge(index);
        }
        Ok(())
    }

    /// Adds the 2D crossings of the passing blocks' p-curves with the degenerated edge's
    /// p-curve as extra paves of its block (`FillPaves`).
    fn fill_paves(
        &self,
        vertex: i32,
        edge: i32,
        face: i32,
        passing: &[PaveBlockHandle],
        degenerated: &PaveBlockHandle,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let vertex_shape = Vertex::unchecked(structure.shape(vertex)?.clone());
        let edge_shape = Edge::unchecked(structure.shape(edge)?.clone());
        let face_shape = Face::unchecked(structure.shape(face)?.clone());
        let tolerance = BRepTool::vertex_tolerance(&vertex_shape)?;
        let surface = self.context_ref()?.surface_adaptor(&face_shape)?;
        let (u_resolution, v_resolution) = {
            let surface = surface.read();
            (
                surface.u_resolution(tolerance)?,
                surface.v_resolution(tolerance)?,
            )
        };
        // 2D tolerances from the vertex tolerance resolve the touching cases.
        let intersection_tolerance = precision::p_confusion().max(u_resolution.max(v_resolution));
        let on_face = BRepTool::curve_on_face(&edge_shape, &face_shape)?;
        let Some(pcurve) = on_face.pcurve.get() else {
            return Ok(());
        };
        let along_u = (pcurve.value(on_face.first)?.y() - pcurve.value(on_face.last)?.y()).abs()
            < precision::p_confusion();
        let compare =
            precision::p_confusion().max(if along_u { u_resolution } else { v_resolution });
        let degenerated_adaptor =
            Geom2dAdaptorCurve::with_range(&on_face.pcurve, on_face.first, on_face.last)?;
        for block in passing {
            let (split_edge, paves) = {
                let locked = block.lock().expect("pave-block lock");
                (locked.edge(), (locked.pave1(), locked.pave2()))
            };
            if split_edge < 0 {
                continue;
            }
            let split_shape = Edge::unchecked(structure.shape(split_edge)?.clone());
            let other = BRepTool::curve_on_face(&split_shape, &face_shape)?;
            if other.pcurve.is_null() {
                continue;
            }
            let other_adaptor = if other.pcurve.cast::<Geom2dLine>().is_some() {
                Geom2dAdaptorCurve::with_curve(&other.pcurve)?
            } else {
                Geom2dAdaptorCurve::with_range(&other.pcurve, other.first, other.last)?
            };
            let intersector = GInter::between(
                &degenerated_adaptor,
                &other_adaptor,
                intersection_tolerance,
                intersection_tolerance,
            )?;
            if intersector.is_done() && intersector.nb_points()? > 0 {
                for index in 1..=intersector.nb_points()? {
                    let parameter = intersector.point(index)?.param_on_first();
                    add_split_point(degenerated, Pave::new(vertex, parameter), compare);
                }
                continue;
            }
            // The intersection failed: project the end of the curve at the vertex.
            let parameter = if vertex == paves.0.index() {
                paves.0.parameter()
            } else {
                paves.1.parameter()
            };
            let point = other.pcurve.value(parameter)?;
            let projection = ProjectPointOnCurve2d::with_range(
                point,
                &on_face.pcurve,
                on_face.first,
                on_face.last,
            )?;
            if projection.nb_points() > 0 {
                let parameter = projection.lower_distance_parameter()?;
                add_split_point(degenerated, Pave::new(vertex, parameter), compare);
            }
        }
        Ok(())
    }
}

/// Native static `MakeSplitEdge1`: a degenerated edge between the vertices on the face.
fn make_split_edge1(
    edge: &Edge,
    face: &Face,
    (v1, p1): (&Vertex, f64),
    (v2, p2): (&Vertex, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut copy = edge.empty_copied()?;
    builder.add(&mut copy, &v1.clone().into_shape())?;
    builder.add(&mut copy, &v2.clone().into_shape())?;
    let copy = Edge::unchecked(copy);
    builder.range_on_face(&copy, face, p1, p2)?;
    builder.degenerated(&copy, true)?;
    builder.update_edge_tolerance(&copy, 1.0e-7)?;
    Ok(copy)
}

/// Native static `AddSplitPoint`: accepts a parameter strictly inside the block and not
/// already used, as an unconditional extra pave.
fn add_split_point(block: &PaveBlockHandle, pave: Pave, tolerance: f64) -> bool {
    let mut locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = locked.range();
    let parameter = pave.parameter();
    if parameter - first < tolerance || last - parameter < tolerance {
        return false;
    }
    let mut used = 0;
    if locked.contains_parameter(parameter, tolerance, &mut used) {
        return false;
    }
    locked.append_ext_pave1(pave);
    true
}
