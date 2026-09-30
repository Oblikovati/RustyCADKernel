//! `Units_Dimensions`: the powers of the seven SI base quantities plus plane and
//! solid angle, immutable once built.
use super::support::{object, stream_real};
use crate::standard::{Failure, Handle, StandardTransient};
use std::{io::Write, sync::OnceLock};

/// Order of the nine exponents: mass, length, time, electric current, thermodynamic
/// temperature, amount of substance, luminous intensity, plane angle, solid angle.
pub type Exponents = [f64; 9];

/// Physical dimensions of a quantity; `Dimensions::new([0.,1.,0.,0.,0.,0.,0.,0.,0.])`.
pub struct Dimensions {
    base: StandardTransient,
    exponents: Exponents,
}
crate::impl_transient!(Dimensions, "Units_Dimensions", StandardTransient, base);

const LABELS: [&str; 9] = [
    "mass                      ",
    "length                    ",
    "time                      ",
    "electric current          ",
    "thermodynamic temperature ",
    "amount of substance       ",
    "luminous intensity        ",
    "plane angle               ",
    "solid angle               ",
];

impl Dimensions {
    /// Stores the nine exponents in native argument order; `Dimensions::new([1.0; 9])`.
    pub fn new(exponents: Exponents) -> Self {
        Self {
            base: StandardTransient::default(),
            exponents,
        }
    }
    /// Handle form of `new`; `Dimensions::handle([0.0; 9])`.
    pub fn handle(exponents: Exponents) -> Handle<Self> {
        Handle::new(Self::new(exponents))
    }
    /// All nine exponents in native order; `dims.exponents()[1]` is the length power.
    pub fn exponents(&self) -> &Exponents {
        &self.exponents
    }
    /// Power of mass; `dims.mass()`.
    pub fn mass(&self) -> f64 {
        self.exponents[0]
    }
    /// Power of length; `dims.length()`.
    pub fn length(&self) -> f64 {
        self.exponents[1]
    }
    /// Power of time; `dims.time()`.
    pub fn time(&self) -> f64 {
        self.exponents[2]
    }
    /// Power of electric current; `dims.electric_current()`.
    pub fn electric_current(&self) -> f64 {
        self.exponents[3]
    }
    /// Power of thermodynamic temperature; `dims.thermodynamic_temperature()`.
    pub fn thermodynamic_temperature(&self) -> f64 {
        self.exponents[4]
    }
    /// Power of amount of substance; `dims.amount_of_substance()`.
    pub fn amount_of_substance(&self) -> f64 {
        self.exponents[5]
    }
    /// Power of luminous intensity; `dims.luminous_intensity()`.
    pub fn luminous_intensity(&self) -> f64 {
        self.exponents[6]
    }
    /// Power of plane angle; `dims.plane_angle()`.
    pub fn plane_angle(&self) -> f64 {
        self.exponents[7]
    }
    /// Power of solid angle; `dims.solid_angle()`.
    pub fn solid_angle(&self) -> f64 {
        self.exponents[8]
    }
    /// Sums the exponents into a new object (`Multiply`); `a.multiply(&b)?`.
    pub fn multiply(&self, other: &Handle<Dimensions>) -> Result<Handle<Dimensions>, Failure> {
        let other = object(other, "Units_Dimensions::Multiply")?;
        Ok(Self::handle(std::array::from_fn(|i| {
            self.exponents[i] + other.exponents[i]
        })))
    }
    /// Subtracts the exponents into a new object (`Divide`); `a.divide(&b)?`.
    pub fn divide(&self, other: &Handle<Dimensions>) -> Result<Handle<Dimensions>, Failure> {
        let other = object(other, "Units_Dimensions::Divide")?;
        Ok(Self::handle(std::array::from_fn(|i| {
            self.exponents[i] - other.exponents[i]
        })))
    }
    /// Scales every exponent into a new object (`Power`); `a.power(2.0)`.
    pub fn power(&self, exponent: f64) -> Handle<Dimensions> {
        Self::handle(std::array::from_fn(|i| self.exponents[i] * exponent))
    }
    /// Exact equality of all nine exponents; `a.is_equal(&b)?`.
    pub fn is_equal(&self, other: &Handle<Dimensions>) -> Result<bool, Failure> {
        let other = object(other, "Units_Dimensions::IsEqual")?;
        Ok(self.exponents == other.exponents)
    }
    /// Negation of `is_equal`; `a.is_not_equal(&b)?`.
    pub fn is_not_equal(&self, other: &Handle<Dimensions>) -> Result<bool, Failure> {
        Ok(!self.is_equal(other)?)
    }
    /// Writes the native debugging listing (indented by `shift` pairs of spaces);
    /// `dims.dump(0, &mut std::io::stdout())?`.
    pub fn dump(&self, shift: i32, out: &mut dyn Write) -> std::io::Result<()> {
        let indent = "  ".repeat(shift.max(0) as usize);
        writeln!(out, "{indent} with the physical dimensions : ")?;
        for (label, value) in LABELS.iter().zip(self.exponents) {
            writeln!(out, "{indent}         {label}: {}", stream_real(value))?;
        }
        Ok(())
    }
}

