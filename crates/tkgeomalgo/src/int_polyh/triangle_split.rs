// Rust translation of the three split cases of MiddleRefinement and of GetInfoTA,
// NewTriangle, NewEdge and OldEdge of ModelingAlgorithms/TKGeomAlgo/IntPolyh/
// IntPolyh_Triangle.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The new triangles and edges of a split. The three native cases (the longest side
//! P1P2, P2P3 or P3P1) number their triangles and edges differently, and those numbers
//! reach the results, so each case is ported as written.
use super::triangle_refine::Mesh;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// The state of one `MiddleRefinement`: the middle point, the points and edges of the
/// refined triangle. The native counters `FinTT` and `FinTE` equal the item counts that
/// `NewTriangle` and `NewEdge` read, so the new numbers are those counts.
pub(crate) struct Splitter<'m, 'a> {
    pub(crate) fin_tp: i32,
    /// `numP1, numP2, numP3`.
    pub(crate) points: [i32; 3],
    /// `myEdges`.
    pub(crate) my_edges: [i32; 3],
    pub(crate) num_tri: i32,
    pub(crate) surface: &'m dyn Adaptor3dSurface,
    pub(crate) mesh: &'m mut Mesh<'a>,
}

/// The third point of the adjacent triangle and its edges from that point
/// (`GetInfoTA`): `(numP3b, Edge2b, Edge3b)`; -1 when no point qualifies.
fn get_info_ta(mesh: &Mesh<'_>, num_p1: i32, num_p2: i32, num_ta: i32) -> (i32, i32, i32) {
    let tri_adj = &mesh.triangles[num_ta];
    let (p1b, p2b, p3b) = (
        tri_adj.first_point(),
        tri_adj.second_point(),
        tri_adj.third_point(),
    );
    let (e1, e2, e3) = (
        tri_adj.first_edge(),
        tri_adj.second_edge(),
        tri_adj.third_edge(),
    );
    if p1b != num_p1 && p1b != num_p2 {
        // P1b P2b is the edge from the third point to numP1 when P2b is numP1.
        return if p2b == num_p1 {
            (p1b, e3, e1)
        } else {
            (p1b, e1, e3)
        };
    }
    if p2b != num_p1 && p2b != num_p2 {
        return if p1b == num_p1 {
            (p2b, e2, e1)
        } else {
            (p2b, e1, e2)
        };
    }
    if p3b != num_p1 && p3b != num_p2 {
        return if p2b == num_p1 {
            (p3b, e3, e2)
        } else {
            (p3b, e2, e3)
        };
    }
    (-1, -1, -1)
}

impl Splitter<'_, '_> {
    /// `NewTriangle(P1, P2, P3, ...)`: a triangle at the end of the array with its
    /// deflection; returns its number.
    fn new_triangle(&mut self, p1: i32, p2: i32, p3: i32) -> Result<i32, Failure> {
        let fin_tt: i32 = self.mesh.triangles.nb_items();
        let triangle = &mut self.mesh.triangles[fin_tt];
        triangle.set_first_point(p1);
        triangle.set_second_point(p2);
        triangle.set_third_point(p3);
        triangle.compute_deflection(self.surface, self.mesh.points)?;
        self.mesh.triangles.increment_nb_items();
        Ok(fin_tt)
    }

    /// `NewEdge(P1, P2, T1, T2, TEdges)`: an edge at the end of the array; returns its
    /// number.
    fn new_edge(&mut self, p1: i32, p2: i32, t1: i32, t2: i32) -> i32 {
        let fin_te: i32 = self.mesh.edges.nb_items();
        let edge = &mut self.mesh.edges[fin_te];
        edge.set_first_point(p1);
        edge.set_second_point(p2);
        edge.set_first_triangle(t1);
        edge.set_second_triangle(t2);
        self.mesh.edges.increment_nb_items();
        fin_te
    }

    /// `OldEdge(EdgeN, NumTri, NewTriNum, TEdges)`: the edge now bounds `new_tri` where it
    /// bounded `num_tri` (its second triangle when its first one is another).
    fn old_edge(&mut self, edge_n: i32, num_tri: i32, new_tri: i32) {
        let edge = &mut self.mesh.edges[edge_n];
        if edge.first_triangle() == num_tri {
            edge.set_first_triangle(new_tri);
        } else {
            edge.set_second_triangle(new_tri);
        }
    }

    /// `TTriangles[T].LinkEdges2Triangle(TEdges, E1, E2, E3)`.
    fn link(&mut self, t: i32, e: [i32; 3]) {
        self.mesh.triangles[t].link_edges_2_triangle(self.mesh.edges, e[0], e[1], e[2]);
    }

    /// Kills the adjacent triangle.
    fn kill(&mut self, num_ta: i32) {
        self.mesh.triangles[num_ta].set_deflection(-1.0);
        self.mesh.triangles[num_ta].set_intersection_possible(false);
    }

    /// The split of the side `side` (1: P1P2, 2: P2P3, 3: P3P1) with the adjacent
    /// triangle `num_ta` (-1 for none).
    pub(crate) fn run(&mut self, side: i32, num_ta: i32) -> Result<(), Failure> {
        match side {
            1 => self.split_12(num_ta),
            2 => self.split_23(num_ta),
            _ => self.split_31(num_ta),
        }
    }

