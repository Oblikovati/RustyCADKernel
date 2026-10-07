//! `BOPAlgo_MakePeriodic`: makes a shape periodic in the X, Y and Z directions by making its
//! opposite sides identical, and repeats it along those directions.
#[path = "make_periodic/identical.rs"]
mod identical;
#[path = "make_periodic/repeat.rs"]
mod repeat;
use super::Options;
pub(crate) use identical::compound_of;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};

/// The identical shapes of each shape (`myTwins`, `myRepeatedTwins`).
pub(crate) type TwinMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The periodicity parameters of the X, Y and Z directions (`PeriodicityParams`).
///
/// ```
/// use rustycad_tkbo::bop_algo::PeriodicityParams;
/// let params = PeriodicityParams::new();
/// assert!(!params.periodic[0] && params.is_trimmed[2]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PeriodicityParams {
    /// Whether the shape is made periodic in the direction (`myPeriodic`).
    pub periodic: [bool; 3],
    /// The period of the direction (`myPeriod`).
    pub period: [f64; 3],
    /// Whether the input already fits the period, so it is not trimmed (`myIsTrimmed`).
    pub is_trimmed: [bool; 3],
    /// The start of the period, for trimming (`myPeriodFirst`).
    pub period_first: [f64; 3],
}

impl Default for PeriodicityParams {
    fn default() -> Self {
        Self::new()
    }
}

impl PeriodicityParams {
    /// No periodic direction, zero periods, trimmed input (`PeriodicityParams()`);
    /// `PeriodicityParams::new()`.
    pub fn new() -> Self {
        Self {
            periodic: [false; 3],
            period: [0.0; 3],
            is_trimmed: [true; 3],
            period_first: [0.0; 3],
        }
    }

    /// Restores the defaults (`Clear`); `params.clear()`.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

/// The periodic-shape maker: trims the shape to the periods where requested, splits its
/// opposite sides by each other and records the identical opposite sub-shapes (twins).
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoMakePeriodic;
/// let mut maker = BOPAlgoMakePeriodic::new();
/// maker.make_x_periodic(true, 10.0);
/// assert!(maker.is_x_periodic() && maker.x_period() == 10.0);
/// ```
#[derive(Default)]
pub struct BOPAlgoMakePeriodic {
    options: Options,
    input_shape: Shape,
    params: PeriodicityParams,
    shape: Shape,
    repeated_shape: Shape,
    repeat_period: [f64; 3],
    repeated_twins: TwinMap,
    twins: TwinMap,
    split_history: Handle<History>,
    history: Handle<History>,
    /// The empty list `GetTwins` returns for a shape without twins.
    no_twins: List<Shape>,
}

impl BOPAlgoMakePeriodic {
    /// A maker without periodic direction (`BOPAlgo_MakePeriodic()`);
    /// `BOPAlgoMakePeriodic::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The algorithm options (`BOPAlgo_Options` base); `maker.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutable algorithm options; `maker.options_mut().set_run_parallel(false)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Sets the shape to make periodic (`SetShape`); `maker.set_shape(&shape)`.
    pub fn set_shape(&mut self, shape: &Shape) {
        self.input_shape = shape.clone();
    }

    /// Sets all periodicity parameters (`SetPeriodicityParameters`);
    /// `maker.set_periodicity_parameters(&params)`.
    pub fn set_periodicity_parameters(&mut self, params: &PeriodicityParams) {
        self.params = *params;
    }

    /// The periodicity parameters (`PeriodicityParameters`); `maker.periodicity_parameters()`.
    pub fn periodicity_parameters(&self) -> &PeriodicityParams {
        &self.params
    }

    /// The direction index 0 (X), 1 (Y) or 2 (Z) of any integer, `|id % 3|`
    /// (`ToDirectionID`); `BOPAlgoMakePeriodic::to_direction_id(4) == 1`.
    pub fn to_direction_id(direction: i32) -> i32 {
        (direction % 3).abs()
    }

    /// The array slot of a direction.
    fn slot(direction: i32) -> usize {
        Self::to_direction_id(direction) as usize
    }

    /// Makes the direction periodic with `period`, or not (period 0) (`MakePeriodic`);
    /// `maker.make_periodic(2, true, 5.0)`.
    pub fn make_periodic(&mut self, direction: i32, is_periodic: bool, period: f64) {
        let slot = Self::slot(direction);
        self.params.periodic[slot] = is_periodic;
        self.params.period[slot] = if is_periodic { period } else { 0.0 };
    }

    /// Whether the direction is periodic (`IsPeriodic`); `maker.is_periodic(0)`.
    pub fn is_periodic(&self, direction: i32) -> bool {
        self.params.periodic[Self::slot(direction)]
    }

    /// The period of a periodic direction, else 0 (`Period`); `maker.period(0)`.
    pub fn period(&self, direction: i32) -> f64 {
        let slot = Self::slot(direction);
        if self.params.periodic[slot] {
            self.params.period[slot]
        } else {
            0.0
        }
    }

    /// For a periodic direction, whether the input fits the period and, if not, where the
    /// period starts (`SetTrimmed`); `maker.set_trimmed(0, false, -5.0)`.
    pub fn set_trimmed(&mut self, direction: i32, is_trimmed: bool, first: f64) {
        let slot = Self::slot(direction);
        if self.is_periodic(slot as i32) {
            self.params.is_trimmed[slot] = is_trimmed;
            self.params.period_first[slot] = if is_trimmed { 0.0 } else { first };
        }
    }

