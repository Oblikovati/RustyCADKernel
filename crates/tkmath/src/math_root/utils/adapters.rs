//! Functor adapters of MathRoot_MultipleUtils.hxx.
//!
//! The C++ adapters hold plain references to the same function and sample vector at
//! once (`FindAllRoots` builds four of them over one `Function&`). Rust expresses that
//! aliasing with a shared [`MultipleFunctionCell`] and `RefCell<Vector>`; each call
//! borrows only for its own duration, exactly where the C++ call dereferences. The
//! C++ `mutable Status` members become [`Cell`] fields read through `status()`.
use crate::math::{Function, FunctionWithDerivative, Vector};
use crate::math_root::MultipleResult;
use crate::math_utils::Status;
use std::cell::{Cell, RefCell};

/// Shared handle to the caller's function, the Rust form of the adapters' `Function&`;
/// `RefCell::new(&mut function)`.
pub type MultipleFunctionCell<'f, F> = RefCell<&'f mut F>;

/// Sampling step of [`super::find_all_roots_impl`] (`SampleFn`); `sampler.sample_at(0, 0.0)`.
pub trait MultipleSampleCallback {
    /// Samples index `index` at `x`; false stops the search with [`Self::status`].
    fn sample_at(&self, index: usize, x: f64) -> bool;
    /// Failure status recorded by the last failed [`Self::sample_at`].
    fn status(&self) -> Status;
}

/// Sampled shifted value lookup (`GetValueFn`); `lookup.value_at(0)`.
pub trait MultipleSampleLookup {
    /// Returns `f(x_index) - offset` stored by the sampler.
    fn value_at(&self, index: usize) -> f64;
}

/// Original function value at an accepted root (`GetRootValueFn`); `callback.root_value(x, &mut v)`.
pub trait MultipleRootValueCallback {
    /// Writes the non-shifted value; false stops the search with [`Self::status`].
    fn root_value(&self, x: f64, value: &mut f64) -> bool;
    /// Failure status recorded by the last failed [`Self::root_value`].
    fn status(&self) -> Status;
}

/// Extra processing of an interval without a sign change (`IntervalExtraFn`).
pub trait MultipleIntervalHandler {
    /// Inspects interval `index` = `[x0, x1]` with shifted values `f0`, `f1`.
    #[allow(clippy::too_many_arguments)]
    fn handle_interval(
        &self,
        index: usize,
        x0: f64,
        x1: f64,
        f0: f64,
        f1: f64,
        result: &mut MultipleResult,
        eps_x: f64,
    );
}

/// Stores `f(x) - offset` into a sample vector (`MultipleSampleValueFn`);
/// `MultipleSampleValueFn::new(&cell, &samples, 0.0)`.
pub struct MultipleSampleValueFn<'a, 'f, 'v, F> {
    function: &'a MultipleFunctionCell<'f, F>,
    samples: &'a RefCell<Vector<'v>>,
    offset: f64,
    status: Cell<Status>,
}
impl<'a, 'f, 'v, F: Function> MultipleSampleValueFn<'a, 'f, 'v, F> {
    /// Binds the shared function, sample storage and offset; status starts `Ok`.
    pub fn new(
        function: &'a MultipleFunctionCell<'f, F>,
        samples: &'a RefCell<Vector<'v>>,
        offset: f64,
    ) -> Self {
        Self {
            function,
            samples,
            offset,
            status: Cell::new(Status::Ok),
        }
    }
    fn fail(&self, status: Status) -> bool {
        self.status.set(status);
        false
    }
}
impl<F: Function> MultipleSampleCallback for MultipleSampleValueFn<'_, '_, '_, F> {
    fn sample_at(&self, index: usize, x: f64) -> bool {
        let mut value: f64 = 0.0;
        if !self.function.borrow_mut().value(x, &mut value) {
            return self.fail(Status::CallbackError);
        }
        if !value.is_finite() {
            return self.fail(Status::NumericalError);
        }
        let mut samples = self.samples.borrow_mut();
        let slot: &mut f64 = samples
            .change_at(index)
            .expect("sample index inside the vector");
        *slot = value - self.offset;
        if !slot.is_finite() {
            return self.fail(Status::NumericalError);
        }
        true
    }
    fn status(&self) -> Status {
        self.status.get()
    }
}

