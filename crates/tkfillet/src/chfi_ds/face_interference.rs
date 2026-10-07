// Rust translation of ChFiDS_FaceInterference.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The interference of a fillet with one of its support faces.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;

/// The contact line of a fillet on a face (`ChFiDS_FaceInterference`): the index of the 3d
/// line in the data structure, its transition, its pcurves on the face and on the fillet
/// surface, and its parameter range.
///
/// The `Change*` accessors of the native class are the `&mut` accessors here.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::FaceInterference;
/// let mut interference = FaceInterference::new();
/// interference.set_parameter(2.0, false);
/// assert_eq!(interference.parameter(false), 2.0);
/// ```
#[derive(Clone)]
pub struct FaceInterference {
    first_parameter: f64,
    last_parameter: f64,
    pcurve_on_face: Handle<dyn Geom2dCurve>,
    pcurve_on_surf: Handle<dyn Geom2dCurve>,
    line_index: i32,
    line_transition: Orientation,
}

impl Default for FaceInterference {
    fn default() -> Self {
        Self {
            first_parameter: 0.0,
            last_parameter: 0.0,
            pcurve_on_face: Handle::default(),
            pcurve_on_surf: Handle::default(),
            line_index: 0,
            line_transition: Orientation::Forward,
        }
    }
}

impl FaceInterference {
    /// No line, null pcurves, a FORWARD transition and a `[0, 0]` range;
    /// `FaceInterference::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetInterference(LineIndex, Trans, PCurv1, PCurv2)`: the line index, its transition
    /// and the pcurves on the face and on the fillet surface;
    /// `interference.set_interference(1, Orientation::Forward, &on_face, &on_surf)`.
    pub fn set_interference(
        &mut self,
        line_index: i32,
        transition: Orientation,
        pcurve_on_face: &Handle<dyn Geom2dCurve>,
        pcurve_on_surf: &Handle<dyn Geom2dCurve>,
    ) {
        self.line_index = line_index;
        self.line_transition = transition;
        self.pcurve_on_face = pcurve_on_face.clone();
        self.pcurve_on_surf = pcurve_on_surf.clone();
    }

    /// `SetTransition(Trans)`; `interference.set_transition(Orientation::Reversed)`.
    pub fn set_transition(&mut self, transition: Orientation) {
        self.line_transition = transition;
    }

    /// `SetFirstParameter(U1)`; `interference.set_first_parameter(0.0)`.
    pub fn set_first_parameter(&mut self, parameter: f64) {
        self.first_parameter = parameter;
    }

    /// `SetLastParameter(U1)`; `interference.set_last_parameter(1.0)`.
    pub fn set_last_parameter(&mut self, parameter: f64) {
        self.last_parameter = parameter;
    }

    /// `SetParameter(U1, IsFirst)`: the first or the last parameter;
    /// `interference.set_parameter(1.0, true)`.
    pub fn set_parameter(&mut self, parameter: f64, is_first: bool) {
        if is_first {
            self.set_first_parameter(parameter);
        } else {
            self.set_last_parameter(parameter);
        }
    }

    /// `LineIndex()`; `interference.line_index()`.
    pub fn line_index(&self) -> i32 {
        self.line_index
    }

    /// `SetLineIndex(I)`; `interference.set_line_index(3)`.
    pub fn set_line_index(&mut self, line_index: i32) {
        self.line_index = line_index;
    }

    /// `Transition()`; `interference.transition()`.
    pub fn transition(&self) -> Orientation {
        self.line_transition
    }

    /// `PCurveOnFace()`; `interference.pcurve_on_face()`.
    pub fn pcurve_on_face(&self) -> &Handle<dyn Geom2dCurve> {
        &self.pcurve_on_face
    }

    /// `PCurveOnSurf()`; `interference.pcurve_on_surf()`.
    pub fn pcurve_on_surf(&self) -> &Handle<dyn Geom2dCurve> {
        &self.pcurve_on_surf
    }

    /// `ChangePCurveOnFace()`; `*interference.change_pcurve_on_face() = curve`.
    pub fn change_pcurve_on_face(&mut self) -> &mut Handle<dyn Geom2dCurve> {
        &mut self.pcurve_on_face
    }

    /// `ChangePCurveOnSurf()`; `*interference.change_pcurve_on_surf() = curve`.
    pub fn change_pcurve_on_surf(&mut self) -> &mut Handle<dyn Geom2dCurve> {
        &mut self.pcurve_on_surf
    }

    /// `FirstParameter()`; `interference.first_parameter()`.
    pub fn first_parameter(&self) -> f64 {
        self.first_parameter
    }

    /// `LastParameter()`; `interference.last_parameter()`.
    pub fn last_parameter(&self) -> f64 {
        self.last_parameter
    }

    /// `Parameter(IsFirst)`: the first or the last parameter; `interference.parameter(true)`.
    pub fn parameter(&self, is_first: bool) -> f64 {
        if is_first {
            self.first_parameter()
        } else {
            self.last_parameter()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_are_selected_by_the_first_flag() {
        let mut interference = FaceInterference::new();
        interference.set_parameter(-1.0, true);
        interference.set_parameter(3.0, false);
        assert_eq!(interference.first_parameter(), -1.0);
        assert_eq!(interference.last_parameter(), 3.0);
        assert_eq!(interference.parameter(true), -1.0);
    }

    #[test]
    fn set_interference_records_line_and_transition() {
        let mut interference = FaceInterference::new();
        let null: Handle<dyn Geom2dCurve> = Handle::default();
        interference.set_interference(4, Orientation::Internal, &null, &null);
        assert_eq!(interference.line_index(), 4);
        assert_eq!(interference.transition(), Orientation::Internal);
        interference.set_line_index(5);
        interference.set_transition(Orientation::External);
        assert_eq!(interference.line_index(), 5);
        assert_eq!(interference.transition(), Orientation::External);
        assert!(interference.pcurve_on_face().is_null());
        assert!(interference.change_pcurve_on_surf().is_null());
    }
}
