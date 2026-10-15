// Rust translation of BRepOffset_MakeOffset::Generated, Modified and IsDeleted of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The history of the offset algorithm.
use super::BRepOffsetMakeOffset;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BRepOffsetMakeOffset {
    /// The result map filled on the first query.
    fn res_map(&mut self) {
        if self.my_res_map.is_empty() {
            topexp::map_shapes_into_map(&self.my_offset_shape, &mut self.my_res_map, true, true);
        }
    }

    /// `Generated(theS)`: the shapes of the result generated from the shape: for a
    /// vertex, the first vertex shared by the images of its edges (then, falling
    /// through as natively, the edge and face cases); for an edge its last offset
    /// images; for a face (not a cap) its last offset images (reversed with caps); for
    /// the solid the result; only the shapes in the result are kept;
    /// `m.generated(&shape)?`.
    pub fn generated(&mut self, the_s: &Shape) -> Result<&List<Shape>, Failure> {
        self.my_generated.clear();
        let a_type = the_s.shape_type()?;
        if a_type == ShapeEnum::Vertex && self.my_analyse.has_ancestor(the_s) {
            let mut a_m_fence = MapOfShape::default();
            for a_e in self.my_analyse.ancestors(the_s)?.to_vec() {
                if !self.my_generated.is_empty() {
                    break;
                }
                if !self.my_init_offset_edge.has_image(&a_e) {
                    continue;
                }
                for e_im in self.my_init_offset_edge.last_image(&a_e) {
                    if !self.my_generated.is_empty() {
                        break;
                    }
                    let mut it_v = ShapeIterator::new(&e_im, true, true);
                    while it_v.more() {
                        let v = it_v.value()?.clone();
                        it_v.next();
                        if !a_m_fence.add(v.clone()) {
                            self.my_generated.append(v);
                            break;
                        }
                    }
                }
            }
        }
        if matches!(a_type, ShapeEnum::Vertex | ShapeEnum::Edge)
            && self.my_init_offset_edge.has_image(the_s)
        {
            for s in self.my_init_offset_edge.last_image(the_s) {
                self.my_generated.append(s);
            }
        }
        if matches!(
            a_type,
            ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face
        ) {
            let a_s = self
                .my_face_planface_map
                .seek(the_s)
                .cloned()
                .unwrap_or_else(|| the_s.clone());
            if !self.my_faces.contains(&a_s) && self.my_init_offset_face.has_image(&a_s) {
                for s in self.my_init_offset_face.last_image(&a_s) {
                    self.my_generated.append(s);
                }
                if !self.my_faces.is_empty() {
                    // The generated shapes reversed for small solids, useful only for
                    // the faces without influence on the others.
                    // The whole list, as natively.
                    for s in self.my_generated.iter_mut() {
                        s.reverse();
                    }
                }
            }
        } else if a_type == ShapeEnum::Solid && the_s.is_same(&self.my_shape) {
            self.my_generated.append(self.my_offset_shape.clone());
        }
        self.res_map();
        let kept: Vec<Shape> = self
            .my_generated
            .iter()
            .filter(|s| self.my_res_map.contains(s))
            .cloned()
            .collect();
        self.my_generated.clear();
        for s in kept {
            self.my_generated.append(s);
        }
        Ok(&self.my_generated)
    }

    /// `Modified(theShape)`: the last offset images of a cap face (reversed);
    /// `m.modified(&face)?`.
    pub fn modified(&mut self, the_shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.my_generated.clear();
        if the_shape.shape_type()? == ShapeEnum::Face {
            let a_s = self
                .my_face_planface_map
                .seek(the_shape)
                .cloned()
                .unwrap_or_else(|| the_shape.clone());
            if self.my_faces.contains(&a_s) && self.my_init_offset_face.has_image(&a_s) {
                for s in self.my_init_offset_face.last_image(&a_s) {
                    self.my_generated.append(s);
                }
                if !self.my_faces.is_empty() {
                    // The generated shapes reversed for small solids.
                    for s in self.my_generated.iter_mut() {
                        s.reverse();
                    }
                }
            }
        }
        Ok(&self.my_generated)
    }

    /// `IsDeleted(theS)`: false for a shape of the result, else whether it has neither
    /// generated nor modified shapes; `m.is_deleted(&shape)?`.
    pub fn is_deleted(&mut self, the_s: &Shape) -> Result<bool, Failure> {
        self.res_map();
        if self.my_res_map.contains(the_s) {
            return Ok(false);
        }
        Ok(self.generated(the_s)?.is_empty() && self.modified(the_s)?.is_empty())
    }
}
