// Rust translation of ChFiDS_SurfData.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The data of one fillet surface between two faces.
use super::errors::standard_failure;
use super::shared::Shared;
use super::{CommonPoint, FaceInterference};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

/// A shared, mutable `occ::handle<ChFiDS_SurfData>`; a null native handle is `None`.
pub type SurfDataHandle = Shared<SurfData>;

/// Every information about one fillet surface and the two faces it joins
/// (`ChFiDS_SurfData`): the data-structure indices of the faces (or of their curves), of the
/// fillet surface, its orientation, the two face interferences, the four extremity points,
/// the 2d extremities, the spine range and the extensions.
///
/// The `Change*` accessors of the native class are the `&mut` accessors here.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::SurfData;
/// let mut data = SurfData::new();
/// data.set_index_of_c1(3);
/// assert_eq!(data.index_of_c1()?, 3);
/// data.set_index_of_c1(0);
/// assert!(data.index_of_c1().is_err());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct SurfData {
    parent: StandardTransient,
    first_on_s1: CommonPoint,
    last_on_s1: CommonPoint,
    first_on_s2: CommonPoint,
    last_on_s2: CommonPoint,
    interference1: FaceInterference,
    interference2: FaceInterference,
    first_2d_on_s1: Point2,
    last_2d_on_s1: Point2,
    first_2d_on_s2: Point2,
    last_2d_on_s2: Point2,
    first_spine_param: f64,
    last_spine_param: f64,
    first_extension: f64,
    last_extension: f64,
    simul: Handle<StandardTransient>,
    index_of_s1: i32,
    index_of_c1: i32,
    index_of_s2: i32,
    index_of_c2: i32,
    index_of_conge: i32,
    is_on_curve1: bool,
    is_on_curve2: bool,
    twist_on_s1: bool,
    twist_on_s2: bool,
    orientation: Orientation,
}
impl_transient!(SurfData, "ChFiDS_SurfData", StandardTransient, parent);

impl Default for SurfData {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            first_on_s1: CommonPoint::default(),
            last_on_s1: CommonPoint::default(),
            first_on_s2: CommonPoint::default(),
            last_on_s2: CommonPoint::default(),
            interference1: FaceInterference::default(),
            interference2: FaceInterference::default(),
            first_2d_on_s1: Point2::default(),
            last_2d_on_s1: Point2::default(),
            first_2d_on_s2: Point2::default(),
            last_2d_on_s2: Point2::default(),
            first_spine_param: 0.0,
            last_spine_param: 0.0,
            first_extension: 0.0,
            last_extension: 0.0,
            simul: Handle::default(),
            index_of_s1: 0,
            index_of_c1: 0,
            index_of_s2: 0,
            index_of_c2: 0,
            index_of_conge: 0,
            is_on_curve1: false,
            is_on_curve2: false,
            twist_on_s1: false,
            twist_on_s2: false,
            orientation: Orientation::Forward,
        }
    }
}

impl SurfData {
    /// Zero indices, FORWARD orientation, empty interferences and points; `SurfData::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Copy(Other)`: copies everything except the curve indices and their flags, as
    /// natively (`indexOfC1/C2` and `isoncurv1/2` are not copied); `data.copy_from(&other)`.
    pub fn copy_from(&mut self, other: &SurfData) {
        self.index_of_s1 = other.index_of_s1;
        self.index_of_s2 = other.index_of_s2;
        self.index_of_conge = other.index_of_conge;
        self.orientation = other.orientation;
        self.interference1 = other.interference1.clone();
        self.interference2 = other.interference2.clone();
        self.copy_points_from(other);
        self.first_spine_param = other.first_spine_param;
        self.last_spine_param = other.last_spine_param;
        self.simul = other.simul.clone();
        self.first_extension = other.first_extension;
        self.last_extension = other.last_extension;
        self.twist_on_s1 = other.twist_on_s1;
        self.twist_on_s2 = other.twist_on_s2;
    }

