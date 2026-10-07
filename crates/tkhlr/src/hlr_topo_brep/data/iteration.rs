//! HLRTopoBRep_Data.cxx: the walk over the edges that carry inserted vertices and
//! over the vertices of one edge.
use super::{Data, VData};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::standard::Failure;

fn no_vertex(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("HLRTopoBRep_Data::{member}: no current vertex; expected MoreVertex()"),
        "",
    )
}

impl Data {
    // Skips the edges whose vertex list is empty, from the current position.
    fn skip_empty_edges(&mut self) {
        while let Some(key) = self.edge_keys.get(self.edge_index) {
            if self.edges_vertices.seek(key).is_some_and(|l| !l.is_empty()) {
                return;
            }
            self.edge_index += 1;
        }
    }

    /// InitEdge(): the first edge with inserted vertices; `data.init_edge()`.
    pub fn init_edge(&mut self) {
        let mut keys = Vec::new();
        let mut cursor = self.edges_vertices.cursor();
        while cursor.more() {
            if let Ok(key) = cursor.key() {
                keys.push(key.clone());
            }
            cursor.next();
        }
        self.edge_keys = keys;
        self.edge_index = 0;
        self.skip_empty_edges();
    }

    /// MoreEdge(); `data.more_edge()`.
    pub fn more_edge(&self) -> bool {
        self.edge_index < self.edge_keys.len()
    }

    /// NextEdge(): the next edge with inserted vertices; `data.next_edge()`.
    pub fn next_edge(&mut self) {
        self.edge_index += 1;
        self.skip_empty_edges();
    }

    /// Edge(): the current edge; Standard_NoSuchObject past the end;
    /// `data.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        let key = self.edge_keys.get(self.edge_index).ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "HLRTopoBRep_Data::Edge: no current edge; expected MoreEdge()",
                "",
            )
        })?;
        topods::edge(key)
    }

    /// InitVertex(E): the walk over the vertices of `e`, whose list is created
    /// empty when needed; `data.init_vertex(&e)`.
    pub fn init_vertex(&mut self, e: &Edge) {
        let key = Shape::from(e.clone());
        self.edges_vertices.try_emplaced(key.clone(), Vec::new);
        self.v_edge = Some(key);
        self.v_index = 0;
    }

    fn vertex_list(&self) -> Option<&Vec<VData>> {
        self.v_edge
            .as_ref()
            .and_then(|e| self.edges_vertices.seek(e))
    }

    fn vertex_list_mut(&mut self) -> Option<&mut Vec<VData>> {
        let key = self.v_edge.clone()?;
        self.edges_vertices.change_seek(&key)
    }

    /// MoreVertex(); `data.more_vertex()`.
    pub fn more_vertex(&self) -> bool {
        self.vertex_list().is_some_and(|l| self.v_index < l.len())
    }

    /// NextVertex(); `data.next_vertex()`.
    pub fn next_vertex(&mut self) {
        self.v_index += 1;
    }

    fn current_vdata(&self, member: &str) -> Result<&VData, Failure> {
        self.vertex_list()
            .and_then(|l| l.get(self.v_index))
            .ok_or_else(|| no_vertex(member))
    }

    /// Vertex(): the current vertex; `data.vertex()?`.
    pub fn vertex(&self) -> Result<Vertex, Failure> {
        topods::vertex(self.current_vdata("Vertex")?.vertex())
    }

    /// Parameter(): the parameter of the current vertex; `data.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        Ok(self.current_vdata("Parameter")?.parameter())
    }

    /// InsertBefore(V, P): inserts before the current vertex, which stays current;
    /// `data.insert_before(&v, 0.5)?`.
    pub fn insert_before(&mut self, v: &Vertex, p: f64) -> Result<(), Failure> {
        let index = self.v_index;
        let list = self
            .vertex_list_mut()
            .ok_or_else(|| no_vertex("InsertBefore"))?;
        if index > list.len() {
            return Err(no_vertex("InsertBefore"));
        }
        list.insert(index, VData::with_vertex(p, &Shape::from(v.clone())));
        self.v_index += 1;
        Ok(())
    }

    /// Append(V, P): appends to the vertices of the current edge;
    /// `data.append(&v, 0.5)?`.
    pub fn append(&mut self, v: &Vertex, p: f64) -> Result<(), Failure> {
        let list = self.vertex_list_mut().ok_or_else(|| no_vertex("Append"))?;
        list.push(VData::with_vertex(p, &Shape::from(v.clone())));
        Ok(())
    }
}