/// The shared basic dimension `index` (one-hot exponent, or all zero for `None`).
fn basic(cell: &'static OnceLock<Handle<Dimensions>>, index: Option<usize>) -> Handle<Dimensions> {
    cell.get_or_init(|| {
        Dimensions::handle(std::array::from_fn(|i| {
            if Some(i) == index { 1.0 } else { 0.0 }
        }))
    })
    .clone()
}

macro_rules! basic_dimensions {
    ($($(#[$doc:meta])* $name:ident => $index:expr;)*) => {
        impl Dimensions {
            $(
                $(#[$doc])*
                pub fn $name() -> Handle<Dimensions> {
                    static CELL: OnceLock<Handle<Dimensions>> = OnceLock::new();
                    basic(&CELL, $index)
                }
            )*
        }
    };
}
basic_dimensions! {
    /// The shared dimensionless object (`ALess`); `Dimensions::a_less()`.
    a_less => None;
    /// The shared mass dimension (`AMass`); `Dimensions::a_mass()`.
    a_mass => Some(0);
    /// The shared length dimension (`ALength`); `Dimensions::a_length()`.
    a_length => Some(1);
    /// The shared time dimension (`ATime`); `Dimensions::a_time()`.
    a_time => Some(2);
    /// The shared current dimension (`AElectricCurrent`); `Dimensions::a_electric_current()`.
    a_electric_current => Some(3);
    /// The shared temperature dimension; `Dimensions::a_thermodynamic_temperature()`.
    a_thermodynamic_temperature => Some(4);
    /// The shared substance dimension; `Dimensions::a_amount_of_substance()`.
    a_amount_of_substance => Some(5);
    /// The shared luminous dimension; `Dimensions::a_luminous_intensity()`.
    a_luminous_intensity => Some(6);
    /// The shared plane-angle dimension; `Dimensions::a_plane_angle()`.
    a_plane_angle => Some(7);
    /// The shared solid-angle dimension; `Dimensions::a_solid_angle()`.
    a_solid_angle => Some(8);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_is_componentwise_and_shared_basics_are_stable() {
        let length = Dimensions::a_length();
        let time = Dimensions::a_time();
        let speed = length.get().unwrap().divide(&time).unwrap();
        assert_eq!(speed.get().unwrap().exponents()[1..3], [1.0, -1.0]);
        let area = length.get().unwrap().power(2.0);
        assert_eq!(area.get().unwrap().length(), 2.0);
        assert!(std::ptr::eq(
            length.as_ptr(),
            Dimensions::a_length().as_ptr()
        ));
        let product = speed.get().unwrap().multiply(&time).unwrap();
        assert!(product.get().unwrap().is_equal(&length).unwrap());
        assert!(product.get().unwrap().is_not_equal(&time).unwrap());
        assert!(
            Dimensions::a_less()
                .get()
                .unwrap()
                .multiply(&Handle::default())
                .is_err()
        );
    }

    #[test]
    fn dump_lists_every_exponent() {
        let mut text = Vec::new();
        Dimensions::a_mass()
            .get()
            .unwrap()
            .dump(1, &mut text)
            .unwrap();
        let text = String::from_utf8(text).unwrap();
        assert!(text.starts_with("   with the physical dimensions : \n"));
        assert!(text.contains("           mass                      : 1\n"));
        assert_eq!(text.lines().count(), 10);
    }
}
