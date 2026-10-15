// Rust translation of LocOpe_Gluer::Perform and AddEdges of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Gluer.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the basis split by the edges of the glued faces, the generator on the
//! split basis and the glued shape, the descendants of the faces, the regularities of
//! the glued shape recoded on the result and the list of the glued edges kept.
use super::Gluer;
use crate::errors::construction_error;
use crate::loc_ope::{
    GeneratedShape, Generator, GluedShape, Operation, Spliter, WiresOnShape, tgt_faces,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPF;

impl Gluer {
    /// The wires on the basis (the edges of the glued faces on their base faces, the
    /// bound edges on their base edges) and the glued shape on its glued faces.
    fn wires_and_glued(&self) -> Result<(WiresOnShape, GluedShape), Failure> {
        let mut the_w_on_s = WiresOnShape::new(&self.sb);
        let mut the_gs = GluedShape::new(&self.sn);
        for ind in 1..=self.map_ef.extent() {
            let s = self.map_ef.find_key(ind)?;
            if s.shape_type()? == ShapeEnum::Edge {
                let s2 = self.map_ef.find_from_index(ind)?;
                if !s2.is_null() {
                    the_w_on_s.bind_edge(&topods::edge(s)?, &topods::face(s2)?)?;
                }
            } else {
                // TopAbs_FACE
                the_gs.glue_on_face(&topods::face(s)?)?;
            }
        }
        for (key, value) in self.map_ee.items() {
            the_w_on_s.bind_edge_on_edge(&topods::edge(key)?, &topods::edge(value)?);
        }
        Ok((the_w_on_s, the_gs))
    }

    /// Mise a jour des descendants: the split faces of the basis, and the faces of the
    /// glued shape that stay.
    fn split_descendants(
        &mut self,
        the_split: &Spliter,
        the_gs: &mut GluedShape,
    ) -> Result<(), Failure> {
        for f in Explorer::new(&self.sb, ShapeEnum::Face, ShapeEnum::Shape)? {
            let desc = the_split.descendant_shapes(&f)?;
            self.desc_f.bind(f, desc);
        }
        for f in Explorer::new(&self.sn, ShapeEnum::Face, ShapeEnum::Shape)? {
            let mut thelist = List::default();
            if the_gs.oriented_faces()?.iter().any(|x| x.is_same(&f)) {
                thelist.append(f.clone());
            }
            self.desc_f.bind(f, thelist);
        }
        Ok(())
    }

    /// The descendants through the generator: the faces each one became (the vanished
    /// faces dropped).
    fn generated_descendants(&mut self, the_gen: &Generator) -> Result<(), Failure> {
        for (_, desc) in self.desc_f.items_mut() {
            let mut new_desc = List::default();
            for f in desc.iter() {
                for d in the_gen.descendant_face(&topods::face(f)?)?.iter() {
                    if !d.is_null() {
                        // sinon la face a disparu
                        new_desc.append(d.clone());
                    }
                }
            }
            *desc = new_desc;
        }
        Ok(())
    }

    /// Recodage des regularites: the G1 regularities of the glued shape on the edges
    /// shared by two other faces in the result.
    fn recode_regularities(
        &self,
        the_map_ef2: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<(), Failure> {
        let mut the_map_ef1 = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.sn,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut the_map_ef1,
        )?;
        for ind in 1..=the_map_ef1.extent() {
            let edg = topods::edge(the_map_ef1.find_key(ind)?)?;
            let ll = the_map_ef1.find_from_index(ind)?;
            if ll.len() != 2 {
                continue;
            }
            let fac1 = topods::face(ll.first().unwrap_or(&Shape::default()))?;
            let fac2 = topods::face(ll.last().unwrap_or(&Shape::default()))?;
            let thecont = BRepTool::continuity_on_faces(&edg, &fac1, &fac2)?;
            if thecont < Continuity::G1 {
                continue;
            }
            // on essaie de recoder
            let ind2 = the_map_ef2.find_index(&edg);
            if ind2 == 0 {
                continue;
            }
            let ll2 = the_map_ef2.find_from_index(ind2)?;
            if ll2.len() != 2 {
                continue;
            }
            let ff1 = topods::face(ll2.first().unwrap_or(&Shape::default()))?;
            let ff2 = topods::face(ll2.last().unwrap_or(&Shape::default()))?;
            let same = (ff1.is_same(&fac1) && ff2.is_same(&fac2))
                || (ff1.is_same(&fac2) && ff2.is_same(&fac1));
            if !same {
                BRepBuilder::default().continuity_on_faces(&edg, &ff1, &ff2, thecont)?;
            }
        }
        Ok(())
    }

    /// Creation de la liste d`edge: the edges of the wires kept in the result; those
    /// between tangent faces get at least the G1 regularity.
    fn collect_edges(
        &mut self,
        the_w_on_s: &mut WiresOnShape,
        the_map_ef2: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<(), Failure> {
        the_w_on_s.init_edge_iterator()?;
        while the_w_on_s.more_edge() {
            let edg = the_w_on_s.edge()?;
            for ind in 1..=the_map_ef2.extent() {
                if !the_map_ef2.find_key(ind)?.is_same(&edg) {
                    continue;
                }
                self.edges.append(edg.as_ref().clone());
                // recodage eventuel des regularites sur cet edge
                let l = the_map_ef2.find_from_index(ind)?;
                if l.len() != 2 {
                    continue;
                }
                let fac1 = topods::face(l.first().unwrap_or(&Shape::default()))?;
                let fac2 = topods::face(l.last().unwrap_or(&Shape::default()))?;
                if tgt_faces(&edg, &fac1, &fac2)? {
                    self.tgt_edges.append(edg.as_ref().clone());
                    if BRepTool::continuity_on_faces(&edg, &fac1, &fac2)? < Continuity::G1 {
                        BRepBuilder::default().continuity_on_faces(
                            &edg,
                            &fac1,
                            &fac2,
                            Continuity::G1,
                        )?;
                    }
                }
            }
            the_w_on_s.next_edge();
        }
        Ok(())
    }

    /// `Perform()`: glues the new shape on the basis; Standard_ConstructionError without
    /// shapes, bindings or with an invalid operation; not done when the split or the
    /// generation fails; `gluer.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        if self.sb.is_null()
            || self.sn.is_null()
            || self.map_ef.is_empty()
            || self.ope == Operation::Invalid
        {
            return Err(construction_error(
                "LocOpe_Gluer::Perform",
                "missing shapes, face bindings or a valid operation",
            ));
        }
        let (mut the_w_on_s, mut the_gs) = self.wires_and_glued()?;
        the_w_on_s.bind_all()?;
        if !the_w_on_s.is_done() {
            return Ok(());
        }
        let mut the_split = Spliter::with_shape(&self.sb);
        the_split.perform(&mut the_w_on_s)?;
        if !the_split.is_done() {
            return Ok(());
        }
        self.split_descendants(&the_split, &mut the_gs)?;
        let mut the_gen = Generator::with_shape(the_split.resulting_shape()?);
        the_gen.perform(&mut the_gs)?;
        self.done = the_gen.is_done();
        if self.done {
            self.res = the_gen.resulting_shape()?.clone();
            self.add_edges()?;
            self.generated_descendants(&the_gen)?;
        }
        let mut the_map_ef2 = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.res,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut the_map_ef2,
        )?;
        self.recode_regularities(&the_map_ef2)?;
        self.collect_edges(&mut the_w_on_s, &the_map_ef2)
    }

    /// Whether every vertex of `e` outside the face `face` (of vertices `map_v`) lies
    /// on it within its tolerance (no side effect natively: the flag is not used).
    fn vertices_on_face(
        e: &Shape,
        face: &Shape,
        map_v: &IndexedMapOfShape,
    ) -> Result<bool, Failure> {
        let face = topods::face(face)?;
        let mut ext = BRepExtremaExtPF::new();
        ext.initialize(&face, ExtFlag::MinMax, ExtAlgo::Grad)?;
        let mut flag = false;
        for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if map_v.contains(&v) {
                flag = true;
                continue;
            }
            let vtx = topods::vertex(&v)?;
            ext.perform(&vtx, &face)?;
            if !ext.is_done() || ext.nb_ext() == 0 {
                return Ok(false);
            }
            let mut dist2min = ext.square_distance(1)?;
            for i in 2..=ext.nb_ext() {
                dist2min = dist2min.min(ext.square_distance(i)?);
            }
            let tol = BRepTool::vertex_tolerance(&vtx)?;
            if dist2min >= tol * tol {
                return Ok(false);
            }
            flag = true;
        }
        Ok(flag)
    }

    /// `AddEdges()`: examines the edges of the new shape touching the faces of the
    /// result that are not its own; the native body computes the vertex distances and
    /// keeps nothing (its last `if (flag == 1) {}` is empty), the evaluations are kept.
    pub(super) fn add_edges(&self) -> Result<(), Failure> {
        let mut map_f_prism = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&self.sn, ShapeEnum::Face, &mut map_f_prism)?;
        let mut map_e = IndexedMapOfShape::default();
        for face in Explorer::new(&self.res, ShapeEnum::Face, ShapeEnum::Shape)? {
            if map_f_prism.contains(&face) {
                continue;
            }
            let mut map_v = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(&face, ShapeEnum::Vertex, &mut map_v)?;
            topexp::map_shapes_of_type(&face, ShapeEnum::Edge, &mut map_e)?;
            for e in Explorer::new(&self.sn, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if map_e.contains(&e) {
                    continue;
                }
                let touches = Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)?
                    .any(|v| map_v.contains(&v));
                if touches {
                    Self::vertices_on_face(&e, &face, &map_v)?;
                }
            }
        }
        Ok(())
    }
}
