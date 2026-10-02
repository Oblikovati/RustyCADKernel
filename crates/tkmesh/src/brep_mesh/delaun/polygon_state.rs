//! Coupled native sequences preserve one-based link/box edits.
use super::BRepMeshDelaun;
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkmath::{bnd::CompactBox2, gp::Xy};
#[derive(Default)]
pub(super) struct PolygonLoop {
    pub(super) links: Sequence<i32>,
    pub(super) boxes: Sequence<CompactBox2>,
}
pub(super) fn link_box(a: Xy, b: Xy) -> CompactBox2 {
    let mut bounds = CompactBox2::default();
    bounds.add_xy(a);
    bounds.add_xy(b);
    bounds.enlarge(rustycad_tkernel::precision::p_confusion());
    bounds
}
impl PolygonLoop {
    pub(super) fn clear(&mut self) {
        self.links.clear(None);
        self.boxes.clear(None);
    }
    pub(super) fn len(&self) -> usize {
        self.links.length() as usize
    }
    pub(super) fn append(&mut self, index: i32, bounds: CompactBox2) {
        self.links.append(index);
        self.boxes.append(bounds);
    }
    pub(super) fn prepend(&mut self, index: i32, bounds: CompactBox2) {
        self.links.prepend(index);
        self.boxes.prepend(bounds);
    }
    pub(super) fn replace(
        &mut self,
        index: usize,
        edge: i32,
        bounds: CompactBox2,
    ) -> Result<(), Failure> {
        self.links.set_value(index, edge)?;
        self.boxes.set_value(index, bounds)
    }
    pub(super) fn remove(&mut self, first: usize, last: usize) -> Result<(), Failure> {
        self.links.remove_range(first, last)?;
        self.boxes.remove_range(first, last)
    }
    pub(super) fn split(&mut self, index: usize) -> Result<Self, Failure> {
        let mut cut = Self::default();
        self.links.split(index, &mut cut.links)?;
        self.boxes.split(index, &mut cut.boxes)?;
        Ok(cut)
    }
}
impl BRepMeshDelaun {
    pub(super) fn boxed_polygon(&self, links: &Sequence<i32>) -> Result<PolygonLoop, Failure> {
        let mut polygon = PolygonLoop {
            links: links.clone(),
            boxes: Sequence::default(),
        };
        for index in links.iter() {
            let [a, b] = self.polygon_edge_points(*index)?;
            polygon.boxes.append(link_box(a, b));
        }
        Ok(polygon)
    }
}
