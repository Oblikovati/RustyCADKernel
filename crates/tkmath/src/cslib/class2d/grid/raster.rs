// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Grid construction: edge-box rasterization, then per-component flood fill.
use super::{GRID_SIZE, GridCell};
use crate::cslib::class2d::normalize::is_safe_for_direct_arithmetic;
use crate::cslib::class2d::ray::NormalizedPolygon;

/// Native Standard_OutOfMemory / std::bad_alloc while allocating the grid.
pub(super) struct AllocationFailure;

const TOTAL_CELLS: usize = (GRID_SIZE * GRID_SIZE) as usize;

/// Classified cells, `Ok(None)` when the grid is intrinsically unusable
/// (Disabled), or an allocation failure (state returns to NotBuilt).
pub(super) fn classify_cells(
    polygon: &NormalizedPolygon,
    tol: [f64; 2],
) -> Result<Option<Box<[GridCell]>>, AllocationFailure> {
    let mut cells: Vec<GridCell> = allocate(GridCell::Unvisited)?;
    let mut queue: Vec<i32> = Vec::new();
    queue
        .try_reserve_exact(TOTAL_CELLS)
        .map_err(|_| AllocationFailure)?;
    // Non-finite polygon data and domain-sized tolerances are valid for the
    // exact path but cannot produce a useful, safely indexed cache.
    if tol[0] >= 1.0 || tol[1] >= 1.0 || !has_finite_vertices(polygon) {
        return Ok(None);
    }
    mark_boundary_cells(polygon, tol, &mut cells);
    let usable: usize = flood_fill(polygon, &mut cells, &mut queue);
    if usable == 0 {
        return Ok(None);
    }
    Ok(Some(cells.into_boxed_slice()))
}

fn allocate(value: GridCell) -> Result<Vec<GridCell>, AllocationFailure> {
    let mut cells: Vec<GridCell> = Vec::new();
    cells
        .try_reserve_exact(TOTAL_CELLS)
        .map_err(|_| AllocationFailure)?;
    cells.resize(TOTAL_CELLS, value);
    Ok(cells)
}

fn has_finite_vertices(polygon: &NormalizedPolygon) -> bool {
    (0..polygon.count()).all(|index| {
        is_safe_for_direct_arithmetic(polygon.xs[index])
            && is_safe_for_direct_arithmetic(polygon.ys[index])
    })
}

/// The inclusive cell range covered by a clipped `[min, max]` coordinate span.
fn cell_span(min: f64, max: f64) -> (i32, i32) {
    let size: f64 = f64::from(GRID_SIZE);
    let first: i32 = ((min.clamp(0.0, 1.0) * size).ceil() as i32 - 1).max(0);
    let last: i32 = ((max.clamp(0.0, 1.0) * size).floor() as i32).min(GRID_SIZE - 1);
    (first, last)
}

// Rasterizes tolerance-expanded edge boxes; equivalent to testing every cell box
// against every edge box without the O(cells * edges) scan.
fn mark_boundary_cells(polygon: &NormalizedPolygon, tol: [f64; 2], cells: &mut [GridCell]) {
    let eps: f64 = 8.0 * f64::EPSILON;
    let (xs, ys) = (&polygon.xs, &polygon.ys);
    for edge in 0..polygon.count() {
        let min_x: f64 = xs[edge].min(xs[edge + 1]) - tol[0] - eps;
        let max_x: f64 = xs[edge].max(xs[edge + 1]) + tol[0] + eps;
        let min_y: f64 = ys[edge].min(ys[edge + 1]) - tol[1] - eps;
        let max_y: f64 = ys[edge].max(ys[edge + 1]) + tol[1] + eps;
        if max_x < 0.0 || min_x > 1.0 || max_y < 0.0 || min_y > 1.0 {
            continue;
        }
        let (first_x, last_x) = cell_span(min_x, max_x);
        let (first_y, last_y) = cell_span(min_y, max_y);
        for cell_y in first_y..=last_y {
            for cell_x in first_x..=last_x {
                cells[(cell_y * GRID_SIZE + cell_x) as usize] = GridCell::Boundary;
            }
        }
    }
}

// A polygon cannot change classification inside a connected set of cells that
// contains no boundary: classify one center per component and flood the result.
fn flood_fill(polygon: &NormalizedPolygon, cells: &mut [GridCell], queue: &mut Vec<i32>) -> usize {
    let mut usable: usize = 0;
    for seed in 0..GRID_SIZE * GRID_SIZE {
        if cells[seed as usize] != GridCell::Unvisited {
            continue;
        }
        let value: GridCell = seed_classification(polygon, seed);
        queue.clear();
        queue.push(seed);
        cells[seed as usize] = value;
        usable += flood_component(cells, queue, value);
    }
    usable
}

fn seed_classification(polygon: &NormalizedPolygon, seed: i32) -> GridCell {
    let cell_size: f64 = 1.0 / f64::from(GRID_SIZE);
    let center_x: f64 = (f64::from(seed % GRID_SIZE) + 0.5) * cell_size;
    let center_y: f64 = (f64::from(seed / GRID_SIZE) + 0.5) * cell_size;
    if polygon.contains(center_x, center_y) {
        GridCell::Inside
    } else {
        GridCell::Outside
    }
}

// Breadth-first flood over 4-neighbours in the native left, right, down, up order.
fn flood_component(cells: &mut [GridCell], queue: &mut Vec<i32>, value: GridCell) -> usize {
    let mut index: usize = 0;
    while index < queue.len() {
        let current: i32 = queue[index];
        index += 1;
        for neighbour in neighbours(current).into_iter().flatten() {
            if cells[neighbour as usize] == GridCell::Unvisited {
                cells[neighbour as usize] = value;
                queue.push(neighbour);
            }
        }
    }
    index
}

fn neighbours(cell: i32) -> [Option<i32>; 4] {
    let x: i32 = cell % GRID_SIZE;
    let y: i32 = cell / GRID_SIZE;
    [
        (x > 0).then_some(cell - 1),
        (x + 1 < GRID_SIZE).then_some(cell + 1),
        (y > 0).then_some(cell - GRID_SIZE),
        (y + 1 < GRID_SIZE).then_some(cell + GRID_SIZE),
    ]
}
