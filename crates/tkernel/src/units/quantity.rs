//! `Units_Quantity`: a physical quantity, its dimensions and its units.
use super::dimensions::Dimensions;
use super::support::{c_str, item, length, write_bytes};
use super::unit::{Unit, UnitHandle};
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::io::Write;

/// A named quantity; `Quantity::new(b"LENGTH", &Dimensions::a_length(), &units)`.
pub struct Quantity {
    base: StandardTransient,
    name: AsciiString,
    dimensions: Handle<Dimensions>,
    units: Handle<HSequence<Handle<Unit>>>,
}
crate::impl_transient!(Quantity, "Units_Quantity", StandardTransient, base);

impl Quantity {
    /// Shares the given dimensions and unit sequence; `Quantity::new(b"MASS", &d, &units)`.
    pub fn new(
        name: &[u8],
        dimensions: &Handle<Dimensions>,
        units: &Handle<HSequence<Handle<Unit>>>,
    ) -> Self {
        Self {
            base: StandardTransient::default(),
            name: AsciiString::from_bytes(c_str(name)),
            dimensions: dimensions.clone(),
            units: units.clone(),
        }
    }
    /// The quantity name; `quantity.name()`.
    pub fn name(&self) -> AsciiString {
        self.name.clone()
    }
    /// The shared dimensions; `quantity.dimensions()`.
    pub fn dimensions(&self) -> Handle<Dimensions> {
        self.dimensions.clone()
    }
    /// The shared unit sequence; `quantity.sequence()`.
    pub fn sequence(&self) -> Handle<HSequence<Handle<Unit>>> {
        self.units.clone()
    }
    /// Name equals the C string; `quantity.is_equal(b"LENGTH")`.
    pub fn is_equal(&self, text: &[u8]) -> bool {
        self.name.as_bytes() == c_str(text)
    }
    /// Writes the name, then each unit's dump when `level > 0`;
    /// `quantity.dump(0, 1, &mut std::io::stdout())?`.
    pub fn dump(&self, shift: i32, level: i32, out: &mut dyn Write) -> Result<(), Failure> {
        let indent = "  ".repeat(shift.max(0) as usize);
        let heading = [b"\n", indent.as_bytes(), self.name.as_bytes(), b"\n"].concat();
        write_bytes(out, &heading)?;
        if level > 0 {
            for index in 1..=length(&self.units)? {
                item(&self.units, index)?.dump(shift + 1, 0, out)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::support::new_sequence;

    #[test]
    fn quantity_dump_lists_units_at_level_one() {
        let units = new_sequence::<Handle<Unit>>();
        let quantity = Handle::new(Quantity::new(b"LENGTH", &Dimensions::a_length(), &units));
        units
            .get()
            .unwrap()
            .append(Handle::new(Unit::new(b"metre", b"m", 1.0, &quantity)));
        let object = quantity.get().unwrap();
        assert!(object.is_equal(b"LENGTH") && !object.is_equal(b"LEN"));
        let mut out = Vec::new();
        object.dump(1, 0, &mut out).unwrap();
        assert_eq!(out, b"\n  LENGTH\n");
        out.clear();
        object.dump(0, 1, &mut out).unwrap();
        assert_eq!(
            out,
            b"\nLENGTH\n\"m\"\t\tName:  metre\t\t(= 1 SI)\n".to_vec()
        );
    }
}
