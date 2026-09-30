//! The `Put*`/`Get*` scalar members and their `operator<<`/`operator>>` shorthands.
use super::BaseDriver;
use crate::standard::Failure;

/// Scalar output and input of a driver. A C `char` is a `u8` byte; drivers that convert
/// it to wider integers sign-extend it as the native signed `char` does.
pub trait DriverPrimitives {
    /// `PutReference`; `driver.put_reference(3)?`.
    fn put_reference(&mut self, value: i32) -> Result<(), Failure>;
    /// `PutCharacter`; `driver.put_character(b'A')?`.
    fn put_character(&mut self, value: u8) -> Result<(), Failure>;
    /// `PutExtCharacter`; `driver.put_ext_character(0x20AC)?`.
    fn put_ext_character(&mut self, value: u16) -> Result<(), Failure>;
    /// `PutInteger`; `driver.put_integer(-1)?`.
    fn put_integer(&mut self, value: i32) -> Result<(), Failure>;
    /// `PutBoolean`; `driver.put_boolean(true)?`.
    fn put_boolean(&mut self, value: bool) -> Result<(), Failure>;
    /// `PutReal`; `driver.put_real(0.5)?`.
    fn put_real(&mut self, value: f64) -> Result<(), Failure>;
    /// `PutShortReal`; `driver.put_short_real(0.5)?`.
    fn put_short_real(&mut self, value: f32) -> Result<(), Failure>;

    /// `GetReference`; like the native reference, `value` may change before a failure.
    fn get_reference(&mut self, value: &mut i32) -> Result<(), Failure>;
    /// `GetCharacter`; `driver.get_character(&mut byte)?`.
    fn get_character(&mut self, value: &mut u8) -> Result<(), Failure>;
    /// `GetExtCharacter`; `driver.get_ext_character(&mut unit)?`.
    fn get_ext_character(&mut self, value: &mut u16) -> Result<(), Failure>;
    /// `GetInteger`; `driver.get_integer(&mut value)?`.
    fn get_integer(&mut self, value: &mut i32) -> Result<(), Failure>;
    /// `GetBoolean`; `driver.get_boolean(&mut flag)?`.
    fn get_boolean(&mut self, value: &mut bool) -> Result<(), Failure>;
    /// `GetReal`; `driver.get_real(&mut value)?`.
    fn get_real(&mut self, value: &mut f64) -> Result<(), Failure>;
    /// `GetShortReal`; `driver.get_short_real(&mut value)?`.
    fn get_short_real(&mut self, value: &mut f32) -> Result<(), Failure>;
}

/// A scalar with `operator<<`/`operator>>` overloads on `Storage_BaseDriver`: `u8` (`char`),
/// `u16` (`char16_t`), `i32`, `bool`, `f64` and `f32`.
pub trait DriverValue: Sized {
    /// The matching `Put*` member.
    fn put_into(self, driver: &mut dyn BaseDriver) -> Result<(), Failure>;
    /// The matching `Get*` member.
    fn get_from(&mut self, driver: &mut dyn BaseDriver) -> Result<(), Failure>;
}

macro_rules! driver_value {
    ($type:ty, $put:ident, $get:ident) => {
        impl DriverValue for $type {
            fn put_into(self, driver: &mut dyn BaseDriver) -> Result<(), Failure> {
                driver.$put(self)
            }
            fn get_from(&mut self, driver: &mut dyn BaseDriver) -> Result<(), Failure> {
                driver.$get(self)
            }
        }
    };
}

driver_value!(u8, put_character, get_character);
driver_value!(u16, put_ext_character, get_ext_character);
driver_value!(i32, put_integer, get_integer);
driver_value!(bool, put_boolean, get_boolean);
driver_value!(f64, put_real, get_real);
driver_value!(f32, put_short_real, get_short_real);

impl dyn BaseDriver + '_ {
    /// `operator<<`: writes `value` with its `Put*` member and allows chaining.
    /// Example: `driver.put(1)?.put(2.5)?`.
    pub fn put<V: DriverValue>(&mut self, value: V) -> Result<&mut Self, Failure> {
        value.put_into(self)?;
        Ok(self)
    }

    /// `operator>>`: reads `value` with its `Get*` member and allows chaining.
    /// Example: `driver.get(&mut count)?.get(&mut flag)?`.
    pub fn get<V: DriverValue>(&mut self, value: &mut V) -> Result<&mut Self, Failure> {
        value.get_from(self)?;
        Ok(self)
    }
}
