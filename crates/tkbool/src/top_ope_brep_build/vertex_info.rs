// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_VertexInfo and
// TopOpeBRepBuild_Tools2d (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges in and out of the vertices of a wire (`VertexInfo`) and the path through
//! all the edges of a wire (`Tools2d`).
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The edges in and out of a vertex of a wire (`TopOpeBRepBuild_VertexInfo`), with the
/// edges passed by a path; `let mut vi = TopOpeBRepBuildVertexInfo::new();`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildVertexInfo {
    my_vertex: Vertex,
    my_current: Edge,
    my_current_in: Edge,
    my_smart: bool,
    my_edges_in: IndexedMap<Shape>,
    my_edges_out: IndexedMap<Shape>,
    my_local_edges_out: IndexedMap<Shape>,
    my_edges_passed: List<Shape>,
    my_found_out: i32,
}

impl TopOpeBRepBuildVertexInfo {
    /// `TopOpeBRepBuild_VertexInfo()`; `TopOpeBRepBuildVertexInfo::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetVertex(aV)`; `vi.set_vertex(&v)`.
    pub fn set_vertex(&mut self, a_v: &Vertex) {
        self.my_vertex = a_v.clone();
    }

    /// `Vertex()`; `vi.vertex()`.
    pub fn vertex(&self) -> &Vertex {
        &self.my_vertex
    }

    /// `SetSmart(aFlag)`; `vi.set_smart(true)`.
    pub fn set_smart(&mut self, a_flag: bool) {
        self.my_smart = a_flag;
    }

    /// `Smart()`; `vi.smart()`.
    pub fn smart(&self) -> bool {
        self.my_smart
    }

    /// `NbCases()`: the edges out not in the path; `vi.nb_cases()`.
    pub fn nb_cases(&self) -> i32 {
        self.my_local_edges_out.extent() as i32
    }

    /// `FoundOut()`; `vi.found_out()`.
    pub fn found_out(&self) -> i32 {
        self.my_found_out
    }

    /// `AddIn(anE)`; `vi.add_in(&e)`.
    pub fn add_in(&mut self, an_e: &Edge) {
        self.my_edges_in.add(an_e.clone().into_shape());
    }

    /// `AddOut(anE)`; `vi.add_out(&e)`.
    pub fn add_out(&mut self, an_e: &Edge) {
        self.my_edges_out.add(an_e.clone().into_shape());
    }

    /// `SetCurrentIn(anE)`; `vi.set_current_in(&e)`.
    pub fn set_current_in(&mut self, an_e: &Edge) {
        self.my_current_in = an_e.clone();
    }

    /// `EdgesIn()`; `vi.edges_in()`.
    pub fn edges_in(&self) -> &IndexedMap<Shape> {
        &self.my_edges_in
    }

    /// `EdgesOut()`; `vi.edges_out()`.
    pub fn edges_out(&self) -> &IndexedMap<Shape> {
        &self.my_edges_out
    }

    /// `ChangeEdgesOut()`; `vi.change_edges_out()`.
    pub fn change_edges_out(&mut self) -> &mut IndexedMap<Shape> {
        &mut self.my_edges_out
    }

    /// `Dump()`: the smart flag (printed 0 when smart, as natively) and the edge counts;
    /// `vi.dump()`.
    pub fn dump(&self) {
        println!(" *** Dump the Vertex Info ***");
        println!(" mySmart  : {}", if self.my_smart { 0 } else { 1 });
        println!(
            " Edges    : {} In, {} Out",
            self.my_edges_in.extent(),
            self.my_edges_out.extent()
        );
        println!();
    }

    /// `AppendPassed(anE)`: the edge at the head of the passed edges; `vi.append_passed(&e)`.
    pub fn append_passed(&mut self, an_e: &Edge) {
        self.my_edges_passed.prepend(an_e.clone().into_shape());
    }

    /// `RemovePassed()`: the head of the passed edges removed (Standard_NoSuchObject when
    /// none); `vi.remove_passed()?`.
    pub fn remove_passed(&mut self) -> Result<(), Failure> {
        self.my_edges_passed.remove_first()
    }

    /// `ListPassed()`; `vi.list_passed()`.
    pub fn list_passed(&self) -> &List<Shape> {
        &self.my_edges_passed
    }

    /// `Prepare(aL)`: the edges out not in `a_l`; `vi.prepare(&path)`.
    pub fn prepare(&mut self, a_l: &List<Shape>) {
        self.my_local_edges_out.clear(false);
        let mut tmp: IndexedMap<Shape> = IndexedMap::default();
        for s in a_l.iter() {
            tmp.add(s.clone());
        }
        for a_e in self.my_edges_out.iter() {
            if !tmp.contains(a_e) {
                self.my_local_edges_out.add(a_e.clone());
            }
        }
    }

    /// `CurrentOut()`: the first edge out of the path not passed yet (`FoundOut` 1), else
    /// the null edge (`FoundOut` 0); `vi.current_out()?`.
    pub fn current_out(&mut self) -> Result<Edge, Failure> {
        let mut passed: IndexedMap<Shape> = IndexedMap::default();
        for s in self.my_edges_passed.iter() {
            passed.add(s.clone());
        }
        for e in self.my_local_edges_out.iter() {
            if !passed.contains(e) {
                self.my_current = topods::edge(e)?;
                self.my_found_out = 1;
                return Ok(self.my_current.clone());
            }
        }
        self.my_found_out = 0;
        self.my_current = Edge::default();
        Ok(self.my_current.clone())
    }
}

