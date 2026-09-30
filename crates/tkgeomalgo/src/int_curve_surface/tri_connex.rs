// Rust translation of IntCurveSurface_PolyhedronUtils::TriConnex
// (IntCurveSurface_PolyhedronUtils.pxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TriConnex` of the grid polyhedron: the triangle across an edge and its third
//! point, by the line and column arithmetic of the native code.

/// A grid line and column (`lig`/`col` of a point, `lin`/`col` of a triangle).
#[derive(Clone, Copy)]
struct Cell {
    lin: i32,
    col: i32,
}

/// The grid sizes and the pivot and edge cells of one query.
struct Query {
    nb_delta_u: i32,
    nb_delta_v: i32,
    pivot: Cell,
    edge: Cell,
    /// `typE`: 0 unknown edge, 1 same line, 2 same column, 3 diagonal.
    typ: i32,
}

/// `TriConnex(Triang, Pivot, Pedge, TriCon, OtherP)`: the triangle across the edge
/// `pivot pedge` of `triang` and its third point, as `(returned, TriCon, OtherP)`;
/// on a free edge TriCon is 0 and OtherP the point beyond the pivot along the
/// border; `triang` or `pedge` 0 lets the native rules pick them;
/// `tri_connex(1, 1, 2, (3, 3))`.
pub fn tri_connex(
    triang: i32,
    pivot: i32,
    pedge: i32,
    (nb_delta_u, nb_delta_v): (i32, i32),
) -> (i32, i32, i32) {
    let nbdelta_vp1: i32 = nb_delta_v + 1;
    let pivotm1: i32 = pivot - 1;
    let lig_p: i32 = pivotm1 / nbdelta_vp1;
    let piv = Cell {
        lin: lig_p,
        col: pivotm1 - lig_p * nbdelta_vp1,
    };
    let mut q = Query {
        nb_delta_u,
        nb_delta_v,
        pivot: piv,
        edge: Cell { lin: 0, col: 0 },
        typ: 0,
    };
    if pedge != 0 {
        let lig_e: i32 = (pedge - 1) / nbdelta_vp1;
        q.edge = Cell {
            lin: lig_e,
            col: (pedge - 1) - lig_e * nbdelta_vp1,
        };
        q.typ = if piv.lin == q.edge.lin {
            1
        } else if piv.col == q.edge.col {
            2
        } else {
            3
        };
    }
    let (t, o) = if triang != 0 {
        from_triangle(&mut q, triang)
    } else {
        from_pivot(&q, pedge)
    };
    let (tri_con, other) = clamp_to_grid(&q, t, o);
    let other_p: i32 = other.lin * nbdelta_vp1 + other.col + 1;
    (tri_con, tri_con, other_p)
}

/// The next triangle and third point when the current triangle is known.
fn from_triangle(q: &mut Query, triang: i32) -> (Cell, Cell) {
    let nbdelta_vm2: i32 = q.nb_delta_v + q.nb_delta_v;
    let t: i32 = (triang - 1) / nbdelta_vm2;
    let tt: i32 = (triang - 1) - t * nbdelta_vm2;
    let mut tri = Cell {
        lin: 1 + t,
        col: 1 + tt,
    };
    if q.typ == 0 {
        guess_edge(q, tri);
    }
    let (p, e) = (q.pivot, q.edge);
    let other: Cell = match q.typ {
        1 => step_line(&mut tri, p, e),
        2 => step_column(&mut tri, p, e),
        3 => step_diagonal(&mut tri, p, e),
        _ => Cell { lin: 0, col: 0 },
    };
    (tri, other)
}

/// The edge a known triangle implies when none is given.
fn guess_edge(q: &mut Query, tri: Cell) {
    let p: Cell = q.pivot;
    if p.lin == tri.lin {
        (q.edge, q.typ) = (
            Cell {
                lin: p.lin - 1,
                col: p.col - 1,
            },
            3,
        );
    } else if tri.col == p.lin + p.lin {
        (q.edge, q.typ) = (
            Cell {
                lin: p.lin,
                col: p.col - 1,
            },
            1,
        );
    } else {
        (q.edge, q.typ) = (
            Cell {
                lin: p.lin + 1,
                col: p.col + 1,
            },
            3,
        );
    }
}