    fn copy_points_from(&mut self, other: &SurfData) {
        self.first_on_s1 = other.first_on_s1.clone();
        self.last_on_s1 = other.last_on_s1.clone();
        self.first_on_s2 = other.first_on_s2.clone();
        self.last_on_s2 = other.last_on_s2.clone();
        self.first_2d_on_s1 = other.first_2d_on_s1;
        self.last_2d_on_s1 = other.last_2d_on_s1;
        self.first_2d_on_s2 = other.first_2d_on_s2;
        self.last_2d_on_s2 = other.last_2d_on_s2;
    }

    /// `IndexOfS1()`; `data.index_of_s1()`.
    pub fn index_of_s1(&self) -> i32 {
        self.index_of_s1
    }

    /// `IndexOfS2()`; `data.index_of_s2()`.
    pub fn index_of_s2(&self) -> i32 {
        self.index_of_s2
    }

    /// `IsOnCurve1()`; `data.is_on_curve1()`.
    pub fn is_on_curve1(&self) -> bool {
        self.is_on_curve1
    }

    /// `IsOnCurve2()`; `data.is_on_curve2()`.
    pub fn is_on_curve2(&self) -> bool {
        self.is_on_curve2
    }

    /// `IndexOfC1()`: Standard_Failure when the first interference is not on a curve;
    /// `data.index_of_c1()?`.
    pub fn index_of_c1(&self) -> Result<i32, Failure> {
        curve_index(self.is_on_curve1, self.index_of_c1)
    }

    /// `IndexOfC2()`: Standard_Failure when the second interference is not on a curve;
    /// `data.index_of_c2()?`.
    pub fn index_of_c2(&self) -> Result<i32, Failure> {
        curve_index(self.is_on_curve2, self.index_of_c2)
    }

    /// `Surf()`: the index of the fillet surface; `data.surf()`.
    pub fn surf(&self) -> i32 {
        self.index_of_conge
    }

    /// `Orientation()`; `data.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// `InterferenceOnS1()`; `data.interference_on_s1()`.
    pub fn interference_on_s1(&self) -> &FaceInterference {
        &self.interference1
    }

    /// `InterferenceOnS2()`; `data.interference_on_s2()`.
    pub fn interference_on_s2(&self) -> &FaceInterference {
        &self.interference2
    }

    /// `VertexFirstOnS1()`; `data.vertex_first_on_s1()`.
    pub fn vertex_first_on_s1(&self) -> &CommonPoint {
        &self.first_on_s1
    }

    /// `VertexFirstOnS2()`; `data.vertex_first_on_s2()`.
    pub fn vertex_first_on_s2(&self) -> &CommonPoint {
        &self.first_on_s2
    }

    /// `VertexLastOnS1()`; `data.vertex_last_on_s1()`.
    pub fn vertex_last_on_s1(&self) -> &CommonPoint {
        &self.last_on_s1
    }

    /// `VertexLastOnS2()`; `data.vertex_last_on_s2()`.
    pub fn vertex_last_on_s2(&self) -> &CommonPoint {
        &self.last_on_s2
    }

    /// `ChangeIndexOfS1(Index)`; `data.change_index_of_s1(2)`.
    pub fn change_index_of_s1(&mut self, index: i32) {
        self.index_of_s1 = index;
    }

    /// `ChangeIndexOfS2(Index)`; `data.change_index_of_s2(2)`.
    pub fn change_index_of_s2(&mut self, index: i32) {
        self.index_of_s2 = index;
    }

    /// `ChangeSurf(Index)`; `data.change_surf(5)`.
    pub fn change_surf(&mut self, index: i32) {
        self.index_of_conge = index;
    }

    /// `SetIndexOfC1(Index)`: the curve index; the interference is on a curve when it is
    /// not 0; `data.set_index_of_c1(1)`.
    pub fn set_index_of_c1(&mut self, index: i32) {
        self.index_of_c1 = index;
        self.is_on_curve1 = index != 0;
    }

    /// `SetIndexOfC2(Index)`: as [`SurfData::set_index_of_c1`] for the second face;
    /// `data.set_index_of_c2(1)`.
    pub fn set_index_of_c2(&mut self, index: i32) {
        self.index_of_c2 = index;
        self.is_on_curve2 = index != 0;
    }

    /// `ChangeOrientation()`; `*data.change_orientation() = Orientation::Reversed`.
    pub fn change_orientation(&mut self) -> &mut Orientation {
        &mut self.orientation
    }

