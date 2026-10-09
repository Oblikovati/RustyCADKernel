// Rust translation of DetectUnclosedWire, CorrectGclosedWire, DetectPseudoInternalEdge,
// FUN_DetectVerticesOn1Edge and FUN_AnalyzemapVon1E of
// ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_FaceBuilder.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (RM_HANGING undefined, as natively).

//! The unclosed wires of a face builder: their detection, the correction of the
//! geometrically closed ones and the pseudo internal edges.
use super::TopOpeBRepBuildFaceBuilder;
use crate::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// A map of shapes to shapes (`NCollection_IndexedDataMap<TopoDS_Shape, TopoDS_Shape,
/// TopTools_ShapeMapHasher>`).
pub type ShapeToShapeMap = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// `ISUNKNOWN`: an undecided wire.
pub const ISUNKNOWN: i32 = -1;
/// `ISVERTEX`: a wire of a degenerated edge.
pub const ISVERTEX: i32 = 0;
/// `GCLOSEDW`: a geometrically (not topologically) closed wire.
pub const GCLOSEDW: i32 = 1;
/// `UNCLOSEDW`: an unclosed wire.
pub const UNCLOSEDW: i32 = 2;
/// `CLOSEDW`: a closed wire.
pub const CLOSEDW: i32 = 10;

/// `FUN_DetectVerticesOn1Edge(W, mapVon1E)`: the non INTERNAL vertices of `w` on a
/// single edge, neither INTERNAL nor EXTERNAL, bound to it;
/// `fun_detect_vertices_on1_edge(&w, &mut map)?`.
pub fn fun_detect_vertices_on1_edge(
    w: &Shape,
    map_von1e: &mut ShapeToShapeMap,
) -> Result<(), Failure> {
    let mut map_vedges = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(w, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map_vedges)?;
    for i in 1..=map_vedges.len() {
        let v = map_vedges.find_key(i)?;
        let loe = map_vedges.find_from_index(i)?;
        if v.orientation() == Orientation::Internal || loe.len() >= 2 {
            continue;
        }
        let e = loe.first().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!("FUN_DetectVerticesOn1Edge: vertex {i} has no edge; expected one"),
                "TopOpeBRepBuild_FaceBuilder",
            )
        })?;
        if matches!(
            e.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            continue;
        }
        map_von1e.add(v.clone(), e.clone());
    }
    Ok(())
}

/// `FUN_AnalyzemapVon1E(mapVon1E, mapVV)`: CLOSEDW without free vertex, for one
/// ISVERTEX (degenerated edge), CLOSEDW (closed edge) or UNCLOSEDW, else GCLOSEDW when
/// every free vertex has a coincident one (bound both ways in `map_vv`) and UNCLOSEDW
/// otherwise; `fun_analyzemap_von1e(&map, &mut vv)?`.
pub fn fun_analyzemap_von1e(
    map_von1e: &ShapeToShapeMap,
    map_vv: &mut ShapeToShapeMap,
) -> Result<i32, Failure> {
    let nv = map_von1e.len();
    if nv == 0 {
        return Ok(CLOSEDW);
    }
    if nv == 1 {
        let e = map_von1e.find_from_index(1)?;
        if BRepTool::degenerated(&topods::edge(e)?)? {
            return Ok(ISVERTEX);
        }
        return Ok(if BRepTool::is_closed(e)? {
            CLOSEDW
        } else {
            UNCLOSEDW
        });
    }
    pair_coincident_vertices(map_von1e, map_vv)?;
    Ok(if map_vv.len() == nv {
        GCLOSEDW
    } else {
        UNCLOSEDW
    })
}

/// The first later vertex of `map_von1e` at the point of each one (within
/// Precision::Confusion()), bound both ways in `map_vv`.
fn pair_coincident_vertices(
    map_von1e: &ShapeToShapeMap,
    map_vv: &mut ShapeToShapeMap,
) -> Result<(), Failure> {
    let tol = precision::confusion();
    let nv = map_von1e.len();
    for i in 1..=nv {
        let vi = map_von1e.find_key(i)?;
        let pi = BRepTool::pnt(&topods::vertex(vi)?)?;
        for j in i + 1..=nv {
            let vj = map_von1e.find_key(j)?;
            if pi.is_equal(&BRepTool::pnt(&topods::vertex(vj)?)?, tol) {
                map_vv.add(vi.clone(), vj.clone());
                map_vv.add(vj.clone(), vi.clone());
                break;
            }
        }
    }
    Ok(())
}

/// A new empty compound as a shape.
fn new_compound() -> Shape {
    let mut cmp = Compound::default();
    BRepBuilder::default().make_compound(&mut cmp);
    cmp.into_shape()
}

impl TopOpeBRepBuildFaceBuilder {
    /// The valid edges of the current (block) wire added to `cmp`.
    fn add_current_wire_edges(&mut self, cmp: &mut Shape) -> Result<(), Failure> {
        self.init_edge()?;
        while self.more_edge() {
            let e = self.edge()?.clone();
            self.add_edge_wire(&e, cmp)?;
            self.next_edge()?;
        }
        Ok(())
    }

