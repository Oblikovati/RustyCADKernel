//! Native indexed-vertex ordering used before Delaunay insertion.
use super::BRepMeshDelaun;
use rustycad_tkernel::standard::Failure;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
struct ProjectedIndex {
    index: i32,
    projection: f64,
}

impl BRepMeshDelaun {
    /// Sort vertex IDs by OCCT's `(x + y)` heap order; `delaun.sort_vertex_indices(&mut [1,2])?`.
    #[doc(hidden)]
    pub fn sort_vertex_indices(&self, indices: &mut [i32]) -> Result<(), Failure> {
        let mut projected = indices
            .iter()
            .map(|index| {
                Ok(ProjectedIndex {
                    index: *index,
                    projection: self.get_vertex(*index)?.coord().x()
                        + self.get_vertex(*index)?.coord().y(),
                })
            })
            .collect::<Result<Vec<_>, Failure>>()?;
        make_native_heap(&mut projected);
        sort_native_heap(&mut projected);
        for (index, entry) in indices.iter_mut().zip(projected) {
            *index = entry.index;
        }
        Ok(())
    }
}

fn make_native_heap(entries: &mut [ProjectedIndex]) {
    if entries.len() < 2 {
        return;
    }
    let mut parent = (entries.len() - 2) / 2;
    loop {
        let value = entries[parent];
        adjust_heap(entries, parent, entries.len(), value);
        if parent == 0 {
            return;
        }
        parent -= 1;
    }
}

fn sort_native_heap(entries: &mut [ProjectedIndex]) {
    let mut end = entries.len();
    while end > 1 {
        end -= 1;
        let value = entries[end];
        entries[end] = entries[0];
        adjust_heap(entries, 0, end, value);
    }
}

fn adjust_heap(entries: &mut [ProjectedIndex], hole: usize, length: usize, value: ProjectedIndex) {
    let top = hole;
    let mut hole = hole;
    let mut child = 2 * (hole + 1);
    while child < length {
        if entries[child].projection < entries[child - 1].projection {
            child -= 1;
        }
        entries[hole] = entries[child];
        hole = child;
        child = 2 * (hole + 1);
    }
    if child == length {
        entries[hole] = entries[child - 1];
        hole = child - 1;
    }
    while hole > top {
        let parent = (hole - 1) / 2;
        if entries[parent].projection.partial_cmp(&value.projection) != Some(Ordering::Less) {
            break;
        }
        entries[hole] = entries[parent];
        hole = parent;
    }
    entries[hole] = value;
}
