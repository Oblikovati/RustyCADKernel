//! Adaptive selection tolerance counts at OCCT 3d097a0.
use rustycad_tkernel::{ncollection::DataMap, standard::Failure};

/// Retains zero-frequency keys so excess decrements remain observable.
/// Example: `let mut tolerances = SelectMgrToleranceMap::new(); tolerances.add(4);`.
pub struct SelectMgrToleranceMap {
    frequencies: DataMap<i32, i32>,
    largest: i32,
    custom: i32,
}
impl Default for SelectMgrToleranceMap {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrToleranceMap {
    /// Start with adaptive/default tolerance 2 and custom sentinel -1; `SelectMgrToleranceMap::new()`.
    pub fn new() -> Self {
        Self {
            frequencies: DataMap::default(),
            largest: -1,
            custom: -1,
        }
    }
    /// Activate one occurrence of a tolerance; `tolerances.add(4)`.
    pub fn add(&mut self, tolerance: i32) {
        if let Some(frequency) = self.frequencies.change_seek(&tolerance) {
            *frequency += 1;
            if *frequency == 1 && tolerance != self.largest {
                self.largest = tolerance.max(self.largest);
            }
            return;
        }
        self.frequencies.bind(tolerance, 1);
        self.largest = if self.frequencies.extent() == 1 {
            tolerance
        } else {
            tolerance.max(self.largest)
        };
    }
    /// Deactivate a count, rejecting an already inactive key; `tolerances.decrement(4)?`.
    pub fn decrement(&mut self, tolerance: i32) -> Result<(), Failure> {
        let Some(frequency) = self.frequencies.change_seek(&tolerance) else {
            return Ok(());
        };
        if *frequency == 0 {
            return Err(inactive_tolerance_error(tolerance));
        }
        *frequency -= 1;
        if tolerance == self.largest && *frequency == 0 {
            self.largest = self
                .frequencies
                .items()
                .filter(|(_, count)| **count != 0)
                .fold(-1, |maximum, (key, _)| maximum.max(*key));
        }
        Ok(())
    }
    /// Read adaptive tolerance plus any nonnegative custom offset; `tolerances.tolerance()`.
    pub fn tolerance(&self) -> i32 {
        if self.largest < 0 {
            return 2;
        }
        if self.custom < 0 {
            return self.largest;
        }
        self.largest + self.custom
    }
    /// Store an offset without changing active counts; `tolerances.set_custom_tolerance(3)`.
    pub fn set_custom_tolerance(&mut self, tolerance: i32) {
        self.custom = tolerance;
    }
    /// Disable the custom offset, retaining active counts; `tolerances.reset_defaults()`.
    pub fn reset_defaults(&mut self) {
        self.custom = -1;
    }
    /// Read the custom value including its sentinel; `tolerances.custom_tolerance()`.
    pub fn custom_tolerance(&self) -> i32 {
        self.custom
    }
    /// Whether custom tolerance is strictly positive; `tolerances.is_custom_tol_set()`.
    pub fn is_custom_tol_set(&self) -> bool {
        self.custom > 0
    }
}

fn inactive_tolerance_error(tolerance: i32) -> Failure {
    Failure::with_message(
        "Standard_ProgramError",
        &format!(
            "ToleranceMap::Decrement tolerance={tolerance}, frequency=0; expected positive frequency"
        ),
    )
}
