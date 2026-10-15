// Rust translation of BRepOffset_MakeOffset::RemoveInternalEdges of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx, and the entry points
// of BRepOffset_MakeOffset_1.cxx (BuildSplitsOfExtendedFaces, BuildSplitsOfTrimmedFaces)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The removal of the internal edges of the result, and the complete-intersection
//! splits of planar solids.
use super::BRepOffsetMakeOffset;
use super::statics::{EdgesOrigins, remove_shapes};
use crate::brep_offset::build_offset_faces::BuildOffsetFaces;
use crate::brep_offset::inter3d::ShapeShapeMap;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl BRepOffsetMakeOffset {
    /// `RemoveInternalEdges()`: the INTERNAL edges of a single face removed from their
    /// wires, and the wires of such edges only removed from their faces.
    pub(super) fn remove_internal_edges(&mut self) -> Result<(), Failure> {
        let mut a_dmelf = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.my_offset_shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut a_dmelf,
        )?;
        for mut a_f in Explorer::new(&self.my_offset_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let mut a_liw = List::default();
            for mut a_w in Explorer::new(&a_f, ShapeEnum::Wire, ShapeEnum::Shape)? {
                let mut b_remove_wire = true;
                let mut a_lie = List::default();
                for a_e in Explorer::new(&a_w, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if a_e.orientation() != Orientation::Internal {
                        b_remove_wire = false;
                        continue;
                    }
                    let a_lf = a_dmelf.find_from_key(&a_e)?;
                    if a_lf.len() == 1 {
                        a_lie.append(a_e);
                    } else {
                        b_remove_wire = false;
                    }
                }
                if b_remove_wire {
                    a_liw.append(a_w);
                } else if !a_lie.is_empty() {
                    remove_shapes(&mut a_w, &a_lie)?;
                }
            }
            if !a_liw.is_empty() {
                remove_shapes(&mut a_f, &a_liw)?;
            }
        }
        Ok(())
    }

    /// `BuildSplitsOfExtendedFaces(theLF, theAnalyse, theAsDes, theEdgesOrigins,
    /// theFacesOrigins, theETrimEInf, theImage, Range)`: the splits of the extended offset
    /// faces, with the invalid splits rebuilt (BRepOffset_BuildOffsetFaces).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_splits_of_extended_faces(
        &mut self,
        the_lf: &List<rustycad_tkbrep::topods::Shape>,
        the_as_des: &BRepAlgoAsDes,
        the_edges_origins: &mut EdgesOrigins,
        the_faces_origins: &mut ShapeShapeMap,
        the_e_trim_e_inf: &mut ShapeShapeMap,
        the_image: &mut BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut bf_tool = BuildOffsetFaces::new(the_image);
        bf_tool.set_faces(the_lf);
        bf_tool.set_as_des_info(the_as_des);
        bf_tool.set_analysis(&self.my_analyse);
        bf_tool.set_edges_origins(the_edges_origins);
        bf_tool.set_faces_origins(the_faces_origins);
        bf_tool.set_inf_edges(the_e_trim_e_inf);
        bf_tool.build_splits_of_extended_faces(the_range)
    }

    /// `BuildSplitsOfTrimmedFaces(theLF, theAsDes, theImage, Range)`: the splits of the
    /// trimmed offset faces (BRepOffset_BuildOffsetFaces).
    pub(super) fn build_splits_of_trimmed_faces(
        &mut self,
        the_lf: &List<rustycad_tkbrep::topods::Shape>,
        the_as_des: &BRepAlgoAsDes,
        the_image: &mut BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut bf_tool = BuildOffsetFaces::new(the_image);
        bf_tool.set_faces(the_lf);
        bf_tool.set_as_des_info(the_as_des);
        bf_tool.build_splits_of_trimmed_faces(the_range)
    }
}