    /// `ChangeInterferenceOnS1()`; `data.change_interference_on_s1().set_line_index(1)`.
    pub fn change_interference_on_s1(&mut self) -> &mut FaceInterference {
        &mut self.interference1
    }

    /// `ChangeInterferenceOnS2()`; `data.change_interference_on_s2().set_line_index(1)`.
    pub fn change_interference_on_s2(&mut self) -> &mut FaceInterference {
        &mut self.interference2
    }

    /// `ChangeVertexFirstOnS1()`; `data.change_vertex_first_on_s1().reset()`.
    pub fn change_vertex_first_on_s1(&mut self) -> &mut CommonPoint {
        &mut self.first_on_s1
    }

    /// `ChangeVertexFirstOnS2()`; `data.change_vertex_first_on_s2().reset()`.
    pub fn change_vertex_first_on_s2(&mut self) -> &mut CommonPoint {
        &mut self.first_on_s2
    }

    /// `ChangeVertexLastOnS1()`; `data.change_vertex_last_on_s1().reset()`.
    pub fn change_vertex_last_on_s1(&mut self) -> &mut CommonPoint {
        &mut self.last_on_s1
    }

    /// `ChangeVertexLastOnS2()`; `data.change_vertex_last_on_s2().reset()`.
    pub fn change_vertex_last_on_s2(&mut self) -> &mut CommonPoint {
        &mut self.last_on_s2
    }

    /// `Interference(OnS)`: the first interference for `on_s == 1`, else the second;
    /// `data.interference(1)`.
    pub fn interference(&self, on_s: i32) -> &FaceInterference {
        if on_s == 1 {
            &self.interference1
        } else {
            &self.interference2
        }
    }

    /// `ChangeInterference(OnS)`; `data.change_interference(2).set_line_index(1)`.
    pub fn change_interference(&mut self, on_s: i32) -> &mut FaceInterference {
        if on_s == 1 {
            &mut self.interference1
        } else {
            &mut self.interference2
        }
    }

    /// `Index(OfS)`: the first face index for `of_s == 1`, else the second; `data.index(1)`.
    pub fn index(&self, of_s: i32) -> i32 {
        if of_s == 1 {
            self.index_of_s1
        } else {
            self.index_of_s2
        }
    }

    /// `Vertex(First, OnS)`: one of the four extremity points; as natively, an `on_s`
    /// other than 1 and 2 selects the last point on the second face; `data.vertex(true, 1)`.
    pub fn vertex(&self, first: bool, on_s: i32) -> &CommonPoint {
        match (first, on_s) {
            (true, 1) => &self.first_on_s1,
            (true, 2) => &self.first_on_s2,
            (false, 1) => &self.last_on_s1,
            _ => &self.last_on_s2,
        }
    }

    /// `ChangeVertex(First, OnS)`, with the selection of [`SurfData::vertex`];
    /// `data.change_vertex(false, 2).reset()`.
    pub fn change_vertex(&mut self, first: bool, on_s: i32) -> &mut CommonPoint {
        match (first, on_s) {
            (true, 1) => &mut self.first_on_s1,
            (true, 2) => &mut self.first_on_s2,
            (false, 1) => &mut self.last_on_s1,
            _ => &mut self.last_on_s2,
        }
    }

    /// `IsOnCurve(OnS)`: the first flag for `on_s == 1`, else the second;
    /// `data.is_on_curve(2)`.
    pub fn is_on_curve(&self, on_s: i32) -> bool {
        if on_s == 1 {
            self.is_on_curve1
        } else {
            self.is_on_curve2
        }
    }

    /// `IndexOfC(OnS)`: Standard_Failure when that interference is not on a curve;
    /// `data.index_of_c(1)?`.
    pub fn index_of_c(&self, on_s: i32) -> Result<i32, Failure> {
        if on_s == 1 {
            return self.index_of_c1();
        }
        self.index_of_c2()
    }

    /// `FirstSpineParam()`; `data.first_spine_param()`.
    pub fn first_spine_param(&self) -> f64 {
        self.first_spine_param
    }