/// Case 1 (edge along a grid line): the triangle across it and the third point.
fn step_line(tri: &mut Cell, p: Cell, e: Cell) -> Cell {
    if tri.lin == p.lin {
        tri.lin += 1;
        Cell {
            lin: p.lin + 1,
            col: p.col.max(e.col),
        }
    } else {
        tri.lin -= 1;
        Cell {
            lin: p.lin - 1,
            col: p.col.min(e.col),
        }
    }
}

/// Case 2 (edge along a grid column).
fn step_column(tri: &mut Cell, p: Cell, e: Cell) -> Cell {
    if tri.col == p.col + p.col {
        tri.col += 1;
        Cell {
            lin: p.lin.max(e.lin),
            col: p.col + 1,
        }
    } else {
        tri.col -= 1;
        Cell {
            lin: p.lin.min(e.lin),
            col: p.col - 1,
        }
    }
}

/// Case 3 (diagonal edge).
fn step_diagonal(tri: &mut Cell, p: Cell, e: Cell) -> Cell {
    if (tri.col & 1) == 0 {
        tri.col -= 1;
        Cell {
            lin: p.lin.max(e.lin),
            col: p.col.min(e.col),
        }
    } else {
        tri.col += 1;
        Cell {
            lin: p.lin.min(e.lin),
            col: p.col.max(e.col),
        }
    }
}

/// The triangle and third point from the pivot (and the edge, when given) alone.
fn from_pivot(q: &Query, pedge: i32) -> (Cell, Cell) {
    let (p, e) = (q.pivot, q.edge);
    if pedge == 0 {
        let tri = Cell {
            lin: 1.max(p.lin),
            col: 1.max(p.col + p.col),
        };
        let other = Cell {
            lin: if p.lin == 0 { p.lin + 1 } else { p.lin - 1 },
            col: p.col,
        };
        return (tri, other);
    }
    match q.typ {
        1 => (
            Cell {
                lin: p.lin + 1,
                col: p.col.max(e.col) + p.col.max(e.col),
            },
            Cell {
                lin: p.lin + 1,
                col: p.col.max(e.col),
            },
        ),
        2 => (
            Cell {
                lin: p.lin.max(e.lin),
                col: p.col + p.col,
            },
            Cell {
                lin: p.lin.min(e.lin),
                col: p.col - 1,
            },
        ),
        _ => (
            Cell {
                lin: p.lin.max(e.lin),
                col: p.col + e.col,
            },
            Cell {
                lin: p.lin.max(e.lin),
                col: p.col.min(e.col),
            },
        ),
    }
}

/// `TriCon` from the triangle cell, set to 0 with the third point moved onto the
/// border when the triangle leaves the grid (lines first, then columns).
fn clamp_to_grid(q: &Query, t: Cell, mut o: Cell) -> (i32, Cell) {
    let (p, e) = (q.pivot, q.edge);
    let (nbu, nbv) = (q.nb_delta_u, q.nb_delta_v);
    let mut tri_con: i32 = (t.lin - 1) * (nbv + nbv) + t.col;
    if t.lin < 1 {
        o = border_line(0, 1, p.col + p.col - e.col, nbv);
        tri_con = 0;
    } else if t.lin > nbu {
        o = border_line(nbu, nbu - 1, p.col + p.col - e.col, nbv);
        tri_con = 0;
    }
    if t.col < 1 {
        o = border_column(0, 1, p.lin + p.lin - e.lin, nbu);
        tri_con = 0;
    } else if t.col > nbv {
        o = border_column(nbv, nbv - 1, p.lin + p.lin - e.lin, nbu);
        tri_con = 0;
    }
    (tri_con, o)
}

/// The third point on the line `lin` at column `col`, moved to line `inner` at the
/// nearest column when `col` leaves `[0, nbv]`.
fn border_line(lin: i32, inner: i32, col: i32, nbv: i32) -> Cell {
    if col < 0 {
        Cell { lin: inner, col: 0 }
    } else if col > nbv {
        Cell {
            lin: inner,
            col: nbv,
        }
    } else {
        Cell { lin, col }
    }
}

/// The third point on the column `col` at line `lin`, moved to column `inner` at the
/// nearest line when `lin` leaves `[0, nbu]`.
fn border_column(col: i32, inner: i32, lin: i32, nbu: i32) -> Cell {
    if lin < 0 {
        Cell { lin: 0, col: inner }
    } else if lin > nbu {
        Cell {
            lin: nbu,
            col: inner,
        }
    } else {
        Cell { lin, col }
    }
}
