// Rust translation of ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade_SplitCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeUpgrade_SplitCurve`: the split values of a curve range, the base of the curve
//! splitters (docs/shhealing-design.md section 8).
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::ncollection::{HSequence, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};

/// The protected fields of a `ShapeUpgrade_SplitCurve`.
#[derive(Clone, Default)]
pub struct SplitCurveFields {
    /// `mySplitValues`: the parameters that bound the pieces, first and last included.
    pub split_values: Handle<HSequence<f64>>,
    /// `myNbCurves`.
    pub nb_curves: i32,
    /// `myStatus`: the ShapeExtend status bits.
    pub status: i32,
}

/// The fields a curve splitter embeds and lends through [`SplitCurve::split_curve_data`].
#[derive(Default)]
pub struct SplitCurveData {
    fields: SharedState<SplitCurveFields>,
}

impl SplitCurveData {
    /// Reads the fields; `data.with(|fields| fields.status)`.
    pub fn with<R>(&self, read: impl FnOnce(&SplitCurveFields) -> R) -> R {
        read(&self.fields.read())
    }

    /// Changes the fields; `data.with_mut(|fields| fields.nb_curves = 2)`.
    pub fn with_mut<R>(&self, edit: impl FnOnce(&mut SplitCurveFields) -> R) -> R {
        edit(&mut self.fields.write())
    }
}

/// `ShapeUpgrade_SplitCurve`: splits a parameter range at given values; the virtual
/// `Compute` (which finds the split values) and `Build` (which builds the pieces) do
/// nothing but reset the status here, and the 2D and 3D splitters override them.
///
/// ```
/// use rustycad_tkernel::ncollection::HSequence;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkshhealing::shape_upgrade::{ShapeUpgradeSplitCurve, SplitCurve};
/// let splitter = ShapeUpgradeSplitCurve::new();
/// splitter.init(0.0, 10.0);
/// let values = Handle::new(HSequence::default());
/// values.append(4.0);
/// splitter.set_split_values(&values).unwrap();
/// assert_eq!(splitter.split_values().sequence(|s| s.len()), 3);
/// ```
pub trait SplitCurve: Transient {
    /// The embedded protected fields.
    fn split_curve_data(&self) -> &SplitCurveData;

    /// `Init(First, Last)`: status OK, one curve, the split values `[First, Last]`;
    /// `splitter.init(0.0, 1.0)`.
    fn init(&self, first: f64, last: f64) {
        let values = Handle::new(HSequence::default());
        values.append(first);
        values.append(last);
        self.split_curve_data().with_mut(|fields| {
            fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
            fields.nb_curves = 1;
            fields.split_values = values;
        });
    }

    /// `SetSplitValues(SplitValues)`: inserts the given values (increasing) that lie
    /// inside each current interval by more than `Precision::PConfusion()`; nothing for
    /// a null or empty sequence. Standard_NullObject before `Init` (a null dereference
    /// natively); `splitter.set_split_values(&values)?`.
    fn set_split_values(&self, split_values: &Handle<HSequence<f64>>) -> Result<(), Failure> {
        let Some(given) = split_values.get() else {
            return Ok(());
        };
        if given.sequence(|s| s.is_empty()) {
            return Ok(());
        }
        let own = self.split_values();
        let own = own.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "ShapeUpgrade_SplitCurve::SetSplitValues: Init was not called",
                "",
            )
        })?;
        if std::ptr::eq(own, given) {
            // The same sequence is both read and edited natively.
            return own.change_sequence(|values| insert_split_values(values, None));
        }
        let given: Vec<f64> = given.sequence(|s| s.iter().copied().collect());
        own.change_sequence(|values| insert_split_values(values, Some(&given)))
    }

    /// `Build(Segment)` (virtual): the base class resets the status to OK;
    /// `splitter.build(true)?`.
    fn build(&self, _segment: bool) -> Result<(), Failure> {
        let ok = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        self.split_curve_data()
            .with_mut(|fields| fields.status = ok);
        Ok(())
    }

    /// `SplitValues()`: the handle of the split values (null before `Init`);
    /// `splitter.split_values()`.
    fn split_values(&self) -> Handle<HSequence<f64>> {
        self.split_curve_data()
            .with(|fields| fields.split_values.clone())
    }

    /// `Compute()` (virtual): the base class resets the status to OK;
    /// `splitter.compute()?`.
    fn compute(&self) -> Result<(), Failure> {
        let ok = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        self.split_curve_data()
            .with_mut(|fields| fields.status = ok);
        Ok(())
    }

    /// `Perform(Segment)`: `Compute()` then `Build(Segment)` (the native default
    /// `Segment` is true); `splitter.perform(true)?`.
    fn perform(&self, segment: bool) -> Result<(), Failure> {
        self.compute()?;
        self.build(segment)
    }

    /// `Status(status)`: the flag of the status bits;
    /// `splitter.status_flag(ShapeExtendStatus::Ok)`.
    fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        let bits = self.split_curve_data().with(|fields| fields.status);
        ShapeExtend::decode_status(bits, status)
    }
}
impl_transient_interface!(dyn SplitCurve, "ShapeUpgrade_SplitCurve", StandardTransient);

/// The loops of `SetSplitValues` on the current values: for each interval
/// `[First, Last]`, the given values beyond `First + PConfusion` and before
/// `Last - PConfusion` are inserted before `Last` in order (the reading index goes on
/// across the intervals). `given` is `None` when the given sequence is the current one.
pub(super) fn insert_split_values(
    values: &mut Sequence<f64>,
    given: Option<&[f64]>,
) -> Result<(), Failure> {
    let tolerance = precision::p_confusion();
    let given_value = |values: &Sequence<f64>, i: usize| -> Result<f64, Failure> {
        match given {
            Some(given) => Ok(given[i - 1]),
            None => values.value(i).copied(),
        }
    };
    let len = given.map_or(values.len(), <[f64]>::len);
    let mut first = *values.value(1)?;
    let mut i = 1usize;
    let mut k = 2usize;
    while k <= values.len() {
        let last = *values.value(k)?;
        while i <= len {
            let value = given_value(values, i)?;
            if first + tolerance >= value {
                i += 1;
                continue;
            }
            if last - tolerance <= value {
                break;
            }
            values.insert_before(k, value)?;
            k += 1;
            i += 1;
        }
        first = last;
        k += 1;
    }
    Ok(())
}

/// The base class `ShapeUpgrade_SplitCurve` itself.
#[derive(Default)]
pub struct ShapeUpgradeSplitCurve {
    parent: StandardTransient,
    data: SplitCurveData,
}
impl_transient!(
    ShapeUpgradeSplitCurve,
    "ShapeUpgrade_SplitCurve",
    StandardTransient,
    parent,
    interfaces: [dyn SplitCurve]
);

impl ShapeUpgradeSplitCurve {
    /// `ShapeUpgrade_SplitCurve()`: no split values, no curve, status 0;
    /// `ShapeUpgradeSplitCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl SplitCurve for ShapeUpgradeSplitCurve {
    fn split_curve_data(&self) -> &SplitCurveData {
        &self.data
    }
}
