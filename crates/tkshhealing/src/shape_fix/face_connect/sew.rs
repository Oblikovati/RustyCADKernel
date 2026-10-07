//! `sewConnectedFaces` of ShapeFix_FaceConnect: the resulting free edges of each pair
//! of connected faces sewn (each in its own wire of an empty copy of its face), and the
//! sewn edges moved to the resulting free or shared edges of their original free edge.
use super::{ShapeFixFaceConnect, ShapeListMap};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{FlatDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;

type ShapeShapeMap = FlatDataMap<Shape, Shape, ShapeMapHasher>;

/// The values of a list map entry (empty when the key is absent).
fn list_of(map: &ShapeListMap, key: &Shape) -> Vec<Shape> {
    map.seek(key)
        .map(|list| list.iter().cloned().collect())
        .unwrap_or_default()
}

/// Whether `first` is in the processed list of `second`.
fn already_processed(processed: &ShapeListMap, first: &Shape, second: &Shape) -> bool {
    processed
        .seek(second)
        .is_some_and(|list| list.iter().any(|face| first.is_same(face)))
}

impl ShapeFixFaceConnect {
    /// `sewConnectedFaces(theSewingTolerance)`.
    pub(super) fn sew_connected_faces(&mut self, tolerance: f64) -> Result<(), Failure> {
        if self.ori_free_edges.is_empty() {
            return Ok(());
        }
        let mut processed = ShapeListMap::default();
        let connected: Vec<(Shape, Vec<Shape>)> = self
            .connected
            .items()
            .map(|(face, list)| (face.clone(), list.iter().cloned().collect()))
            .collect();
        for (first, list) in connected {
            if self.ori_free_edges.seek(&first).is_none() {
                continue;
            }
            let mut faces = [first.clone(), Shape::default()];
            let mut count = 1;
            let mut processed_list = List::default();
            for second in list {
                if self.ori_free_edges.seek(&second).is_none() {
                    continue;
                }
                faces[1] = second.clone();
                processed_list.append(second.clone());
                if already_processed(&processed, &first, &second) {
                    continue;
                }
                // Native quirk kept: the count of faces to sew is never reset to 1.
                if !first.is_same(&second) {
                    count = 2;
                }
                self.sew_pair(&faces[..count], tolerance)?;
            }
            processed.try_bind(first, processed_list);
        }
        Ok(())
    }

    /// The sewing of the resulting free edges of the faces, then the results recorded.
    fn sew_pair(&mut self, faces: &[Shape], tolerance: f64) -> Result<(), Failure> {
        let mut wires = ShapeShapeMap::default();
        let mut sewer = BRepBuilderAPISewing::new(tolerance, true, true, true, false);
        for face in faces {
            let aux_face = self.free_edge_face(face, &mut wires, &mut sewer)?;
            sewer.add(&aux_face)?;
        }
        // A failure of the sewing skips the pair, as the native try/catch.
        let sewn =
            sewer.perform(&ProgressRange::default()).is_ok() && !sewer.sewed_shape().is_null();
        if !sewn {
            return Ok(());
        }
        let mut results = ShapeShapeMap::default();
        for face in faces {
            for original in list_of(&self.ori_free_edges, face) {
                self.record_sewn_edges(&original, &wires, &sewer, &mut results)?;
            }
        }
        for (edge, original) in results.items() {
            self.res_free_edges
                .change_find(original)?
                .append(edge.clone());
        }
        Ok(())
    }

    /// An empty copy of the face holding each resulting free edge of its original free
    /// edges in its own wire; each wire is also added to the sewer.
    fn free_edge_face(
        &self,
        face: &Shape,
        wires: &mut ShapeShapeMap,
        sewer: &mut BRepBuilderAPISewing,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut aux_face = face.empty_copied()?;
        for original in list_of(&self.ori_free_edges, face) {
            for edge in self.res_free_edges.find(&original)?.iter() {
                let mut wire = Wire::default();
                builder.make_wire(&mut wire);
                builder.add(&mut wire, edge)?;
                builder.add(&mut aux_face, &wire)?;
                wires.bound(edge.clone(), wire.clone().into_shape());
                sewer.add(wire.as_ref())?;
            }
        }
        Ok(aux_face)
    }

    /// The sewn edges of the wires of the resulting free edges of `original`: an edge met
    /// once is a free edge of it, met twice a shared edge of both original edges; the
    /// modified resulting free edges are removed.
    fn record_sewn_edges(
        &mut self,
        original: &Shape,
        wires: &ShapeShapeMap,
        sewer: &BRepBuilderAPISewing,
        results: &mut ShapeShapeMap,
    ) -> Result<(), Failure> {
        let old_free: Vec<Shape> = self
            .res_free_edges
            .find(original)?
            .iter()
            .cloned()
            .collect();
        let mut kept = List::default();
        for edge in old_free {
            let wire = wires.find(&edge)?.clone();
            if !sewer.is_modified(&wire) {
                kept.append(edge);
                continue;
            }
            for sewn in Explorer::new(&sewer.modified(&wire), ShapeEnum::Edge, ShapeEnum::Shape)? {
                if results.try_bind(sewn.clone(), original.clone()) {
                    continue;
                }
                // "Edge was shared - move in results list".
                let other = results.find(&sewn)?.clone();
                self.res_shar_edges
                    .change_find(&other)?
                    .append(sewn.clone());
                self.res_shar_edges
                    .change_find(original)?
                    .append(sewn.clone());
                results.unbind(&sewn);
            }
        }
        *self.res_free_edges.change_find(original)? = kept;
        Ok(())
    }
}