    /// `LastSpineParam()`; `data.last_spine_param()`.
    pub fn last_spine_param(&self) -> f64 {
        self.last_spine_param
    }

    /// `FirstSpineParam(Par)`; `data.set_first_spine_param(0.0)`.
    pub fn set_first_spine_param(&mut self, parameter: f64) {
        self.first_spine_param = parameter;
    }

    /// `LastSpineParam(Par)`; `data.set_last_spine_param(1.0)`.
    pub fn set_last_spine_param(&mut self, parameter: f64) {
        self.last_spine_param = parameter;
    }

    /// `FirstExtensionValue()`; `data.first_extension_value()`.
    pub fn first_extension_value(&self) -> f64 {
        self.first_extension
    }

    /// `LastExtensionValue()`; `data.last_extension_value()`.
    pub fn last_extension_value(&self) -> f64 {
        self.last_extension
    }

    /// `FirstExtensionValue(Extend)`; `data.set_first_extension_value(0.1)`.
    pub fn set_first_extension_value(&mut self, extension: f64) {
        self.first_extension = extension;
    }

    /// `LastExtensionValue(Extend)`; `data.set_last_extension_value(0.1)`.
    pub fn set_last_extension_value(&mut self, extension: f64) {
        self.last_extension = extension;
    }

    /// `Simul()`: the simulation object of the surface; `data.simul()`.
    pub fn simul(&self) -> Handle<StandardTransient> {
        self.simul.clone()
    }

    /// `SetSimul(S)`; `data.set_simul(&object)`.
    pub fn set_simul(&mut self, simul: &Handle<StandardTransient>) {
        self.simul = simul.clone();
    }

    /// `ResetSimul()`: nullifies the simulation object; `data.reset_simul()`.
    pub fn reset_simul(&mut self) {
        self.simul = Handle::default();
    }

    /// `Get2dPoints(First, OnS)`: one 2d extremity; `data.get_2d_point(true, 1)`.
    pub fn get_2d_point(&self, first: bool, on_s: i32) -> Point2 {
        match (first, on_s) {
            (true, 1) => self.first_2d_on_s1,
            (false, 1) => self.last_2d_on_s1,
            (true, 2) => self.first_2d_on_s2,
            _ => self.last_2d_on_s2,
        }
    }

    /// `Get2dPoints(P2df1, P2dl1, P2df2, P2dl2)`: the four 2d extremities;
    /// `let [f1, l1, f2, l2] = data.get_2d_points();`.
    pub fn get_2d_points(&self) -> [Point2; 4] {
        [
            self.first_2d_on_s1,
            self.last_2d_on_s1,
            self.first_2d_on_s2,
            self.last_2d_on_s2,
        ]
    }

    /// `Set2dPoints(P2df1, P2dl1, P2df2, P2dl2)`; `data.set_2d_points([p; 4])`.
    pub fn set_2d_points(&mut self, points: [Point2; 4]) {
        [
            self.first_2d_on_s1,
            self.last_2d_on_s1,
            self.first_2d_on_s2,
            self.last_2d_on_s2,
        ] = points;
    }

    /// `TwistOnS1()`; `data.twist_on_s1()`.
    pub fn twist_on_s1(&self) -> bool {
        self.twist_on_s1
    }

    /// `TwistOnS2()`; `data.twist_on_s2()`.
    pub fn twist_on_s2(&self) -> bool {
        self.twist_on_s2
    }

    /// `TwistOnS1(T)`; `data.set_twist_on_s1(true)`.
    pub fn set_twist_on_s1(&mut self, twist: bool) {
        self.twist_on_s1 = twist;
    }

    /// `TwistOnS2(T)`; `data.set_twist_on_s2(true)`.
    pub fn set_twist_on_s2(&mut self, twist: bool) {
        self.twist_on_s2 = twist;
    }
}

/// The curve index, or the native "Interference pas sur courbe" Standard_Failure.
fn curve_index(is_on_curve: bool, index: i32) -> Result<i32, Failure> {
    if is_on_curve {
        return Ok(index);
    }
    Err(standard_failure("Interference pas sur courbe"))
}

#[cfg(test)]
#[path = "surf_data_tests.rs"]
mod tests;
