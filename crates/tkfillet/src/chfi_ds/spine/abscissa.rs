//! The curvilinear abscissa of a guideline: edge bounds, extensions, period and indices.
use super::Spine;
use crate::chfi_ds::State;
use crate::chfi_ds::errors::{null_object, out_of_range, standard_failure};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::elclib;

impl Spine {
    /// The loaded abscissa: Standard_NullObject before [`Spine::load`].
    pub(super) fn loaded_abscissa(&self) -> Result<&[f64], Failure> {
        self.abscissa
            .as_deref()
            .ok_or_else(|| null_object("ChFiDS_Spine: the abscissa is null; expected Load() first"))
    }

    /// `abscissa->Value(index)`: Standard_OutOfRange outside `[1, Length]`.
    pub(super) fn abscissa_value(&self, index: i32) -> Result<f64, Failure> {
        let abscissa = self.loaded_abscissa()?;
        let length = abscissa.len() as i32;
        if index < 1 || index > length {
            return Err(out_of_range("ChFiDS_Spine abscissa", index, length));
        }
        Ok(abscissa[(index - 1) as usize])
    }

    /// `abscissa->Value(abscissa->Upper())`: the length of the whole guideline.
    pub(super) fn total_length(&self) -> Result<f64, Failure> {
        let length = self.loaded_abscissa()?.len() as i32;
        self.abscissa_value(length)
    }

    /// `abscissa->Length()`.
    pub(super) fn abscissa_length(&self) -> Result<i32, Failure> {
        Ok(self.loaded_abscissa()?.len() as i32)
    }

    /// `FirstParameter()`: the start extension, else 0; `spine.first_parameter()`.
    pub fn first_parameter(&self) -> f64 {
        if self.ends.first_prolon {
            return self.ends.first_param;
        }
        0.0
    }

