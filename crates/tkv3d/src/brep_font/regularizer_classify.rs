//! `BRepFont_Regularizer::classifyLoops`: nests the loops by containment, applies the fill
//! rule, keeps the boundary loops and packs the material regions with their holes.
use super::planar_region::{
    BRepFontCurveId, BRepFontLoopId, BRepFontPlanarRegion, BRepFontPlanarRegionLoop,
};
use super::planar_region::{BRepFontPlanarRegionRegion, BRepFontPlanarRegionUse, BRepFontVertexId};
use super::regularizer_append::index;
use super::regularizer_intersect::loop_pair_key;
use super::regularizer_scanline::{contains, interior_point};
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::{FontGlyphOutline, FontGlyphOutlineFillRule};
use std::collections::HashSet;

/// `LoopInfo::Role`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum LoopRole {
    #[default]
    None,
    Outer,
    Hole,
}

/// `LoopInfo`: the containment parent, a point inside, the winding and the fill state.
#[derive(Clone, Copy, Debug, Default)]
struct LoopInfo {
    parent: Option<u32>,
    interior_point: Xy,
    winding: i32,
    is_filled: bool,
    role: LoopRole,
}

const INVALID_INDEX: u32 = u32::MAX;

/// `classifyLoops`: false when a loop has no interior point or an index overflows.
pub(super) fn classify_loops(
    outline: &FontGlyphOutline,
    region: &mut BRepFontPlanarRegion,
    intersecting_pairs: &HashSet<u64>,
) -> bool {
    let Some(mut infos) = interior_points(region) else {
        return false;
    };
    let order = area_order(region);
    for (position, &loop_index) in order.iter().enumerate() {
        nest_loop(
            outline,
            region,
            intersecting_pairs,
            &order[..position],
            loop_index,
            &mut infos,
        );
    }
    let holes = hole_bindings(&infos);
    let compact = infos.iter().any(|info| info.role == LoopRole::None);
    let (mut boundary, loop_map) = if compact {
        let Some(compacted) = compact_boundary(region, &infos) else {
            return false;
        };
        compacted
    } else {
        (std::mem::take(region), Vec::new())
    };
    if !pack_regions(
        &mut boundary,
        &infos,
        &holes,
        compact.then_some(&loop_map[..]),
    ) {
        return false;
    }
    *region = boundary;
    true
}

fn interior_points(region: &BRepFontPlanarRegion) -> Option<Vec<LoopInfo>> {
    region
        .loops()
        .iter()
        .map(|loop_| {
            Some(LoopInfo {
                interior_point: interior_point(region, loop_)?,
                ..LoopInfo::default()
            })
        })
        .collect()
}

