//! Native IMeshTools mesh strategy selection and unnormalized input parameters.
/// Built-in algorithm selector; `MeshAlgoType::Default` defers global selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum MeshAlgoType {
    /// Global default or CSF_MeshAlgo environment setting.
    #[default]
    Default = -1,
    /// Watson triangulation.
    Watson = 0,
    /// Delabella triangulation.
    Delabella = 1,
}
/// Native `IMeshTools_Parameters`; `MeshParameters::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshParameters {
    /// Surface triangulation strategy.
    pub mesh_algo: MeshAlgoType,
    /// Boundary angular deflection.
    pub angle: f64,
    /// Boundary linear deflection.
    pub deflection: f64,
    /// Interior angular deflection, negative until normalized.
    pub angle_interior: f64,
    /// Interior linear deflection, negative until normalized.
    pub deflection_interior: f64,
    /// Minimum element size, negative until normalized.
    pub min_size: f64,
    /// Enable parallel processing.
    pub in_parallel: bool,
    /// Scale deflection by edge size.
    pub relative: bool,
    /// Include internal face vertices.
    pub internal_vertices_mode: bool,
    /// Check deviation of the face interior.
    pub control_surface_deflection: bool,
    /// Apply surface control to analytical surfaces too.
    pub enable_control_surface_deflection_all_surfaces: bool,
    /// Release the temporary discrete model on completion.
    pub clean_model: bool,
    /// Adapt minimum size locally.
    pub adjust_min_size: bool,
    /// Force shape tolerances into face deflection.
    pub force_face_deflection: bool,
    /// Permit a coarser mesh than the existing one.
    pub allow_quality_decrease: bool,
}
impl Default for MeshParameters {
    fn default() -> Self {
        Self {
            mesh_algo: MeshAlgoType::Default,
            angle: 0.5,
            deflection: 0.001,
            angle_interior: -1.0,
            deflection_interior: -1.0,
            min_size: -1.0,
            in_parallel: false,
            relative: false,
            internal_vertices_mode: true,
            control_surface_deflection: true,
            enable_control_surface_deflection_all_surfaces: false,
            clean_model: true,
            adjust_min_size: false,
            force_face_deflection: false,
            allow_quality_decrease: false,
        }
    }
}
impl MeshParameters {
    /// Native default minimum-size multiplier; `MeshParameters::rel_min_size() == 0.1`.
    pub const fn rel_min_size() -> f64 {
        0.1
    }
}
