//! Measurement arithmetic and its C++ operator spellings. A null token, which the
//! native code dereferences unchecked, is a `Standard_NullObject` error.
use super::Measurement;
use crate::standard::{Failure, Handle};
use crate::units::support::object;
use crate::units::token::{Token, TokenHandle};
use std::ops::{Add, Div, Mul, Sub};

impl Measurement {
    /// Adds `other` converted into this unit, or a default measurement when the
    /// dimensions differ; `a.add(&b)?`.
    pub fn add(&self, other: &Measurement) -> Result<Measurement, Failure> {
        self.additive(other, "Units_Measurement::Add", |a, b| a + b)
    }
    /// Subtracts `other` converted into this unit, or a default measurement when the
    /// dimensions differ; `a.subtract(&b)?`.
    pub fn subtract(&self, other: &Measurement) -> Result<Measurement, Failure> {
        self.additive(other, "Units_Measurement::Subtract", |a, b| a - b)
    }
    fn additive(
        &self,
        other: &Measurement,
        member: &str,
        combine: fn(f64, f64) -> f64,
    ) -> Result<Measurement, Failure> {
        let mine = object(&self.token, member)?.dimensions();
        let theirs = object(&other.token, member)?.dimensions();
        if object(&mine, member)?.is_not_equal(&theirs)? {
            return Ok(Measurement::new());
        }
        let value = other.token.multiplied(other.measurement)?;
        let value = self.token.divided(value)?;
        let value = combine(self.measurement, value);
        Ok(Measurement::with_token(value, &self.token.creates()?))
    }
    /// Multiplies values and tokens; `a.multiply(&b)?`.
    pub fn multiply(&self, other: &Measurement) -> Result<Measurement, Failure> {
        let value = self.measurement * other.measurement;
        let token = object(&self.token, "Units_Measurement::Multiply")?.multiply(&other.token)?;
        Ok(Measurement::with_token(value, &token))
    }
    /// Scales the value, copying the token; `a.multiply_real(2.0)?`.
    pub fn multiply_real(&self, value: f64) -> Result<Measurement, Failure> {
        Ok(Measurement::with_token(
            self.measurement * value,
            &self.token.creates()?,
        ))
    }
    /// Divides values and tokens; `a.divide(&b)?`.
    pub fn divide(&self, other: &Measurement) -> Result<Measurement, Failure> {
        let value = self.measurement / other.measurement;
        let token = self.token.divide(&other.token)?;
        Ok(Measurement::with_token(value, &token))
    }
    /// Divides the value, copying the token; `a.divide_real(2.0)?`.
    pub fn divide_real(&self, value: f64) -> Result<Measurement, Failure> {
        Ok(Measurement::with_token(
            self.measurement / value,
            &self.token.creates()?,
        ))
    }
    /// Raises value and token; `a.power(2.0)?`.
    pub fn power(&self, exponent: f64) -> Result<Measurement, Failure> {
        let value = self.measurement.powf(exponent);
        let token: Handle<Token> =
            object(&self.token, "Units_Measurement::Power")?.power(exponent)?;
        Ok(Measurement::with_token(value, &token))
    }
}

impl Add for &Measurement {
    type Output = Result<Measurement, Failure>;
    fn add(self, other: &Measurement) -> Self::Output {
        Measurement::add(self, other)
    }
}
impl Sub for &Measurement {
    type Output = Result<Measurement, Failure>;
    fn sub(self, other: &Measurement) -> Self::Output {
        self.subtract(other)
    }
}
impl Mul for &Measurement {
    type Output = Result<Measurement, Failure>;
    fn mul(self, other: &Measurement) -> Self::Output {
        self.multiply(other)
    }
}
impl Mul<f64> for &Measurement {
    type Output = Result<Measurement, Failure>;
    fn mul(self, value: f64) -> Self::Output {
        self.multiply_real(value)
    }
}
impl Div for &Measurement {
    type Output = Result<Measurement, Failure>;
    fn div(self, other: &Measurement) -> Self::Output {
        self.divide(other)
    }
}
impl Div<f64> for &Measurement {
    type Output = Result<Measurement, Failure>;
    fn div(self, value: f64) -> Self::Output {
        self.divide_real(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Dimensions;

    fn metres(value: f64, word: &[u8], factor: f64) -> Measurement {
        let token = Token::with_dimensions(word, b"U", factor, &Dimensions::a_length());
        Measurement::with_token(value, &Handle::new(token))
    }

    #[test]
    fn arithmetic_converts_into_the_left_unit() {
        let (km, m) = (metres(1.0, b"km", 1000.0), metres(500.0, b"m", 1.0));
        assert_eq!((&km + &m).unwrap().measurement(), 1.5);
        assert_eq!((&km - &m).unwrap().measurement(), 0.5);
        let area = (&km * &m).unwrap();
        assert_eq!(area.token().get().unwrap().word(), "(km)*(m)");
        assert_eq!((&km * 3.0).unwrap().measurement(), 3.0);
        assert_eq!((&m / 2.0).unwrap().measurement(), 250.0);
        assert_eq!((&km / &m).unwrap().measurement(), 0.002);
        assert_eq!(km.power(2.0).unwrap().token().get().unwrap().value(), 1e6);
        let time = Token::with_dimensions(b"s", b"U", 1.0, &Dimensions::a_time());
        let seconds = Measurement::with_token(2.0, &Handle::new(time));
        assert!(!(&km + &seconds).unwrap().has_token());
        assert!((&Measurement::new() + &km).is_err());
    }
}
