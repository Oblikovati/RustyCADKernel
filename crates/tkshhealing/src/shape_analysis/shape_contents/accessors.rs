use super::*;
impl ShapeAnalysisShapeContents {
    /// Return the native solids count; `contents.nb_solids()`.
    pub fn nb_solids(&self) -> i32 {
        self.counts.solids
    }
    /// Return the native shells count; `contents.nb_shells()`.
    pub fn nb_shells(&self) -> i32 {
        self.counts.shells
    }
    /// Return the native faces count; `contents.nb_faces()`.
    pub fn nb_faces(&self) -> i32 {
        self.counts.faces
    }
    /// Return the native wires count; `contents.nb_wires()`.
    pub fn nb_wires(&self) -> i32 {
        self.counts.wires
    }
    /// Return the native edges count; `contents.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.counts.edges
    }
    /// Return the native vertices count; `contents.nb_vertices()`.
    pub fn nb_vertices(&self) -> i32 {
        self.counts.vertices
    }
    /// Return the native solids with voids count; `contents.nb_solids_with_voids()`.
    pub fn nb_solids_with_voids(&self) -> i32 {
        self.counts.solids_with_voids
    }
    /// Return the native big splines count; `contents.nb_big_splines()`.
    pub fn nb_big_splines(&self) -> i32 {
        self.counts.big_splines
    }
    /// Return the native c0 surfaces count; `contents.nb_c0_surfaces()`.
    pub fn nb_c0_surfaces(&self) -> i32 {
        self.counts.c0_surfaces
    }
    /// Return the native c0 curves count; `contents.nb_c0_curves()`.
    pub fn nb_c0_curves(&self) -> i32 {
        self.counts.c0_curves
    }
    /// Return the native offset surfaces count; `contents.nb_offset_surfaces()`.
    pub fn nb_offset_surfaces(&self) -> i32 {
        self.counts.offset_surfaces
    }
    /// Return the native indirect surfaces count; `contents.nb_indirect_surfaces()`.
    pub fn nb_indirect_surfaces(&self) -> i32 {
        self.counts.indirect_surfaces
    }
    /// Return the native offset curves count; `contents.nb_offset_curves()`.
    pub fn nb_offset_curves(&self) -> i32 {
        self.counts.offset_curves
    }
    /// Return the native trimmed curves 2d count; `contents.nb_trimmed_curves_2d()`.
    pub fn nb_trimmed_curves_2d(&self) -> i32 {
        self.counts.trimmed_curves_2d
    }
    /// Return the native trimmed curves 3d count; `contents.nb_trimmed_curves_3d()`.
    pub fn nb_trimmed_curves_3d(&self) -> i32 {
        self.counts.trimmed_curves_3d
    }
    /// Return the native bspline surfaces count; `contents.nb_bspline_surfaces()`.
    pub fn nb_bspline_surfaces(&self) -> i32 {
        self.counts.bspline_surfaces
    }
    /// Return the native bezier surfaces count; `contents.nb_bezier_surfaces()`.
    pub fn nb_bezier_surfaces(&self) -> i32 {
        self.counts.bezier_surfaces
    }
    /// Return the native trimmed surfaces count; `contents.nb_trimmed_surfaces()`.
    pub fn nb_trimmed_surfaces(&self) -> i32 {
        self.counts.trimmed_surfaces
    }
    /// Return the native wires with seam count; `contents.nb_wires_with_seam()`.
    pub fn nb_wires_with_seam(&self) -> i32 {
        self.counts.wires_with_seam
    }
    /// Return the native wires with several seams count; `contents.nb_wires_with_several_seams()`.
    pub fn nb_wires_with_several_seams(&self) -> i32 {
        self.counts.wires_with_several_seams
    }
    /// Return the native faces with several wires count; `contents.nb_faces_with_several_wires()`.
    pub fn nb_faces_with_several_wires(&self) -> i32 {
        self.counts.faces_with_several_wires
    }
    /// Return the native missing pcurves count; `contents.nb_missing_pcurves()`.
    pub fn nb_missing_pcurves(&self) -> i32 {
        self.counts.missing_pcurves
    }
    /// Return the native free faces count; `contents.nb_free_faces()`.
    pub fn nb_free_faces(&self) -> i32 {
        self.counts.free_faces
    }
    /// Return the native free wires count; `contents.nb_free_wires()`.
    pub fn nb_free_wires(&self) -> i32 {
        self.counts.free_wires
    }
    /// Return the native free edges count; `contents.nb_free_edges()`.
    pub fn nb_free_edges(&self) -> i32 {
        self.counts.free_edges
    }
    /// Return the native shared solids count; `contents.nb_shared_solids()`.
    pub fn nb_shared_solids(&self) -> i32 {
        self.counts.shared_solids
    }
    /// Return the native shared shells count; `contents.nb_shared_shells()`.
    pub fn nb_shared_shells(&self) -> i32 {
        self.counts.shared_shells
    }
    /// Return the native shared faces count; `contents.nb_shared_faces()`.
    pub fn nb_shared_faces(&self) -> i32 {
        self.counts.shared_faces
    }
    /// Return the native shared wires count; `contents.nb_shared_wires()`.
    pub fn nb_shared_wires(&self) -> i32 {
        self.counts.shared_wires
    }
    /// Return the native shared free wires count; `contents.nb_shared_free_wires()`.
    pub fn nb_shared_free_wires(&self) -> i32 {
        self.counts.shared_free_wires
    }
    /// Return the native shared free edges count; `contents.nb_shared_free_edges()`.
    pub fn nb_shared_free_edges(&self) -> i32 {
        self.counts.shared_free_edges
    }
    /// Return the native shared edges count; `contents.nb_shared_edges()`.
    pub fn nb_shared_edges(&self) -> i32 {
        self.counts.shared_edges
    }
    /// Return the native shared vertices count; `contents.nb_shared_vertices()`.
    pub fn nb_shared_vertices(&self) -> i32 {
        self.counts.shared_vertices
    }
    /// Borrow the big spline recording flag; `*contents.modify_big_spline_mode()=true`.
    pub fn modify_big_spline_mode(&mut self) -> &mut bool {
        &mut self.recordings.big_spline.enabled
    }
    /// Share the native big spline sequence; `contents.big_spline_sequence()`.
    pub fn big_spline_sequence(&self) -> Handle<HSequence<Shape>> {
        self.recordings.big_spline.shapes.clone()
    }
    /// Borrow the indirect recording flag; `*contents.modify_indirect_mode()=true`.
    pub fn modify_indirect_mode(&mut self) -> &mut bool {
        &mut self.recordings.indirect.enabled
    }
    /// Share the native indirect sequence; `contents.indirect_sequence()`.
    pub fn indirect_sequence(&self) -> Handle<HSequence<Shape>> {
        self.recordings.indirect.shapes.clone()
    }
    /// Borrow the offset surface recording flag; `*contents.modify_offset_surface_mode()=true`.
    pub fn modify_offset_surface_mode(&mut self) -> &mut bool {
        &mut self.recordings.offset_surface.enabled
    }
    /// Share the native offset surface sequence; `contents.offset_surface_sequence()`.
    pub fn offset_surface_sequence(&self) -> Handle<HSequence<Shape>> {
        self.recordings.offset_surface.shapes.clone()
    }
    /// Borrow the trimmed 3d recording flag; `*contents.modify_trimmed_3d_mode()=true`.
    pub fn modify_trimmed_3d_mode(&mut self) -> &mut bool {
        &mut self.recordings.trimmed_3d.enabled
    }
    /// Share the native trimmed 3d sequence; `contents.trimmed_3d_sequence()`.
    pub fn trimmed_3d_sequence(&self) -> Handle<HSequence<Shape>> {
        self.recordings.trimmed_3d.shapes.clone()
    }
    /// Borrow the offset curve recording flag; `*contents.modify_offset_curve_mode()=true`.
    pub fn modify_offset_curve_mode(&mut self) -> &mut bool {
        &mut self.recordings.offset_curve.enabled
    }
    /// Share the native offset curve sequence; `contents.offset_curve_sequence()`.
    pub fn offset_curve_sequence(&self) -> Handle<HSequence<Shape>> {
        self.recordings.offset_curve.shapes.clone()
    }
    /// Borrow the trimmed 2d recording flag; `*contents.modify_trimmed_2d_mode()=true`.
    pub fn modify_trimmed_2d_mode(&mut self) -> &mut bool {
        &mut self.recordings.trimmed_2d.enabled
    }
    /// Share the native trimmed 2d sequence; `contents.trimmed_2d_sequence()`.
    pub fn trimmed_2d_sequence(&self) -> Handle<HSequence<Shape>> {
        self.recordings.trimmed_2d.shapes.clone()
    }
    /// Preserve OCCT's deprecated misspelling as an alias; `*contents.modify_offest_surface_mode()=true`.
    pub fn modify_offest_surface_mode(&mut self) -> &mut bool {
        self.modify_offset_surface_mode()
    }
}
