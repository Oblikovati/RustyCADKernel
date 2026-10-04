use super::{Relations, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::{
    ncollection::{DataMap, List},
    standard::Failure,
};
pub(super) fn collect_common_descendants(
    relations: &Relations,
    first: &Shape,
    second: &Shape,
) -> Vec<Shape> {
    let mut common = Vec::new();
    let Some(descendants) = relations.down.seek(first) else {
        return common;
    };
    for descendant in descendants.iter() {
        if has_ancestor(relations, descendant, second) {
            common.push(descendant.clone());
        }
    }
    common
}
fn has_ancestor(relations: &Relations, shape: &Shape, expected: &Shape) -> bool {
    relations
        .up
        .seek(shape)
        .is_some_and(|ancestors| ancestors.iter().any(|ancestor| ancestor.is_same(expected)))
}
impl Relations {
    pub(super) fn replace(&mut self, old: &Shape, new: &Shape) {
        replace_map_links(&mut self.up, &mut self.down, old, new);
        replace_map_links(&mut self.down, &mut self.up, old, new);
    }
    pub(super) fn remove(&mut self, shape: &Shape) -> Result<(), Failure> {
        if self.down.is_bound(shape) {
            return Err(invalid_remove(shape, "shape has descendants"));
        }
        let Some(ancestors) = self.up.seek(shape) else {
            return Err(invalid_remove(shape, "shape has no ascendant"));
        };
        let ancestors = ancestors.iter().cloned().collect::<Vec<_>>();
        for ancestor in ancestors {
            if let Some(descendants) = self.down.change_seek(&ancestor) {
                remove_first_same(descendants, shape);
            }
        }
        self.up.unbind(shape);
        Ok(())
    }
}
fn replace_map_links(
    map: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    reverse: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    old: &Shape,
    new: &Shape,
) {
    let Some(old_values) = map.seek(old) else {
        return;
    };
    let linked = old_values.iter().cloned().collect::<Vec<_>>();
    for key in &linked {
        if let Some(values) = reverse.change_seek(key) {
            replace_in_list(values, old, new);
        }
    }
    merge_replacement(map, old, new);
}
fn replace_in_list(list: &mut List<Shape>, old: &Shape, new: &Shape) {
    let original = list.iter().cloned().collect::<Vec<_>>();
    let mut seen = original.clone();
    let mut rewritten = List::default();
    for shape in original {
        if !shape.is_same(old) {
            rewritten.append(shape);
            continue;
        }
        let replacement = new.oriented(shape.orientation());
        if !seen.iter().any(|known| known.is_equal(&replacement)) {
            seen.push(replacement.clone());
            rewritten.append(replacement);
        }
    }
    *list = rewritten;
}
fn merge_replacement(
    map: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    old: &Shape,
    new: &Shape,
) {
    let Some(old_values) = map.seek(old) else {
        return;
    };
    let old_values = old_values.clone();
    merge_map_values(map, old_values, new);
    map.unbind(old);
}
fn merge_map_values(
    map: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    old_values: List<Shape>,
    new: &Shape,
) {
    if !map.is_bound(new) {
        map.bind(new.clone(), unique_values(&old_values));
        return;
    }
    merge_without_duplicates(map, &old_values, new);
}
fn unique_values(values: &List<Shape>) -> List<Shape> {
    let mut unique: List<Shape> = List::default();
    for value in values.iter() {
        if !unique.iter().any(|known| known.is_equal(value)) {
            unique.append(value.clone());
        }
    }
    unique
}
fn merge_without_duplicates(
    map: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    old_values: &List<Shape>,
    new: &Shape,
) {
    let merged = map.change_seek(new).expect("new key checked above");
    for value in old_values.iter() {
        if !merged.iter().any(|present| present.is_equal(value)) {
            merged.append(value.clone());
        }
    }
}
fn remove_first_same(list: &mut List<Shape>, shape: &Shape) {
    let mut retained = false;
    list.retain(|candidate| {
        if !retained && candidate.is_same(shape) {
            retained = true;
            false
        } else {
            true
        }
    });
}
fn invalid_remove(shape: &Shape, reason: &str) -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        &format!("BRepAlgo_AsDes::Remove cannot remove {shape:?}: {reason}"),
        "",
    )
}