    /// Whether the input of the direction is not trimmed (`IsInputTrimmed`);
    /// `maker.is_input_trimmed(0)`.
    pub fn is_input_trimmed(&self, direction: i32) -> bool {
        self.params.is_trimmed[Self::slot(direction)]
    }

    /// The start of the period of an untrimmed direction, else 0 (`PeriodFirst`);
    /// `maker.period_first(0)`.
    pub fn period_first(&self, direction: i32) -> f64 {
        let slot = Self::slot(direction);
        if self.params.is_trimmed[slot] {
            0.0
        } else {
            self.params.period_first[slot]
        }
    }

    /// The periodic shape, base of the repetitions (`Shape`); `maker.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// The shape after the repetitions (`RepeatedShape`); `maker.repeated_shape()`.
    pub fn repeated_shape(&self) -> &Shape {
        &self.repeated_shape
    }

    /// The identical opposite shapes of `shape`, after the repetitions when there were any
    /// (`GetTwins`); `maker.get_twins(&face).extent()`.
    pub fn get_twins(&self, shape: &Shape) -> &List<Shape> {
        let twins = if self.repeated_twins.is_empty() {
            &self.twins
        } else {
            &self.repeated_twins
        };
        twins.seek(shape).unwrap_or(&self.no_twins)
    }

    /// The history of the shape modifications (`History`); `maker.history().is_null()`.
    pub fn history(&self) -> &Handle<History> {
        &self.history
    }

    /// Forgets the repetitions: the repeat periods, the repeated shape and twins, and the
    /// history back to the split history (`ClearRepetitions`); `maker.clear_repetitions()?`.
    pub fn clear_repetitions(&mut self) -> Result<(), Failure> {
        self.repeat_period = [0.0; 3];
        self.repeated_shape = Shape::default();
        self.repeated_twins.clear(false);
        let Some(history) = self.history.get() else {
            return Ok(());
        };
        history.clear();
        if self.split_history.get().is_some() {
            history.merge_handle(&self.split_history)?;
        }
        Ok(())
    }

    /// Forgets the parameters, the results, the histories and the repetitions (`Clear`);
    /// `maker.clear()?`.
    pub fn clear(&mut self) -> Result<(), Failure> {
        self.options.clear();
        self.params.clear();
        self.shape = Shape::default();
        if let Some(history) = self.split_history.get() {
            history.clear();
        }
        if let Some(history) = self.history.get() {
            history.clear();
        }
        self.clear_repetitions()
    }
}

/// Generates the named X, Y and Z accessors of the native header.
macro_rules! direction_accessors {
    ($direction:expr, $make:ident, $is:ident, $period:ident, $set_trimmed:ident, $is_trimmed:ident, $first:ident, $repeat:ident) => {
        impl BOPAlgoMakePeriodic {
            #[doc = concat!("`make_periodic(", stringify!($direction), ", ..)`; `maker.", stringify!($make), "(true, 1.0)`.")]
            pub fn $make(&mut self, is_periodic: bool, period: f64) {
                self.make_periodic($direction, is_periodic, period);
            }
            #[doc = concat!("`is_periodic(", stringify!($direction), ")`; `maker.", stringify!($is), "()`.")]
            pub fn $is(&self) -> bool {
                self.is_periodic($direction)
            }
            #[doc = concat!("`period(", stringify!($direction), ")`; `maker.", stringify!($period), "()`.")]
            pub fn $period(&self) -> f64 {
                self.period($direction)
            }
            #[doc = concat!("`set_trimmed(", stringify!($direction), ", ..)`. Native quirk kept: the start is \
                             declared `bool`, so it is 0 or 1; `maker.", stringify!($set_trimmed), "(false, true)`.")]
            pub fn $set_trimmed(&mut self, is_trimmed: bool, first: bool) {
                self.set_trimmed($direction, is_trimmed, f64::from(u8::from(first)));
            }
            #[doc = concat!("`is_input_trimmed(", stringify!($direction), ")`; `maker.", stringify!($is_trimmed), "()`.")]
            pub fn $is_trimmed(&self) -> bool {
                self.is_input_trimmed($direction)
            }
            #[doc = concat!("`period_first(", stringify!($direction), ")`; `maker.", stringify!($first), "()`.")]
            pub fn $first(&self) -> f64 {
                self.period_first($direction)
            }
            #[doc = concat!("`repeat_shape(", stringify!($direction), ", times)`; `maker.", stringify!($repeat), "(2)?`.")]
            pub fn $repeat(&mut self, times: i32) -> Result<&Shape, Failure> {
                self.repeat_shape($direction, times)
            }
        }
    };
}

direction_accessors!(
    0,
    make_x_periodic,
    is_x_periodic,
    x_period,
    set_x_trimmed,
    is_input_x_trimmed,
    x_period_first,
    x_repeat
);
direction_accessors!(
    1,
    make_y_periodic,
    is_y_periodic,
    y_period,
    set_y_trimmed,
    is_input_y_trimmed,
    y_period_first,
    y_repeat
);
direction_accessors!(
    2,
    make_z_periodic,
    is_z_periodic,
    z_period,
    set_z_trimmed,
    is_input_z_trimmed,
    z_period_first,
    z_repeat
);
