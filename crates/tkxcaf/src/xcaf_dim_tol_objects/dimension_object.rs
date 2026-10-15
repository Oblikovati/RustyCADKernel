// Rust port of DataExchange/TKXCAF/XCAFDimTolObjects/XCAFDimTolObjects_DimensionObject.hxx and
// .cxx (the value, tolerance and classification members) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDimTolObjects_DimensionObject`: the description of a dimension (type, value or
//! range or plus-minus tolerance, qualifiers, class of tolerance, modifiers, connections and
//! presentation).
use super::{
    XCAFDimTolObjectsAngularQualifier, XCAFDimTolObjectsDimensionFormVariance,
    XCAFDimTolObjectsDimensionGrade, XCAFDimTolObjectsDimensionModif,
    XCAFDimTolObjectsDimensionQualifier, XCAFDimTolObjectsDimensionType,
};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};

/// The fields of `XCAFDimTolObjects_DimensionObject`. The native default constructor leaves
/// the type, qualifiers, hole flag, class of tolerance and decimal places uninitialized; the
/// OCCT allocator clears memory, so they read zero, as here.
#[derive(Clone, Default)]
pub(super) struct DimensionFields {
    pub(super) kind: XCAFDimTolObjectsDimensionType,
    pub(super) values: Handle<HArray1<f64>>,
    pub(super) qualifier: XCAFDimTolObjectsDimensionQualifier,
    pub(super) angular_qualifier: XCAFDimTolObjectsAngularQualifier,
    pub(super) is_hole: bool,
    pub(super) form_variance: XCAFDimTolObjectsDimensionFormVariance,
    pub(super) grade: XCAFDimTolObjectsDimensionGrade,
    pub(super) left: i32,
    pub(super) right: i32,
    pub(super) modifiers: Vec<XCAFDimTolObjectsDimensionModif>,
    pub(super) path: Edge,
    pub(super) direction: Direction3,
    pub(super) connection1: Axis2,
    pub(super) connection2: Axis2,
    pub(super) has_connection1: bool,
    pub(super) has_connection2: bool,
    pub(super) connection_is_point1: bool,
    pub(super) connection_is_point2: bool,
    pub(super) connection_name1: Handle<HAsciiString>,
    pub(super) connection_name2: Handle<HAsciiString>,
    pub(super) plane: Axis2,
    pub(super) has_plane: bool,
    pub(super) has_point_text: bool,
    pub(super) point_text: Point3,
    pub(super) presentation: Shape,
    pub(super) semantic_name: Handle<HAsciiString>,
    pub(super) presentation_name: Handle<HAsciiString>,
    pub(super) descriptions: Vec<Handle<HAsciiString>>,
    pub(super) description_names: Vec<Handle<HAsciiString>>,
}

/// `XCAFDimTolObjects_DimensionObject`.
///
/// ```
/// use rustycad_tkxcaf::xcaf_dim_tol_objects::XCAFDimTolObjectsDimensionObject;
/// let dimension = XCAFDimTolObjectsDimensionObject::new();
/// dimension.set_value(2.0);
/// assert!(dimension.set_upper_tol_value(0.1));
/// assert!(dimension.is_dim_with_plus_minus_tolerance());
/// assert_eq!(dimension.get_upper_tol_value(), 0.1);
/// ```
#[derive(Default)]
pub struct XCAFDimTolObjectsDimensionObject {
    transient: StandardTransient,
    pub(super) fields: SharedState<DimensionFields>,
}
rustycad_tkernel::impl_transient!(
    XCAFDimTolObjectsDimensionObject,
    "XCAFDimTolObjects_DimensionObject",
    StandardTransient,
    transient
);

/// A new shared real array `1..=values.len()` holding `values`.
pub(crate) fn real_array(values: &[f64]) -> Handle<HArray1<f64>> {
    let array = HArray1::with_bounds(1, values.len() as i32).unwrap_or_default();
    array.change_array1().as_mut_slice().copy_from_slice(values);
    Handle::new(array)
}

/// The values of a shared real array, `None` when the handle is null.
pub(crate) fn array_values(array: &Handle<HArray1<f64>>) -> Option<Vec<f64>> {
    array.get().map(|array| array.array1().as_slice().to_vec())
}

