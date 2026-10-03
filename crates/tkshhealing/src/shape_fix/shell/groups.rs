//! Native connected-component ordering, largest first with stable ties.
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Edge, Face, Shape},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::{ncollection::DataMap, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) type ShapeMap<T> = DataMap<Shape, T, ShapeMapHasher>;
pub(super) struct FaceConnectivity {
    pub faces: Vec<Face>,
    pub edges: Vec<Vec<Edge>>,
    adjacency: ShapeMap<Vec<usize>>,
}
impl FaceConnectivity {
    pub fn new(faces: &[Face]) -> Result<Self, Failure> {
        let mut graph = Self {
            faces: faces.to_vec(),
            edges: Vec::new(),
            adjacency: ShapeMap::default(),
        };
        for (index, face) in faces.iter().enumerate() {
            graph.append_face(index, face)?;
        }
        Ok(graph)
    }
    fn append_face(&mut self, index: usize, face: &Face) -> Result<(), Failure> {
        let edges = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?
            .map(|shape| topods::edge(&shape))
            .collect::<Result<Vec<_>, _>>()?;
        for edge in &edges {
            let adjacent = self
                .adjacency
                .try_bound(edge.clone().into_shape(), Vec::new());
            if !adjacent.contains(&index) {
                adjacent.push(index);
            }
        }
        self.edges.push(edges);
        Ok(())
    }
    pub fn connected_groups(&self) -> Result<Vec<Vec<Face>>, Failure> {
        let mut visited = vec![false; self.faces.len()];
        let mut groups = Vec::new();
        for root in 0..self.faces.len() {
            if visited[root] {
                continue;
            }
            visited[root] = true;
            let mut stack = vec![root];
            let mut group = Vec::new();
            while let Some(index) = stack.pop() {
                group.push(self.faces[index].clone());
                self.visit_neighbors(index, &mut visited, &mut stack)?;
            }
            groups.push(group);
        }
        groups.sort_by_key(|group| std::cmp::Reverse(group.len()));
        Ok(groups)
    }
    fn visit_neighbors(
        &self,
        index: usize,
        visited: &mut [bool],
        stack: &mut Vec<usize>,
    ) -> Result<(), Failure> {
        for edge in &self.edges[index] {
            for &neighbor in self.adjacency.find(edge)? {
                if visited[neighbor] {
                    continue;
                }
                visited[neighbor] = true;
                stack.push(neighbor);
            }
        }
        Ok(())
    }
    pub fn face_edges(&self, face: &Face) -> &[Edge] {
        self.faces
            .iter()
            .position(|candidate| candidate.is_same(face))
            .map_or(&[], |index| self.edges[index].as_slice())
    }
}
