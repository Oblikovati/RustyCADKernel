//! `Units::Convert`, `Units::ToSI` and `Units::FromSI`.
use super::{LastUnit, Units};
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;
use crate::units::dimensions::Dimensions;
use crate::units::measurement::Measurement;
use crate::units::support::{c_str, object};
use crate::units::token::ShiftedToken;
use crate::units::unit_sentence::UnitSentence;

impl Units {
    /// Converts `value` from `first` to `second` through a measurement; an invalid
    /// `first` warns and returns `value` unchanged, a dimension mismatch keeps it too;
    /// `units.convert(1.0, b"km", b"m")?`.
    pub fn convert(&self, value: f64, first: &[u8], second: &[u8]) -> Result<f64, Failure> {
        let mut measurement = Measurement::with_unit(self, value, first)?;
        if !measurement.has_token() {
            let text = [
                b"Units::Convert: can not convert - incorrect unit '".as_slice(),
                c_str(first),
                b"' => result is not correct",
            ];
            self.warning(&text.concat());
            return Ok(value);
        }
        measurement.convert(self, second)?;
        Ok(measurement.measurement())
    }
    /// `(value + move) * factor` for `unit`, 0 for an invalid unit; `units.to_si(1.0, b"mm")?`.
    pub fn to_si(&self, value: f64, unit: &[u8]) -> Result<f64, Failure> {
        self.to_si_dimensions(value, unit, &mut Handle::default())
    }
    /// `to_si`, also storing the unit dimensions in `dimensions` (left untouched for an
    /// invalid unit). The last evaluated unit is cached, the cache starting as the
    /// empty unit with factor 0; `units.to_si_dimensions(1.0, b"mm", &mut dims)?`.
    pub fn to_si_dimensions(
        &self,
        value: f64,
        unit: &[u8],
        dimensions: &mut Handle<Dimensions>,
    ) -> Result<f64, Failure> {
        let Some((factor, movement, dims)) = self.last_unit(unit)? else {
            return Ok(0.0);
        };
        *dimensions = dims;
        Ok((value + movement) * factor)
    }
    /// `value / factor - move` for `unit`, 0 for an invalid unit; `units.from_si(1.0, b"mm")?`.
    pub fn from_si(&self, value: f64, unit: &[u8]) -> Result<f64, Failure> {
        self.from_si_dimensions(value, unit, &mut Handle::default())
    }
    /// `from_si`, also storing the unit dimensions (untouched for an invalid unit);
    /// `units.from_si_dimensions(1.0, b"mm", &mut dims)?`.
    pub fn from_si_dimensions(
        &self,
        value: f64,
        unit: &[u8],
        dimensions: &mut Handle<Dimensions>,
    ) -> Result<f64, Failure> {
        let Some((factor, movement, dims)) = self.last_unit(unit)? else {
            return Ok(0.0);
        };
        *dimensions = dims;
        Ok((value / factor) - movement)
    }
    /// The cached factor, move and dimensions of `unit`, re-evaluated when the unit
    /// changes; `None` for a sentence that is not done (cache kept).
    fn last_unit(&self, unit: &[u8]) -> Result<Option<(f64, f64, Handle<Dimensions>)>, Failure> {
        let spelled = AsciiString::from_bytes(unit);
        if self.caches().last.unit.as_bytes() != spelled.as_bytes() {
            let mut sentence = UnitSentence::new(self, unit)?;
            if !sentence.is_done()? {
                return Ok(None);
            }
            let token = sentence.evaluate()?;
            let evaluated = object(&token, "Units::ToSI")?;
            let movement = token
                .cast::<ShiftedToken>()
                .get()
                .map_or(0.0, ShiftedToken::movement);
            self.caches().last = LastUnit {
                unit: spelled,
                value: evaluated.value(),
                movement,
                dimensions: evaluated.dimensions(),
            };
        }
        let caches = self.caches();
        let last = &caches.last;
        Ok(Some((last.value, last.movement, last.dimensions.clone())))
    }
}
