use super::{BOPDSIterator, DataStructure, Pair, interference_index, source_types};
use crate::bop_tools::{BoxSelector, BoxSet};
use rustycad_tkernel::{ncollection::Map, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    bnd::box3_to_bvh,
    bvh::{BvhBox, BvhSetTraverse, BvhVec3d},
};
impl BOPDSIterator {
    /// Return the native extended-list count (VV, VE, EE, VF); `BOPDSIterator::nb_ext_interfs() == 4`.
    pub const fn nb_ext_interfs() -> i32 {
        4
    }
    /// Append candidates for enlarged/mapped shapes and activate extended lists; `iterator.intersect_extended(&ds,&indices)?`.
    pub fn intersect_extended(
        &mut self,
        structure: &DataStructure,
        indices: &Map<i32>,
    ) -> Result<(), Failure> {
        self.shape_types = source_types(structure)?;
        let (mut boxes, queries) = extended_boxes(structure, indices)?;
        boxes.build()?;
        let mut fence = Vec::new();
        for (index, bounds) in queries {
            let candidates = select_extended(&mut boxes, bounds)?;
            for candidate in candidates {
                self.append_extended(structure, index, candidate, &mut fence)?;
            }
        }
        self.use_extended = true;
        Ok(())
    }
    fn append_extended(
        &mut self,
        structure: &DataStructure,
        first: i32,
        second: i32,
        fence: &mut Vec<Pair>,
    ) -> Result<(), Failure> {
        if structure.rank(first) == structure.rank(second)
            || self.is_contained_pair(structure, first, second)?
        {
            return Ok(());
        }
        let pair = Pair::new(first, second);
        if fence.contains(&pair) {
            return Ok(());
        }
        fence.push(pair);
        let first_type = structure.shape_info(first)?.shape_type();
        let second_type = structure.shape_info(second)?.shape_type();
        if let Some(index) = interference_index(first_type, second_type)
            && index < Self::nb_ext_interfs() as usize
        {
            self.extended_lists[index].push(pair);
        }
        Ok(())
    }
}
type ExtendedBoxes = (BoxSet<BvhVec3d>, Vec<(i32, BvhBox<BvhVec3d>)>);
fn extended_boxes(structure: &DataStructure, indices: &Map<i32>) -> Result<ExtendedBoxes, Failure> {
    let mut boxes = BoxSet::default();
    boxes.set_size(structure.nb_source_shapes() as usize);
    let mut queries = Vec::new();
    for index in 0..structure.nb_source_shapes() {
        let info = structure.shape_info(index)?;
        if !info.is_interfering() || info.shape_type() == ShapeEnum::Solid {
            continue;
        }
        let bounds = if indices.contains(&index) {
            let representative = structure.shape_sd(index).unwrap_or(index);
            let bounds = box3_to_bvh(structure.shape_info(representative)?.box_bounds())?;
            queries.push((index, bounds));
            bounds
        } else {
            box3_to_bvh(info.box_bounds())?
        };
        boxes.add(index, bounds);
    }
    Ok((boxes, queries))
}
fn select_extended(
    boxes: &mut BoxSet<BvhVec3d>,
    bounds: BvhBox<BvhVec3d>,
) -> Result<Vec<i32>, Failure> {
    let mut selector = BoxSelector::default();
    selector.set_bvh_set(Some(boxes));
    selector.set_box(bounds);
    selector.select()?;
    Ok(selector.indices().iter().copied().collect())
}