// Loop indices by decreasing absolute area, ties by index.
fn area_order(region: &BRepFontPlanarRegion) -> Vec<u32> {
    let loops = region.loops().as_slice();
    let mut order: Vec<u32> = (0..loops.len() as u32).collect();
    order.sort_by(|&left, &right| {
        let left_area = loops[left as usize].signed_area().abs();
        let right_area = loops[right as usize].signed_area().abs();
        if left_area == right_area {
            left.cmp(&right)
        } else {
            right_area
                .partial_cmp(&left_area)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    order
}

// The parent is the nearest larger loop (scanning back in area order) that contains the
// loop's interior point and does not intersect it; then winding, fill and role follow.
fn nest_loop(
    outline: &FontGlyphOutline,
    region: &BRepFontPlanarRegion,
    intersecting_pairs: &HashSet<u64>,
    larger: &[u32],
    loop_index: u32,
    infos: &mut [LoopInfo],
) {
    let loops = region.loops().as_slice();
    let loop_ = &loops[loop_index as usize];
    let point = infos[loop_index as usize].interior_point;
    infos[loop_index as usize].parent = larger.iter().rev().copied().find(|&candidate| {
        let other = &loops[candidate as usize];
        !(other.signed_area().abs() <= loop_.signed_area().abs()
            || intersecting_pairs.contains(&loop_pair_key(loop_index, candidate))
            || other.bounds().is_out_box(loop_.bounds())
            || !contains(region, other, &point))
    });
    let (parent_winding, parent_filled) =
        infos[loop_index as usize]
            .parent
            .map_or((0, false), |parent| {
                (
                    infos[parent as usize].winding,
                    infos[parent as usize].is_filled,
                )
            });
    let info = &mut infos[loop_index as usize];
    info.winding = parent_winding + if loop_.signed_area() > 0.0 { 1 } else { -1 };
    info.is_filled = match outline.rule() {
        FontGlyphOutlineFillRule::EvenOdd => !parent_filled,
        FontGlyphOutlineFillRule::NonZero => info.winding != 0,
    };
    if !parent_filled && info.is_filled {
        info.role = LoopRole::Outer;
    } else if parent_filled && !info.is_filled {
        info.role = LoopRole::Hole;
    }
}

// Each hole loop bound to its nearest outer ancestor, sorted by (outer, hole).
fn hole_bindings(infos: &[LoopInfo]) -> Vec<(u32, u32)> {
    let mut holes = Vec::new();
    for (loop_index, info) in infos.iter().enumerate() {
        if info.role != LoopRole::Hole {
            continue;
        }
        let mut ancestor = info.parent;
        while let Some(current) =
            ancestor.filter(|&current| infos[current as usize].role != LoopRole::Outer)
        {
            ancestor = infos[current as usize].parent;
        }
        if let Some(outer) = ancestor {
            holes.push((outer, loop_index as u32));
        }
    }
    holes.sort_unstable();
    holes
}

/// The old-to-new index maps of a compaction.
struct CompactMaps {
    vertices: Vec<u32>,
    curves: Vec<u32>,
    loops: Vec<u32>,
}

// A new region with only the boundary loops (and the vertices and curves they use).
fn compact_boundary(
    region: &BRepFontPlanarRegion,
    infos: &[LoopInfo],
) -> Option<(BRepFontPlanarRegion, Vec<u32>)> {
    let mut maps = CompactMaps {
        vertices: vec![INVALID_INDEX; region.vertices().len()],
        curves: vec![INVALID_INDEX; region.curves().len()],
        loops: vec![INVALID_INDEX; region.loops().len()],
    };
    let mut boundary = BRepFontPlanarRegion {
        units_per_em: region.units_per_em,
        tolerance: region.tolerance,
        is_single_stroke: region.is_single_stroke,
        ..BRepFontPlanarRegion::default()
    };
    for (loop_index, info) in infos.iter().enumerate() {
        if info.role != LoopRole::None {
            copy_loop(region, loop_index, &mut boundary, &mut maps)?;
        }
    }
    Some((boundary, maps.loops))
}

fn copy_loop(
    region: &BRepFontPlanarRegion,
    loop_index: usize,
    boundary: &mut BRepFontPlanarRegion,
    maps: &mut CompactMaps,
) -> Option<()> {
    let old_loop = &region.loops().as_slice()[loop_index];
    let new_loop = index(boundary.loops.len()).ok()?;
    let first_use = index(boundary.uses.len()).ok()?;
    maps.loops[loop_index] = new_loop;
    for offset in 0..old_loop.nb_uses() {
        let old_use = &region.uses().as_slice()[(old_loop.first_use() + offset) as usize];
        let new_use = copy_use(region, old_use, boundary, maps)?;
        boundary.uses.append(new_use);
    }
    let after_last_use = index(boundary.uses.len()).ok()?;
    boundary.loops.append(BRepFontPlanarRegionLoop::new(
        first_use,
        after_last_use - first_use,
        old_loop.signed_area(),
        old_loop.bounds().clone(),
    ));
    Some(())
}

fn copy_use(
    region: &BRepFontPlanarRegion,
    old_use: &BRepFontPlanarRegionUse,
    boundary: &mut BRepFontPlanarRegion,
    maps: &mut CompactMaps,
) -> Option<BRepFontPlanarRegionUse> {
    let curve = old_use.curve_index().value() as usize;
    let (first, last) = (
        old_use.first_vertex().value() as usize,
        old_use.last_vertex().value() as usize,
    );
    if curve >= maps.curves.len() || first >= maps.vertices.len() || last >= maps.vertices.len() {
        return None;
    }
    if maps.curves[curve] == INVALID_INDEX {
        maps.curves[curve] = index(boundary.curves.len()).ok()?;
        boundary.curves.append(region.curves().as_slice()[curve]);
    }
    for vertex in [first, last] {
        if maps.vertices[vertex] == INVALID_INDEX {
            maps.vertices[vertex] = index(boundary.vertices.len()).ok()?;
            boundary
                .vertices
                .append(region.vertices().as_slice()[vertex]);
        }
    }
    Some(BRepFontPlanarRegionUse::new(
        BRepFontCurveId::new(maps.curves[curve]),
        BRepFontVertexId::new(maps.vertices[first]),
        BRepFontVertexId::new(maps.vertices[last]),
        old_use.first_parameter(),
        old_use.last_parameter(),
        old_use.is_reversed(),
    ))
}

// One region per outer loop (in old loop order) with its packed holes.
fn pack_regions(
    boundary: &mut BRepFontPlanarRegion,
    infos: &[LoopInfo],
    holes: &[(u32, u32)],
    loop_map: Option<&[u32]>,
) -> bool {
    let map = |old: u32| loop_map.map_or(old, |map| map[old as usize]);
    let mut hole_index = 0;
    for (old_loop, info) in infos.iter().enumerate() {
        let new_outer = map(old_loop as u32);
        if info.role != LoopRole::Outer || new_outer == INVALID_INDEX {
            continue;
        }
        let Ok(first_hole) = index(boundary.hole_loops.len()) else {
            return false;
        };
        while hole_index < holes.len() && holes[hole_index].0 == old_loop as u32 {
            let new_hole = map(holes[hole_index].1);
            if new_hole == INVALID_INDEX {
                return false;
            }
            boundary.hole_loops.append(BRepFontLoopId::new(new_hole));
            hole_index += 1;
        }
        let Ok(after_last_hole) = index(boundary.hole_loops.len()) else {
            return false;
        };
        boundary.regions.append(BRepFontPlanarRegionRegion::new(
            BRepFontLoopId::new(new_outer),
            first_hole,
            after_last_hole - first_hole,
        ));
    }
    true
}
