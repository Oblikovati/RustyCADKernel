use rustycad_tkbrep::topods::{Edge, Vertex};

#[derive(Default)]
pub struct FaceCreatedImages {
    edges: Vec<(Edge, Edge)>,
    vertices: Vec<(Vertex, Vertex)>,
}

impl FaceCreatedImages {
    pub(super) fn edge_image(&self, source: &Edge) -> Option<Edge> {
        self.edges
            .iter()
            .find(|(candidate, _)| candidate.is_same(source))
            .map(|(_, image)| image.clone())
    }

    pub(super) fn has_edge(&self, source: &Edge) -> bool {
        self.edges
            .iter()
            .any(|(candidate, _)| candidate.is_same(source))
    }

    pub(super) fn bind_edge(&mut self, source: &Edge, image: &Edge) {
        self.edges.push((source.clone(), image.clone()));
    }

    pub(super) fn vertex_image(&self, source: &Vertex) -> Option<Vertex> {
        self.vertices
            .iter()
            .find(|(candidate, _)| candidate.is_same(source))
            .map(|(_, image)| image.clone())
    }

    pub(super) fn bind_vertex(&mut self, source: &Vertex, image: &Vertex) {
        self.vertices.push((source.clone(), image.clone()));
    }
}
