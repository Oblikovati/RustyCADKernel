// SPDX-License-Identifier: GPL-2.0-only
//! Stable names for findings: the 1-based `TopExp::MapShapes` index of each sub-shape kind.
use super::report::TopologyRef;
use rustycad_tkbrep::topexp::map_shapes_of_type;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

const NAMED_KINDS: [ShapeEnum; 8] = [
    ShapeEnum::Compound,
    ShapeEnum::CompSolid,
    ShapeEnum::Solid,
    ShapeEnum::Shell,
    ShapeEnum::Face,
    ShapeEnum::Wire,
    ShapeEnum::Edge,
    ShapeEnum::Vertex,
];

/// The indexed sub-shapes of one root, per kind.
pub(super) struct TopologyNames {
    maps: Vec<(ShapeEnum, IndexedMapOfShape)>,
}

impl TopologyNames {
    /// Index every sub-shape kind of `root`; `TopologyNames::of(&solid)?`.
    pub(super) fn of(root: &Shape) -> Result<Self, Failure> {
        let mut maps = Vec::with_capacity(NAMED_KINDS.len());
        for kind in NAMED_KINDS {
            let mut map = IndexedMapOfShape::default();
            map_shapes_of_type(root, kind, &mut map)?;
            maps.push((kind, map));
        }
        Ok(Self { maps })
    }

    /// The name of `shape` (any orientation); no index when it is not under the root.
    pub(super) fn name(&self, shape: &Shape) -> TopologyRef {
        let kind = shape.shape_type().unwrap_or(ShapeEnum::Shape);
        let mapped = self.maps.iter().find(|(mapped, _)| *mapped == kind);
        let position = mapped.map_or(0, |(_, map)| map.find_index(shape));
        let index = (position > 0).then(|| (position - 1) as u32);
        TopologyRef { kind, index }
    }

    /// The indexed shapes of `kind`; `names.all(ShapeEnum::Edge)`.
    pub(super) fn all(&self, kind: ShapeEnum) -> Vec<Shape> {
        let Some((_, map)) = self.maps.iter().find(|(mapped, _)| *mapped == kind) else {
            return Vec::new();
        };
        (1..=map.extent())
            .filter_map(|index| map.find_key(index).ok().cloned())
            .collect()
    }
}
