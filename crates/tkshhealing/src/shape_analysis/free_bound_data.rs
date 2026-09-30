// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_FreeBoundData.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAnalysis_FreeBoundData`: a free bound and its properties (docs/shhealing-design.md
//! section 4).
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, HSequence};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

/// The fields of a `ShapeAnalysis_FreeBoundData`.
struct FreeBoundFields {
    bound: Wire,
    area: f64,
    perimeter: f64,
    ratio: f64,
    width: f64,
    /// `myNotches`: shared with the handle `Notches` returns.
    notches: Handle<HSequence<Shape>>,
    /// `myNotchesParams`: the width of each notch.
    notch_widths: DataMap<Shape, f64, ShapeMapHasher>,
}

/// `ShapeAnalysis_FreeBoundData`: a free bound (a wire) with its area, perimeter, ratio
/// of average length to average width, average width, and its notches with their
/// maximal widths. It only stores what `ShapeAnalysis_FreeBoundsProperties` computes;
/// the properties are -1 until set.
///
/// ```
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisFreeBoundData;
/// let data = ShapeAnalysisFreeBoundData::new();
/// assert_eq!(data.area(), -1.0);
/// data.set_area(2.5);
/// assert_eq!(data.area(), 2.5);
/// assert_eq!(data.nb_notches(), 0);
/// ```
pub struct ShapeAnalysisFreeBoundData {
    parent: StandardTransient,
    fields: SharedState<FreeBoundFields>,
}
impl_transient!(
    ShapeAnalysisFreeBoundData,
    "ShapeAnalysis_FreeBoundData",
    StandardTransient,
    parent
);

impl Default for ShapeAnalysisFreeBoundData {
    /// `ShapeAnalysis_FreeBoundData()`: a null bound and cleared properties.
    fn default() -> Self {
        let fields = FreeBoundFields {
            bound: Wire::default(),
            area: -1.0,
            perimeter: -1.0,
            ratio: -1.0,
            width: -1.0,
            notches: Handle::new(HSequence::default()),
            notch_widths: DataMap::default(),
        };
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

fn no_notch(member: &str, index: i32, count: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!("{member}: notch {index}; expected 1..={count}"),
    )
}

impl ShapeAnalysisFreeBoundData {
    /// `ShapeAnalysis_FreeBoundData()`; `ShapeAnalysisFreeBoundData::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ShapeAnalysis_FreeBoundData(freebound)`; `ShapeAnalysisFreeBoundData::with_bound(&wire)`.
    pub fn with_bound(free_bound: &Wire) -> Self {
        let data = Self::default();
        data.set_free_bound(free_bound);
        data
    }

    /// `Clear()`: resets the properties to -1 and forgets the notches (the bound stays);
    /// `data.clear()`.
    pub fn clear(&self) {
        let mut fields = self.fields.write();
        (fields.area, fields.perimeter, fields.ratio, fields.width) = (-1.0, -1.0, -1.0, -1.0);
        if let Some(notches) = fields.notches.get() {
            notches.change_sequence(|s| s.clear(None));
        }
        fields.notch_widths.clear(true);
    }

    /// `SetFreeBound(freebound)`; `data.set_free_bound(&wire)`.
    pub fn set_free_bound(&self, free_bound: &Wire) {
        self.fields.write().bound = free_bound.clone();
    }

    /// `SetArea(area)`; `data.set_area(1.0)`.
    pub fn set_area(&self, area: f64) {
        self.fields.write().area = area;
    }

    /// `SetPerimeter(perimeter)`; `data.set_perimeter(4.0)`.
    pub fn set_perimeter(&self, perimeter: f64) {
        self.fields.write().perimeter = perimeter;
    }

    /// `SetRatio(ratio)`; `data.set_ratio(2.0)`.
    pub fn set_ratio(&self, ratio: f64) {
        self.fields.write().ratio = ratio;
    }

    /// `SetWidth(width)`; `data.set_width(0.5)`.
    pub fn set_width(&self, width: f64) {
        self.fields.write().width = width;
    }

    /// `AddNotch(notch, width)`: adds a notch with its maximal width, unless the same
    /// wire (IsSame) is already a notch; `data.add_notch(&wire, 0.1)`.
    pub fn add_notch(&self, notch: &Wire, width: f64) {
        let mut fields = self.fields.write();
        if fields.notch_widths.is_bound(notch) {
            return;
        }
        if let Some(notches) = fields.notches.get() {
            notches.append(notch.clone().into_shape());
        }
        fields.notch_widths.bind(notch.clone().into_shape(), width);
    }

    /// `FreeBound()`; `data.free_bound()`.
    pub fn free_bound(&self) -> Wire {
        self.fields.read().bound.clone()
    }

    /// `Area()`; `data.area()`.
    pub fn area(&self) -> f64 {
        self.fields.read().area
    }

    /// `Perimeter()`; `data.perimeter()`.
    pub fn perimeter(&self) -> f64 {
        self.fields.read().perimeter
    }

    /// `Ratio()`; `data.ratio()`.
    pub fn ratio(&self) -> f64 {
        self.fields.read().ratio
    }

    /// `Width()`; `data.width()`.
    pub fn width(&self) -> f64 {
        self.fields.read().width
    }

    /// `NbNotches()`; `data.nb_notches()`.
    pub fn nb_notches(&self) -> i32 {
        let notches = self.fields.read().notches.clone();
        notches.get().map_or(0, |n| n.sequence(|s| s.length()))
    }

    /// `Notches()`: the notch sequence, shared with this object; `data.notches()`.
    pub fn notches(&self) -> Handle<HSequence<Shape>> {
        self.fields.read().notches.clone()
    }

    /// `Notch(index)`: the notch of rank `index`; Standard_OutOfRange outside the
    /// notches; `data.notch(1)?`.
    pub fn notch(&self, index: i32) -> Result<Wire, Failure> {
        let notches = self.fields.read().notches.clone();
        let count = self.nb_notches();
        let shape = notches
            .get()
            .and_then(|n| n.sequence(|s| s.value(index.max(0) as usize).ok().cloned()))
            .filter(|_| index >= 1)
            .ok_or_else(|| no_notch("ShapeAnalysis_FreeBoundData::Notch", index, count))?;
        topods::wire(&shape)
    }

    /// `NotchWidth(index)`: the width of the notch of rank `index`;
    /// Standard_OutOfRange outside the notches; `data.notch_width(1)?`.
    pub fn notch_width(&self, index: i32) -> Result<f64, Failure> {
        let notch = self.notch(index)?;
        self.notch_width_of(&notch)
    }

    /// `NotchWidth(notch)`: the width of a notch; Standard_NoSuchObject when the wire is
    /// not a notch; `data.notch_width_of(&wire)?`.
    pub fn notch_width_of(&self, notch: &Wire) -> Result<f64, Failure> {
        let fields = self.fields.read();
        fields.notch_widths.find(notch).copied()
    }
}