/// Reads sampled values back (`MultipleGetValueFn`); `MultipleGetValueFn::new(&samples)`.
pub struct MultipleGetValueFn<'a, 'v> {
    samples: &'a RefCell<Vector<'v>>,
}
impl<'a, 'v> MultipleGetValueFn<'a, 'v> {
    /// Binds the sample storage filled by [`MultipleSampleValueFn`].
    pub fn new(samples: &'a RefCell<Vector<'v>>) -> Self {
        Self { samples }
    }
}
impl MultipleSampleLookup for MultipleGetValueFn<'_, '_> {
    fn value_at(&self, index: usize) -> f64 {
        *self
            .samples
            .borrow()
            .at(index)
            .expect("sample index inside the vector")
    }
}

/// Brent target `f(x) - offset` (`MultipleBrentValueWrapper`);
/// `MultipleBrentValueWrapper::new(&cell, 1.0)`.
pub struct MultipleBrentValueWrapper<'a, 'f, F> {
    function: &'a MultipleFunctionCell<'f, F>,
    offset: f64,
}
impl<'a, 'f, F: Function> MultipleBrentValueWrapper<'a, 'f, F> {
    /// Binds the shared function and the level to subtract.
    pub fn new(function: &'a MultipleFunctionCell<'f, F>, offset: f64) -> Self {
        Self { function, offset }
    }
}
impl<F: Function> Function for MultipleBrentValueWrapper<'_, '_, F> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        if !self.function.borrow_mut().value(x, result) {
            return false;
        }
        *result -= self.offset;
        true
    }
}

/// Original value at a root with status tracking (`MultipleGetRootValueFn`);
/// `MultipleGetRootValueFn::new(&cell)`.
pub struct MultipleGetRootValueFn<'a, 'f, F> {
    function: &'a MultipleFunctionCell<'f, F>,
    status: Cell<Status>,
}
impl<'a, 'f, F: Function> MultipleGetRootValueFn<'a, 'f, F> {
    /// Binds the shared function; status starts `Ok`.
    pub fn new(function: &'a MultipleFunctionCell<'f, F>) -> Self {
        Self {
            function,
            status: Cell::new(Status::Ok),
        }
    }
}
impl<F: Function> MultipleRootValueCallback for MultipleGetRootValueFn<'_, '_, F> {
    fn root_value(&self, x: f64, value: &mut f64) -> bool {
        if !self.function.borrow_mut().value(x, value) {
            self.status.set(Status::CallbackError);
            return false;
        }
        if !value.is_finite() {
            self.status.set(Status::NumericalError);
            return false;
        }
        true
    }
    fn status(&self) -> Status {
        self.status.get()
    }
}

/// Exposes `f'(x)` as a Brent target (`MultipleDerivativeValueWrapper`); a non-finite
/// `f(x)` replaces the derivative so Brent reports it; `MultipleDerivativeValueWrapper::new(&cell)`.
pub struct MultipleDerivativeValueWrapper<'a, 'f, F> {
    function: &'a MultipleFunctionCell<'f, F>,
}
impl<'a, 'f, F: FunctionWithDerivative> MultipleDerivativeValueWrapper<'a, 'f, F> {
    /// Binds the shared function whose derivative is searched for zeros.
    pub fn new(function: &'a MultipleFunctionCell<'f, F>) -> Self {
        Self { function }
    }
}
impl<F: FunctionWithDerivative> Function for MultipleDerivativeValueWrapper<'_, '_, F> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        if !self.function.borrow_mut().values(x, &mut value, result) {
            return false;
        }
        if !value.is_finite() {
            *result = value;
        }
        true
    }
}

/// Interval handler that does nothing (`MultipleNoExtraHandler`); value-only sampling
/// cannot see tangential roots; `MultipleNoExtraHandler`.
#[derive(Clone, Copy, Debug, Default)]
pub struct MultipleNoExtraHandler;
impl MultipleIntervalHandler for MultipleNoExtraHandler {
    fn handle_interval(
        &self,
        _: usize,
        _: f64,
        _: f64,
        _: f64,
        _: f64,
        _: &mut MultipleResult,
        _: f64,
    ) {
    }
}
