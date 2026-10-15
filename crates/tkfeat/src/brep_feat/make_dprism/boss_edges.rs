// Rust translation of BRepFeat_MakeDPrism::BossEdges, TopEdges and LatEdges of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeDPrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges of a boss: those of its top (or bottom) faces, and the lateral edges, the
//! edges of the side faces touching the top not shared by two of them.
use super::BRepFeatMakeDPrism;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Whether the edges `e1` and `e2` share a vertex.
fn share_vertex(e1: &Edge, e2: &Edge) -> bool {
    let (v1, v2) = topexp::vertices(e1, false);
    let (vt1, vt2) = topexp::vertices(e2, false);
    v1.is_same(&vt1) || v1.is_same(&vt2) || v2.is_same(&vt1) || v2.is_same(&vt2)
}

/// Each edge of `ff` toggled in `map_e` (added when absent, removed when present).
fn toggle_edges(ff: &Shape, map_e: &mut MapOfShape) -> Result<(), Failure> {
    for e3 in Explorer::new(ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if map_e.contains(&e3) {
            map_e.remove(&e3);
        } else {
            map_e.add(e3);
        }
    }
    Ok(())
}

impl BRepFeatMakeDPrism {
    /// Whether the face `ff` (not one of the top faces `last`) has an edge touching a top
    /// edge.
    fn touches_top(&self, ff: &Shape, last: &List<Shape>) -> Result<bool, Failure> {
        for top_face in last.iter() {
            if ff.is_same(top_face) {
                continue;
            }
            for e1 in Explorer::new(ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let e1 = topods::edge(&e1)?;
                for e2 in self.top_edges.iter() {
                    if share_vertex(&e1, &topods::edge(e2)?) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// The lateral edges of a boss: the edges of the faces touching the top met once,
    /// without the top edges and the degenerated ones.
    fn lateral_edges(&mut self, last: &List<Shape>) -> Result<(), Failure> {
        let shape = self.form.base.my_shape.clone();
        if shape.is_null() {
            return Ok(());
        }
        let mut map_e = MapOfShape::default();
        for ff in Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.touches_top(&ff, last)? {
                toggle_edges(&ff, &mut map_e)?;
            }
        }
        for e in self.top_edges.iter() {
            if map_e.contains(e) {
                map_e.remove(e);
            }
        }
        for e in map_e.iter() {
            if !BRepTool::degenerated(&topods::edge(e)?)? {
                self.lat_edges.append(e.clone());
            }
        }
        Ok(())
    }

    /// `BossEdges(signature)`: the top edges from the faces of the first shape
    /// (`signature` 1 or -1) or of the last shape (2 or -2), and the lateral edges: the
    /// new edges for a negative signature, else those of the faces touching the top;
    /// `dprism.boss_edges(2)?`.
    pub fn boss_edges(&mut self, signature: i32) -> Result<(), Failure> {
        let the_last_shape = match signature {
            1 | -1 => self.form.first_shape()?,
            2 | -2 => self.form.last_shape()?,
            _ => return Ok(()),
        };
        // Edges Top
        for ff in the_last_shape.iter() {
            let ff = topods::face(ff)?;
            for ee in Explorer::new(&ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
                self.top_edges.append(topods::edge(&ee)?.into_shape());
            }
        }
        // Edges Bottom
        if signature < 0 {
            // Attention check if TgtEdges is important
            self.lat_edges = self.form.new_edges().clone();
        } else if signature > 0 {
            self.lateral_edges(&the_last_shape)?;
        }
        Ok(())
    }

    /// `TopEdges()`: the edges of the top of the boss; `dprism.top_edges()`.
    pub fn top_edges(&self) -> &List<Shape> {
        &self.top_edges
    }

    /// `LatEdges()`: the lateral edges of the boss; `dprism.lat_edges()`.
    pub fn lat_edges(&self) -> &List<Shape> {
        &self.lat_edges
    }
}