    /// Every element of `w` invalidated in the block builder.
    fn invalidate_edges_of(&mut self, w: &Shape) -> Result<(), Failure> {
        for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let i = self.my_block_builder.element_index(&e)?;
            self.my_block_builder.set_valid(i, false);
        }
        Ok(())
    }

    /// The analysis of the current new wire: GCLOSEDW maps merged into the outputs,
    /// UNCLOSEDW edges invalidated.
    fn detect_unclosed_current_wire(
        &mut self,
        map_vv_same_g: &mut ShapeToShapeMap,
        map_von1_edge: &mut ShapeToShapeMap,
    ) -> Result<(), Failure> {
        let mut w = new_compound();
        self.add_current_wire_edges(&mut w)?;
        let mut map_von1e = ShapeToShapeMap::default();
        fun_detect_vertices_on1_edge(&w, &mut map_von1e)?;
        let mut map_vv = ShapeToShapeMap::default();
        match fun_analyzemap_von1e(&map_von1e, &mut map_vv)? {
            GCLOSEDW => {
                merge_into(&map_vv, map_vv_same_g)?;
                merge_into(&map_von1e, map_von1_edge)
            }
            UNCLOSEDW => self.invalidate_edges_of(&w),
            _ => Ok(()),
        }
    }

    /// `DetectUnclosedWire(mapVVsameG, mapVon1Edge)`: the new wires that are not
    /// closed removed from the block builder, and the vertex maps of the geometrically
    /// closed ones; `fb.detect_unclosed_wire(&mut vv, &mut von1e)?`.
    pub fn detect_unclosed_wire(
        &mut self,
        map_vv_same_g: &mut ShapeToShapeMap,
        map_von1_edge: &mut ShapeToShapeMap,
    ) -> Result<(), Failure> {
        map_vv_same_g.clear(false);
        map_von1_edge.clear(false);
        self.init_face();
        while self.more_face() {
            self.init_wire();
            while self.more_wire() {
                if !self.is_old_wire()? {
                    self.detect_unclosed_current_wire(map_vv_same_g, map_von1_edge)?;
                }
                self.next_wire();
            }
            self.next_face();
        }
        Ok(())
    }

    /// `CorrectGclosedWire(mapVVref, mapVon1Edge)`: on the edge of each vertex, the
    /// vertex replaced by its reference (with the vertex orientation and parameter);
    /// `fb.correct_gclosed_wire(&vv, &von1e)?`.
    pub fn correct_gclosed_wire(
        &self,
        map_vv_ref: &ShapeToShapeMap,
        map_von1_edge: &ShapeToShapeMap,
    ) -> Result<(), Failure> {
        for i in 1..=map_vv_ref.len() {
            let v = topods::vertex(map_vv_ref.find_key(i)?)?;
            let vref = map_vv_ref.find_from_index(i)?;
            if v.as_ref().is_same(vref) {
                continue;
            }
            let mut e = map_von1_edge.find_from_key(v.as_ref())?.clone();
            let paron_e = BRepTool::parameter(&v, &topods::edge(&e)?)?;
            let bb = BRepBuilder::default();
            e.set_free(true)?;
            bb.remove(&mut e, v.as_ref())?;
            let new_vref = vref.oriented(v.as_ref().orientation());
            bb.add(&mut e, &new_vref)?;
            TopOpeBRepDSBuildTool::new().parameter(&e, &new_vref, paron_e)?;
        }
        Ok(())
    }

    /// The edges of all the new wires of the builder in a compound.
    fn new_wires_compound(&mut self) -> Result<Shape, Failure> {
        let mut cmp = new_compound();
        self.init_face();
        while self.more_face() {
            self.init_wire();
            while self.more_wire() {
                if !self.is_old_wire()? {
                    self.add_current_wire_edges(&mut cmp)?;
                }
                self.next_wire();
            }
            self.next_face();
        }
        Ok(cmp)
    }

    /// `DetectPseudoInternalEdge(MapE)`: the edges of the new wires present twice with
    /// complementary orientations at a vertex of connexity 2, invalidated and kept in
    /// `map_e`; `fb.detect_pseudo_internal_edge(&mut map_e)?`.
    pub fn detect_pseudo_internal_edge(
        &mut self,
        map_e: &mut IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let cmp = self.new_wires_compound()?;
        let mut map_voe = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&cmp, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map_voe)?;
        map_e.clear(false);
        for i in 1..=map_voe.len() {
            let le: Vec<Shape> = map_voe.find_from_index(i)?.iter().cloned().collect();
            let [e1, e2] = le.as_slice() else {
                continue;
            };
            if e1.is_same(e2) && e1.orientation() == top_abs::complement(e2.orientation()) {
                map_e.add(e1.clone());
                self.invalidate_element(e1)?;
                self.invalidate_element(e2)?;
            }
        }
        Ok(())
    }

    /// The element `e` invalidated in the block builder.
    fn invalidate_element(&mut self, e: &Shape) -> Result<(), Failure> {
        let i = self.my_block_builder.element_index(e)?;
        self.my_block_builder.set_valid(i, false);
        Ok(())
    }
}

/// Every binding of `from` added to `to` (in index order).
fn merge_into(from: &ShapeToShapeMap, to: &mut ShapeToShapeMap) -> Result<(), Failure> {
    for i in 1..=from.len() {
        to.add(from.find_key(i)?.clone(), from.find_from_index(i)?.clone());
    }
    Ok(())
}