    fn split_12(&mut self, num_ta: i32) -> Result<(), Failure> {
        let ([p1, p2, p3], m) = (self.points, self.fin_tp);
        let t1: i32 = self.new_triangle(p2, p3, m)?;
        let t2: i32 = self.new_triangle(p3, p1, m)?;
        let my = self.my_edges;
        if num_ta < 0 {
            let e1: i32 = self.new_edge(p1, m, t2, -1);
            let e2: i32 = self.new_edge(m, p2, t1, -1);
            let e3: i32 = self.new_edge(m, p3, t1, t2);
            self.old_edge(my[1], self.num_tri, t1);
            self.old_edge(my[2], self.num_tri, t2);
            self.link(t1, [my[1], e3, e2]);
            self.link(t2, [my[2], e1, e3]);
            return Ok(());
        }
        let (p3b, edge2b, edge3b) = get_info_ta(self.mesh, p1, p2, num_ta);
        let t3: i32 = self.new_triangle(p2, p3b, m)?;
        let t4: i32 = self.new_triangle(p3b, p1, m)?;
        let e1: i32 = self.new_edge(p1, m, t2, t4);
        let e2: i32 = self.new_edge(m, p2, t1, t3);
        let e3: i32 = self.new_edge(m, p3, t1, t2);
        let e4: i32 = self.new_edge(m, p3b, t3, t4);
        self.old_edge(my[1], self.num_tri, t1);
        self.old_edge(my[2], self.num_tri, t2);
        self.old_edge(edge2b, num_ta, t3);
        self.old_edge(edge3b, num_ta, t4);
        self.link(t1, [my[1], e3, e2]);
        self.link(t2, [my[2], e1, e3]);
        self.link(t3, [edge2b, e4, e2]);
        self.link(t4, [edge3b, e1, e4]);
        self.kill(num_ta);
        Ok(())
    }

    fn split_23(&mut self, num_ta: i32) -> Result<(), Failure> {
        let ([p1, p2, p3], m) = (self.points, self.fin_tp);
        let t1: i32 = self.new_triangle(p1, p2, m)?;
        let t2: i32 = self.new_triangle(p3, p1, m)?;
        let my = self.my_edges;
        if num_ta < 0 {
            let e1: i32 = self.new_edge(p2, m, t1, -1);
            let e2: i32 = self.new_edge(m, p3, t2, -1);
            let e3: i32 = self.new_edge(m, p1, t1, t2);
            self.old_edge(my[0], self.num_tri, t1);
            self.old_edge(my[2], self.num_tri, t2);
            self.link(t1, [my[0], e1, e3]);
            self.link(t2, [my[2], e3, e2]);
            return Ok(());
        }
        // The outputs of GetInfoTA land in (numP1b, Edge3b, Edge1b) here.
        let (p1b, edge3b, edge1b) = get_info_ta(self.mesh, p2, p3, num_ta);
        let t3: i32 = self.new_triangle(p2, p1b, m)?;
        let t4: i32 = self.new_triangle(p1b, p3, m)?;
        let e1: i32 = self.new_edge(p2, m, t1, t3);
        let e2: i32 = self.new_edge(m, p3, t2, t4);
        let e3: i32 = self.new_edge(m, p1, t1, t2);
        let e4: i32 = self.new_edge(m, p1b, t3, t4);
        self.old_edge(my[0], self.num_tri, t1);
        self.old_edge(my[2], self.num_tri, t2);
        self.old_edge(edge1b, num_ta, t3);
        self.old_edge(edge3b, num_ta, t4);
        self.link(t1, [my[0], e1, e3]);
        self.link(t2, [my[2], e3, e2]);
        self.link(t3, [edge1b, e4, e1]);
        self.link(t4, [edge3b, e2, e4]);
        self.kill(num_ta);
        Ok(())
    }

    fn split_31(&mut self, num_ta: i32) -> Result<(), Failure> {
        let ([p1, p2, p3], m) = (self.points, self.fin_tp);
        let t1: i32 = self.new_triangle(p1, p2, m)?;
        let t2: i32 = self.new_triangle(p2, p3, m)?;
        let my = self.my_edges;
        if num_ta < 0 {
            let e1: i32 = self.new_edge(p2, m, t1, t2);
            let e2: i32 = self.new_edge(m, p3, t2, -1);
            let e4: i32 = self.new_edge(m, p1, t1, -1);
            self.old_edge(my[0], self.num_tri, t1);
            self.old_edge(my[1], self.num_tri, t2);
            self.link(t1, [my[0], e1, e4]);
            self.link(t2, [my[1], e2, e1]);
            return Ok(());
        }
        let (p2b, edge1b, edge2b) = get_info_ta(self.mesh, p3, p1, num_ta);
        let t3: i32 = self.new_triangle(p1, p2b, m)?;
        let t4: i32 = self.new_triangle(p2b, p3, m)?;
        let e1: i32 = self.new_edge(p2, m, t1, t2);
        let e2: i32 = self.new_edge(m, p3, t2, t4);
        let e3: i32 = self.new_edge(m, p2b, t4, t3);
        let e4: i32 = self.new_edge(m, p1, t1, t3);
        self.old_edge(my[0], self.num_tri, t1);
        self.old_edge(my[1], self.num_tri, t2);
        self.old_edge(edge1b, num_ta, t3);
        self.old_edge(edge2b, num_ta, t4);
        self.link(t1, [my[0], e1, e4]);
        self.link(t2, [my[1], e2, e1]);
        self.link(t3, [edge1b, e3, e4]);
        self.link(t4, [edge2b, e2, e3]);
        self.kill(num_ta);
        Ok(())
    }
}
