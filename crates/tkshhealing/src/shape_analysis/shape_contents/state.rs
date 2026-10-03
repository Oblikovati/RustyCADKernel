use super::*;
#[derive(Clone, Default)]
pub(super) struct ContentCounts {
    pub solids: i32,
    pub shells: i32,
    pub faces: i32,
    pub wires: i32,
    pub edges: i32,
    pub vertices: i32,
    pub solids_with_voids: i32,
    pub big_splines: i32,
    pub c0_surfaces: i32,
    pub c0_curves: i32,
    pub offset_surfaces: i32,
    pub indirect_surfaces: i32,
    pub offset_curves: i32,
    pub trimmed_curves_2d: i32,
    pub trimmed_curves_3d: i32,
    pub bspline_surfaces: i32,
    pub bezier_surfaces: i32,
    pub trimmed_surfaces: i32,
    pub wires_with_seam: i32,
    pub wires_with_several_seams: i32,
    pub faces_with_several_wires: i32,
    pub missing_pcurves: i32,
    pub free_faces: i32,
    pub free_wires: i32,
    pub free_edges: i32,
    pub shared_solids: i32,
    pub shared_shells: i32,
    pub shared_faces: i32,
    pub shared_wires: i32,
    pub shared_free_wires: i32,
    pub shared_free_edges: i32,
    pub shared_edges: i32,
    pub shared_vertices: i32,
}
#[derive(Clone)]
pub(super) struct ContentRecording {
    pub enabled: bool,
    pub shapes: Handle<HSequence<Shape>>,
}
impl Default for ContentRecording {
    fn default() -> Self {
        Self {
            enabled: false,
            shapes: Handle::new(HSequence::default()),
        }
    }
}
impl ContentRecording {
    pub fn append(&self, shape: &Shape) {
        if self.enabled {
            self.shapes.append(shape.clone());
        }
    }
}
#[derive(Clone, Default)]
pub(super) struct ContentRecordings {
    pub big_spline: ContentRecording,
    pub indirect: ContentRecording,
    pub offset_surface: ContentRecording,
    pub trimmed_3d: ContentRecording,
    pub offset_curve: ContentRecording,
    pub trimmed_2d: ContentRecording,
}
impl ContentRecordings {
    fn groups_mut(&mut self) -> [&mut ContentRecording; 6] {
        [
            &mut self.big_spline,
            &mut self.indirect,
            &mut self.offset_surface,
            &mut self.trimmed_3d,
            &mut self.offset_curve,
            &mut self.trimmed_2d,
        ]
    }
    pub fn clear_sequences(&mut self) {
        for group in self.groups_mut() {
            group.shapes.change_sequence(|values| values.clear(None));
        }
    }
    pub fn clear_flags(&mut self) {
        for group in self.groups_mut() {
            group.enabled = false;
        }
    }
}