/// The vertex infos of a wire by vertex.
pub type MapOfShapeVertexInfo = IndexedDataMap<Shape, TopOpeBRepBuildVertexInfo, ShapeMapHasher>;

/// The path of `Tools2d` (static members only).
pub struct TopOpeBRepBuildTools2d;

/// The recursion state of `BuildPath`.
struct PathWalk<'a> {
    vertex0: Shape,
    nb_edges: i32,
    m: &'a mut MapOfShapeVertexInfo,
    break_flag: i32,
    res: List<Shape>,
}

impl PathWalk<'_> {
    /// `BuildPath(myVertex0, myInputEdge, myInputVertex, ...)` (static): the edges out of
    /// the vertex tried in turn until the path closes over all the edges (flag 1) or
    /// dead-ends (flag 2).
    fn build_path(
        &mut self,
        input_edge: &Edge,
        input_vertex: &Shape,
        mut count: i32,
    ) -> Result<(), Failure> {
        if self.break_flag == 1 {
            return Ok(());
        }
        let vi = self.m.change_from_key(input_vertex)?;
        vi.set_current_in(input_edge);
        vi.prepare(&self.res);
        let nb_cases = vi.nb_cases();
        if nb_cases == 0 {
            self.break_flag = 2;
        }
        let mut stop_flag = false;
        for j in 1..=nb_cases {
            let vi = self.m.change_from_key(input_vertex)?;
            let edge = vi.current_out()?;
            if vi.found_out() == 0 {
                self.break_flag = 2;
                return Ok(());
            }
            if stop_flag {
                vi.remove_passed()?;
                self.res.remove_first()?;
                stop_flag = false;
                count -= 1;
            }
            vi.append_passed(&edge);
            self.res.prepend(edge.clone().into_shape());
            count += 1;
            let vertex = if edge.orientation() == Orientation::Forward {
                topexp::last_vertex(&edge, false)
            } else {
                topexp::first_vertex(&edge, false)
            };
            if vertex.is_same(&self.vertex0) && count == self.nb_edges {
                self.break_flag = 1;
                return Ok(());
            }
            self.build_path(&edge, vertex.as_ref(), count)?;
            match self.break_flag {
                1 => return Ok(()),
                2 if j == nb_cases => {
                    self.m.change_from_key(input_vertex)?.remove_passed()?;
                    self.res.remove_first()?;
                    return Ok(());
                }
                2 => (stop_flag, self.break_flag) = (true, 0),
                _ => {}
            }
        }
        Ok(())
    }
}

impl TopOpeBRepBuildTools2d {
    /// `Path(aWire, aResList)`: the edges of the wire in the order of a path from its
    /// first vertex through all its edges (the last edge first); `TopOpeBRepBuildTools2d::path(&w)?`.
    pub fn path(a_wire: &Wire) -> Result<List<Shape>, Failure> {
        let nb_edges = Explorer::new(a_wire, ShapeEnum::Edge, ShapeEnum::Shape)?.count() as i32;
        let mut m = MapOfShapeVertexInfo::default();
        Self::make_map_of_shape_vertex_info(a_wire, &mut m)?;
        let vertex0 = topods::vertex(m.find_key(1)?)?.into_shape();
        let mut walk = PathWalk {
            vertex0: vertex0.clone(),
            nb_edges,
            m: &mut m,
            break_flag: 0,
            res: List::default(),
        };
        walk.build_path(&Edge::default(), &vertex0, 0)?;
        Ok(walk.res)
    }

    /// `MakeMapOfShapeVertexInfo(aWire, aMap)`: the edges in (REVERSED vertex) and out
    /// (FORWARD vertex) of each vertex of the wire, smart when neither count is one;
    /// `TopOpeBRepBuildTools2d::make_map_of_shape_vertex_info(&w, &mut m)?`.
    pub fn make_map_of_shape_vertex_info(
        a_wire: &Wire,
        m: &mut MapOfShapeVertexInfo,
    ) -> Result<(), Failure> {
        for e in Explorer::new(a_wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let an_edge = topods::edge(&e)?;
            for v in Explorer::new(&an_edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let a_vertex = topods::vertex(&v)?;
                if !m.contains(a_vertex.as_ref()) {
                    m.add(
                        a_vertex.clone().into_shape(),
                        TopOpeBRepBuildVertexInfo::new(),
                    );
                }
                let vi = m.change_from_key(a_vertex.as_ref())?;
                vi.set_vertex(&a_vertex);
                match a_vertex.orientation() {
                    Orientation::Forward => vi.add_out(&an_edge),
                    Orientation::Reversed => vi.add_in(&an_edge),
                    _ => {}
                }
            }
        }
        for i in 1..=m.extent() {
            let vi = m.change_from_index(i)?;
            if vi.edges_in().extent() != 1 && vi.edges_out().extent() != 1 {
                vi.set_smart(true);
            }
        }
        Ok(())
    }

    /// `DumpMapOfShapeVertexInfo(aMap)`: per vertex, the passed edges or "none";
    /// `TopOpeBRepBuildTools2d::dump_map_of_shape_vertex_info(&m)`.
    pub fn dump_map_of_shape_vertex_info(m: &MapOfShapeVertexInfo) {
        for i in 1..=m.extent() {
            print!(" Vert.#{i}, ");
            let Ok(vi) = m.find_from_index(i) else {
                continue;
            };
            if vi.list_passed().is_empty() {
                print!("none");
            } else {
                for _ in vi.list_passed().iter() {
                    print!("pass,");
                }
            }
            println!();
        }
    }
}
