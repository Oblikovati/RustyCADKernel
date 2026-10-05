//! Reproduces rustdoc JSON omission for public inherent methods on trait objects.

pub trait StorageDriver {
    fn put_integer(&mut self, value: i32);
    fn get_integer(&mut self, value: &mut i32);
}

pub trait DriverValue {
    fn write(self, driver: &mut dyn StorageDriver);
    fn read(&mut self, driver: &mut dyn StorageDriver);
}

impl DriverValue for i32 {
    fn write(self, driver: &mut dyn StorageDriver) {
        driver.put_integer(self);
    }

    fn read(&mut self, driver: &mut dyn StorageDriver) {
        driver.get_integer(self);
    }
}

impl dyn StorageDriver + '_ {
    /// Writes and returns the trait-object receiver; `driver.put(7)`.
    pub fn put<V: DriverValue>(&mut self, value: V) -> &mut Self {
        value.write(self);
        self
    }

    /// Reads and returns the trait-object receiver; `driver.get(&mut value)`.
    pub fn get<V: DriverValue>(&mut self, value: &mut V) -> &mut Self {
        value.read(self);
        self
    }
}

/// Compile-time proof that the dyn inherent methods are callable.
pub fn exercise_dyn_methods(driver: &mut dyn StorageDriver, value: &mut i32) {
    let _ = driver.put(*value).get(value);
}