impl XCAFDimTolObjectsDimensionObject {
    /// `XCAFDimTolObjects_DimensionObject()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XCAFDimTolObjects_DimensionObject(theObj)`: a copy of every field (the value array is
    /// shared, as the native handle copy; a null object gives a default object).
    pub fn from_object(object: &Handle<XCAFDimTolObjectsDimensionObject>) -> Self {
        let fields = object
            .get()
            .map(|o| o.fields.read().clone())
            .unwrap_or_default();
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
    /// The current values (empty for a null array).
    fn values(&self) -> Vec<f64> {
        array_values(&self.fields.read().values).unwrap_or_default()
    }
    /// Writes one value of the shared array (`myVal->SetValue(index, value)`).
    fn set_shared_value(&self, index: usize, value: f64) {
        let array = self.fields.read().values.clone();
        if let Some(array) = array.get() {
            array.change_array1().as_mut_slice()[index - 1] = value;
        }
    }
    /// Replaces the array by a new one with these values.
    fn replace_values(&self, values: &[f64]) {
        self.fields.write().values = real_array(values);
    }
    /// `GetSemanticName`.
    pub fn get_semantic_name(&self) -> Handle<HAsciiString> {
        self.fields.read().semantic_name.clone()
    }
    /// `SetSemanticName`.
    pub fn set_semantic_name(&self, name: &Handle<HAsciiString>) {
        self.fields.write().semantic_name = name.clone();
    }
    /// `SetQualifier`.
    pub fn set_qualifier(&self, qualifier: XCAFDimTolObjectsDimensionQualifier) {
        self.fields.write().qualifier = qualifier;
    }
    /// `GetQualifier`.
    pub fn get_qualifier(&self) -> XCAFDimTolObjectsDimensionQualifier {
        self.fields.read().qualifier
    }
    /// `HasQualifier`: a qualifier other than `None`.
    pub fn has_qualifier(&self) -> bool {
        self.get_qualifier() != XCAFDimTolObjectsDimensionQualifier::None
    }
    /// `SetAngularQualifier`.
    pub fn set_angular_qualifier(&self, qualifier: XCAFDimTolObjectsAngularQualifier) {
        self.fields.write().angular_qualifier = qualifier;
    }
    /// `GetAngularQualifier`.
    pub fn get_angular_qualifier(&self) -> XCAFDimTolObjectsAngularQualifier {
        self.fields.read().angular_qualifier
    }
    /// `HasAngularQualifier`: an angular qualifier other than `None`.
    pub fn has_angular_qualifier(&self) -> bool {
        self.get_angular_qualifier() != XCAFDimTolObjectsAngularQualifier::None
    }
    /// `SetType`.
    pub fn set_type(&self, kind: XCAFDimTolObjectsDimensionType) {
        self.fields.write().kind = kind;
    }
    /// `GetType`.
    pub fn get_type(&self) -> XCAFDimTolObjectsDimensionType {
        self.fields.read().kind
    }
}

/// The value, range and plus-minus tolerance (one, two or three values).
impl XCAFDimTolObjectsDimensionObject {
    /// `GetValue`: the value (one or three values), the middle of a range, else 0.
    pub fn get_value(&self) -> f64 {
        match self.values().as_slice() {
            [value] | [value, _, _] => *value,
            [lower, upper] => (lower + upper) / 2.0,
            _ => 0.0,
        }
    }
    /// `GetValues`: the shared value array (null when unset).
    pub fn get_values(&self) -> Handle<HArray1<f64>> {
        self.fields.read().values.clone()
    }
    /// `SetValue`: a new one-value array.
    pub fn set_value(&self, value: f64) {
        self.replace_values(&[value]);
    }
    /// `SetValues`: shares the given array.
    pub fn set_values(&self, values: &Handle<HArray1<f64>>) {
        self.fields.write().values = values.clone();
    }
    /// `IsDimWithRange`: two values.
    pub fn is_dim_with_range(&self) -> bool {
        self.values().len() == 2
    }
    /// `SetUpperBound`: the second value of an array of two or more values (natively the
    /// lower tolerance of a three-value array), else a new range of the bound twice.
    pub fn set_upper_bound(&self, bound: f64) {
        if self.values().len() > 1 {
            self.set_shared_value(2, bound);
        } else {
            self.replace_values(&[bound, bound]);
        }
    }
    /// `SetLowerBound`: the first value of an array of two or more values, else a new range
    /// of the bound twice.
    pub fn set_lower_bound(&self, bound: f64) {
        if self.values().len() > 1 {
            self.set_shared_value(1, bound);
        } else {
            self.replace_values(&[bound, bound]);
        }
    }
    /// `GetUpperBound`: the second value of a range, else 0.
    pub fn get_upper_bound(&self) -> f64 {
        match self.values().as_slice() {
            [_, upper] => *upper,
            _ => 0.0,
        }
    }
    /// `GetLowerBound`: the first value of a range, else 0.
    pub fn get_lower_bound(&self) -> f64 {
        match self.values().as_slice() {
            [lower, _] => *lower,
            _ => 0.0,
        }
    }
    /// `IsDimWithPlusMinusTolerance`: three values.
    pub fn is_dim_with_plus_minus_tolerance(&self) -> bool {
        self.values().len() == 3
    }
    /// `SetUpperTolValue`: the third value of a tolerance, or a new tolerance of a single
    /// value with both tolerances set to it; false otherwise.
    pub fn set_upper_tol_value(&self, value: f64) -> bool {
        self.set_tolerance(3, value)
    }
    /// `SetLowerTolValue`: the second value of a tolerance, or a new tolerance of a single
    /// value with both tolerances set to it; false otherwise.
    pub fn set_lower_tol_value(&self, value: f64) -> bool {
        self.set_tolerance(2, value)
    }
    /// The common body of the tolerance setters.
    fn set_tolerance(&self, index: usize, tolerance: f64) -> bool {
        match self.values().as_slice() {
            [_, _, _] => self.set_shared_value(index, tolerance),
            [value] => self.replace_values(&[*value, tolerance, tolerance]),
            _ => return false,
        }
        true
    }
    /// `GetUpperTolValue`: the third value of a tolerance, else 0.
    pub fn get_upper_tol_value(&self) -> f64 {
        match self.values().as_slice() {
            [_, _, upper] => *upper,
            _ => 0.0,
        }
    }
    /// `GetLowerTolValue`: the second value of a tolerance, else 0.
    pub fn get_lower_tol_value(&self) -> f64 {
        match self.values().as_slice() {
            [_, lower, _] => *lower,
            _ => 0.0,
        }
    }
}

/// The class of tolerance, decimal places, modifiers, path and direction.
impl XCAFDimTolObjectsDimensionObject {
    /// `IsDimWithClassOfTolerance`: a form variance other than `None`.
    pub fn is_dim_with_class_of_tolerance(&self) -> bool {
        self.fields.read().form_variance != XCAFDimTolObjectsDimensionFormVariance::None
    }
    /// `SetClassOfTolerance(theHole, theFormVariance, theGrade)`.
    pub fn set_class_of_tolerance(
        &self,
        hole: bool,
        form_variance: XCAFDimTolObjectsDimensionFormVariance,
        grade: XCAFDimTolObjectsDimensionGrade,
    ) {
        let mut fields = self.fields.write();
        (fields.is_hole, fields.form_variance, fields.grade) = (hole, form_variance, grade);
    }
    /// `GetClassOfTolerance(theHole, theFormVariance, theGrade)`: the form variance always;
    /// the hole flag and the grade only when there is a class (then true).
    pub fn get_class_of_tolerance(
        &self,
        hole: &mut bool,
        form_variance: &mut XCAFDimTolObjectsDimensionFormVariance,
        grade: &mut XCAFDimTolObjectsDimensionGrade,
    ) -> bool {
        let fields = self.fields.read();
        *form_variance = fields.form_variance;
        if fields.form_variance == XCAFDimTolObjectsDimensionFormVariance::None {
            return false;
        }
        (*hole, *grade) = (fields.is_hole, fields.grade);
        true
    }
    /// `SetNbOfDecimalPlaces(theL, theR)`.
    pub fn set_nb_of_decimal_places(&self, left: i32, right: i32) {
        let mut fields = self.fields.write();
        (fields.left, fields.right) = (left, right);
    }
    /// `GetNbOfDecimalPlaces(theL, theR)`.
    pub fn get_nb_of_decimal_places(&self) -> (i32, i32) {
        let fields = self.fields.read();
        (fields.left, fields.right)
    }
    /// `GetModifiers`: a copy of the modifiers.
    pub fn get_modifiers(&self) -> Vec<XCAFDimTolObjectsDimensionModif> {
        self.fields.read().modifiers.clone()
    }
    /// `SetModifiers`.
    pub fn set_modifiers(&self, modifiers: &[XCAFDimTolObjectsDimensionModif]) {
        self.fields.write().modifiers = modifiers.to_vec();
    }
    /// `AddModifier`: appends a modifier.
    pub fn add_modifier(&self, modifier: XCAFDimTolObjectsDimensionModif) {
        self.fields.write().modifiers.push(modifier);
    }
    /// `GetPath`.
    pub fn get_path(&self) -> Edge {
        self.fields.read().path.clone()
    }
    /// `SetPath`: ignores a null edge.
    pub fn set_path(&self, path: &Edge) {
        if !path.is_null() {
            self.fields.write().path = path.clone();
        }
    }
    /// `GetDirection(theDir)`: always true.
    pub fn get_direction(&self, direction: &mut Direction3) -> bool {
        *direction = self.fields.read().direction;
        true
    }
    /// `SetDirection(theDir)`: always true.
    pub fn set_direction(&self, direction: &Direction3) -> bool {
        self.fields.write().direction = *direction;
        true
    }
    /// `IsDimensionalLocation`: the location types (`Location_None` to `Location_Oriented`,
    /// except `Location_Angular` and `Location_WithPath`).
    pub fn is_dimensional_location(kind: XCAFDimTolObjectsDimensionType) -> bool {
        use XCAFDimTolObjectsDimensionType as T;
        (kind as i32) <= (T::LocationLinearDistanceFromInnerToInner as i32)
            || kind == T::LocationOriented
    }
    /// `IsDimensionalSize`: the size types `Size_CurveLength` to `Size_Thickness`.
    pub fn is_dimensional_size(kind: XCAFDimTolObjectsDimensionType) -> bool {
        use XCAFDimTolObjectsDimensionType as T;
        (T::SizeCurveLength as i32..=T::SizeThickness as i32).contains(&(kind as i32))
    }
}
