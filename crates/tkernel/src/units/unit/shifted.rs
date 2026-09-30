//! `Units_ShiftedUnit`: a unit whose SI conversion adds a move (temperatures).
use super::{Quantity, Unit};
use crate::standard::{Failure, Handle};
use crate::units::support::{item, object, stream_real, write_bytes};
use crate::units::token::{ShiftedToken, Token};
use std::io::Write;
use std::sync::{PoisonError, RwLock};

/// A unit with an additive offset; `ShiftedUnit::with_name(b"degree Celsius")`.
pub struct ShiftedUnit {
    unit: Unit,
    movement: RwLock<f64>,
}
crate::impl_transient!(ShiftedUnit, "Units_ShiftedUnit", Unit, unit);

impl ShiftedUnit {
    /// Full constructor; `ShiftedUnit::new(b"degree Celsius", b"deg.C", 1.0, 273.15, &q)`.
    pub fn new(
        name: &[u8],
        symbol: &[u8],
        value: f64,
        movement: f64,
        quantity: &Handle<Quantity>,
    ) -> Self {
        Self {
            unit: Unit::new(name, symbol, value, quantity),
            movement: RwLock::new(movement),
        }
    }
    /// Name and first symbol, zero factor and move; `ShiftedUnit::with_symbol(b"n", b"s")`.
    pub fn with_symbol(name: &[u8], symbol: &[u8]) -> Self {
        Self {
            unit: Unit::with_symbol(name, symbol),
            movement: RwLock::new(0.0),
        }
    }
    /// A name without symbols; `ShiftedUnit::with_name(b"degree Celsius")`.
    pub fn with_name(name: &[u8]) -> Self {
        Self {
            unit: Unit::with_name(name),
            movement: RwLock::new(0.0),
        }
    }
    /// The inherited unit part; `shifted.unit().name()`.
    pub fn unit(&self) -> &Unit {
        &self.unit
    }
    /// Replaces the move (`Move(amove)`); `shifted.set_movement(273.15)`.
    pub fn set_movement(&self, movement: f64) {
        *self
            .movement
            .write()
            .unwrap_or_else(PoisonError::into_inner) = movement;
    }
    /// The move (`Move()`); `shifted.movement()`.
    pub fn movement(&self) -> f64 {
        *self.movement.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// `Token()` override: a shifted token for the first symbol; `shifted.token()?`.
    pub fn token(&self) -> Result<Handle<Token>, Failure> {
        let symbol = item(&self.unit.symbols_sequence(), 1)?;
        let quantity = self.unit.quantity();
        let dimensions = object(&quantity, "Units_ShiftedUnit::Token")?.dimensions();
        let token = ShiftedToken::new(
            symbol.as_bytes(),
            b" ",
            self.unit.value(),
            self.movement(),
            &dimensions,
        );
        Ok(Handle::new(token).cast::<Token>())
    }
    /// `Dump` override, which also prints the move; `shifted.dump(&mut out)?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        self.unit.dump_symbols(out)?;
        let tail = format!(
            "\t\t(= *{} SI + {})\n",
            stream_real(self.unit.value()),
            stream_real(self.movement())
        );
        let name = self.unit.name();
        let line = [b"\t\tName:  ".as_slice(), name.as_bytes(), tail.as_bytes()];
        write_bytes(out, &line.concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::support::new_sequence;
    use crate::units::{Dimensions, UnitHandle};

    #[test]
    fn shifted_unit_builds_shifted_tokens_and_dumps_move() {
        let dims = Dimensions::a_thermodynamic_temperature();
        let quantity = Handle::new(Quantity::new(b"TEMPERATURE", &dims, &new_sequence()));
        let unit = Handle::new(ShiftedUnit::new(
            b"degree Celsius",
            b"deg.C",
            1.0,
            0.0,
            &quantity,
        ));
        unit.get().unwrap().set_movement(273.15);
        let base: Handle<Unit> = unit.cast();
        let token = base.token().unwrap();
        assert!(token.is_instance_name("Units_ShiftedToken"));
        let mut out = Vec::new();
        base.dump(0, 0, &mut out).unwrap();
        assert_eq!(
            out,
            b"\"deg.C\"\t\tName:  degree Celsius\t\t(= *1 SI + 273.15)\n".to_vec()
        );
        assert_eq!(ShiftedUnit::with_symbol(b"a", b"b").movement(), 0.0);
        assert_eq!(ShiftedUnit::with_name(b"a").unit().name(), "a");
    }
}