    /// `LastParameter()`: the end extension, else the length of the guideline;
    /// Standard_NullObject before [`Spine::load`]; `spine.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        if self.ends.last_prolon {
            return Ok(self.ends.last_param);
        }
        self.total_length()
    }

    /// `SetFirstParameter(Par)`: extends (or cuts) the start of the guideline;
    /// `spine.set_first_parameter(-1.0)`.
    pub fn set_first_parameter(&mut self, parameter: f64) {
        self.ends.first_prolon = true;
        self.ends.first_param = parameter;
    }

    /// `SetLastParameter(Par)`: extends (or cuts) the end of the guideline;
    /// `spine.set_last_parameter(11.0)`.
    pub fn set_last_parameter(&mut self, parameter: f64) {
        self.ends.last_prolon = true;
        self.ends.last_param = parameter;
    }

    /// `FirstParameter(IndexSpine)`: the total length of the edges before the `index`-th;
    /// `spine.first_parameter_of(2)?`.
    pub fn first_parameter_of(&self, index: i32) -> Result<f64, Failure> {
        if index == 1 {
            return Ok(0.0);
        }
        self.abscissa_value(index - 1)
    }

    /// `LastParameter(IndexSpine)`: the total length up to the `index`-th edge included;
    /// `spine.last_parameter_of(2)?`.
    pub fn last_parameter_of(&self, index: i32) -> Result<f64, Failure> {
        self.abscissa_value(index)
    }

    /// `Length(IndexSpine)`: the length of the `index`-th edge; `spine.length(1)?`.
    pub fn length(&self, index: i32) -> Result<f64, Failure> {
        if index == 1 {
            return self.abscissa_value(index);
        }
        Ok(self.abscissa_value(index)? - self.abscissa_value(index - 1)?)
    }

    /// `IsPeriodic()`: whether the first extremity is [`State::Closed`]; `spine.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.first_state == State::Closed
    }

    /// `Period()`: the length of the guideline; Standard_Failure "Non-periodic Spine" when
    /// not periodic; `spine.period()?`.
    pub fn period(&self) -> Result<f64, Failure> {
        if !self.is_periodic() {
            return Err(standard_failure("Non-periodic Spine"));
        }
        self.total_length()
    }

    /// `SetReference(W)`: the approximation reference at `w`, brought into `[0, length]`
    /// for a periodic guideline; `spine.set_reference(0.5)?`.
    pub fn set_reference(&mut self, w: f64) -> Result<(), Failure> {
        self.ends.has_ref = true;
        let total = self.total_length()?;
        self.ends.val_ref = if self.is_periodic() {
            elclib::in_period(w, 0.0, total)
        } else {
            w
        };
        Ok(())
    }

    /// `SetReference(I)`: the approximation reference at the middle of the `index`-th edge;
    /// `spine.set_reference_on_edge(1)?`.
    pub fn set_reference_on_edge(&mut self, index: i32) -> Result<(), Failure> {
        self.ends.has_ref = true;
        self.ends.val_ref = if index == 1 {
            self.abscissa_value(1)? * 0.5
        } else {
            (self.abscissa_value(index)? + self.abscissa_value(index - 1)?) * 0.5
        };
        Ok(())
    }

    /// `Index(W, Forward)`: the index of the edge holding the abscissa `w`; at an edge
    /// junction (within `max(tolesp, Confusion)`) the next edge when `forward`, else the
    /// previous one, wrapping on a periodic guideline; `spine.index_at(0.5, true)?`.
    pub fn index_at(&self, w: f64, forward: bool) -> Result<i32, Failure> {
        let len = self.abscissa_length()?;
        let last = self.total_length()?;
        let t = std_max(self.tolesp, precision::confusion());
        let mut par = w;
        if self.is_periodic() && par.abs() >= t && (par - last).abs() >= t {
            par = elclib::in_period(par, 0.0, last);
        }
        let (index, f, l) = self.find_interval(par, len)?;
        Ok(self.adjust_at_junction(index, len, (par, f, l), (forward, t)))
    }

    /// The loop of `Index(W)`: the first edge whose end is after `par`, with its bounds.
    fn find_interval(&self, par: f64, len: i32) -> Result<(i32, f64, f64), Failure> {
        let (mut f, mut l) = (0.0, 0.0);
        let mut index = 1;
        while index <= len {
            f = l;
            l = self.abscissa_value(index)?;
            if par < l || index == len {
                break;
            }
            index += 1;
        }
        Ok((index, f, l))
    }

    /// The junction rule of `Index(W, Forward)`.
    fn adjust_at_junction(
        &self,
        index: i32,
        len: i32,
        bounds: (f64, f64, f64),
        rule: (bool, f64),
    ) -> i32 {
        let ((par, f, l), (forward, t)) = (bounds, rule);
        if forward && index < len && (par - l).abs() < t {
            return index + 1;
        }
        if !forward && index > 1 && (par - f).abs() < t {
            return index - 1;
        }
        if forward && self.is_periodic() && index == len && (par - l).abs() < t {
            return 1;
        }
        if !forward && self.is_periodic() && index == 1 && (par - f).abs() < t {
            return len;
        }
        index
    }

    /// `Load()`: computes the abscissa (the cumulated edge lengths), makes the first edge
    /// current, and lowers the tolerance to `5e-5` of the parameter range when that is
    /// smaller; `spine.load()?`.
    pub fn load(&mut self) -> Result<(), Failure> {
        let mut abscissa = Vec::with_capacity(self.edges.len());
        let mut a1 = 0.0;
        for edge in &self.edges {
            self.curve.initialize(edge)?;
            a1 += AbscissaPoint::length(&self.curve as &dyn Adaptor3dCurve)?;
            abscissa.push(a1);
        }
        self.abscissa = Some(abscissa);
        self.index_of_curve = 1;
        self.curve.initialize(self.edges(1)?)?;
        let (umin, umax) = (self.first_parameter(), self.last_parameter()?);
        let new_tolesp = 5.0e-5 * (umax - umin);
        if self.tolesp > new_tolesp {
            self.tolesp = new_tolesp;
        }
        Ok(())
    }
}
