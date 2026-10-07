//! `Aspect_SkydomeBackground`: the parameters of a procedural sky background.
use crate::json_dump::{class_begin, field_dumped, field_integer, field_real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::range_error;
use rustycad_tkmath::gp::Direction3;
use std::io;

/// Sun direction, cloudiness, time, fog and cube-map size of a skydome.
///
/// Example: `AspectSkydomeBackground::default().size() == 512`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectSkydomeBackground {
    sun_direction: Direction3,
    cloudiness: f32,
    time: f32,
    fogginess: f32,
    size: i32,
}
impl Default for AspectSkydomeBackground {
    /// Sun along +Y, cloudiness 0.2, time 0, no fog, 512 pixels.
    fn default() -> Self {
        Self {
            sun_direction: Direction3::new(0.0, 1.0, 0.0).expect("unit Y is a direction"),
            cloudiness: 0.2,
            time: 0.0,
            fogginess: 0.0,
            size: 512,
        }
    }
}

/// `Standard_RangeError_Raise_if(condition, message)` with the offending value appended.
fn range_check(invalid: bool, message: &str, value: f64) -> Result<(), Failure> {
    if invalid {
        return Err(range_error(&format!("{message}; got {value}")));
    }
    Ok(())
}

impl AspectSkydomeBackground {
    /// All parameters; `Standard_RangeError` for negative fog or cloudiness or a size `<= 0`.
    ///
    /// Example: `AspectSkydomeBackground::new(sun, 0.2, 0.0, 0.0, 512)?`.
    pub fn new(
        sun_direction: Direction3,
        cloudiness: f32,
        time: f32,
        fogginess: f32,
        size: i32,
    ) -> Result<Self, Failure> {
        let what = "Aspect_SkydomeBackground::Aspect_SkydomeBackground()";
        range_check(
            fogginess < 0.0,
            &format!("{what} theFoggines must be >= 0"),
            f64::from(fogginess),
        )?;
        range_check(
            cloudiness < 0.0,
            &format!("{what} theCloudiness must be >= 0"),
            f64::from(cloudiness),
        )?;
        range_check(
            size <= 0,
            &format!("{what} theSize must be > 0"),
            f64::from(size),
        )?;
        Ok(Self {
            sun_direction,
            cloudiness,
            time,
            fogginess,
            size,
        })
    }
    /// The sun (moon) light direction; `sky.sun_direction()`.
    pub fn sun_direction(&self) -> Direction3 {
        self.sun_direction
    }
    /// The cloud intensity; `sky.cloudiness()`.
    pub fn cloudiness(&self) -> f32 {
        self.cloudiness
    }
    /// The simulation time; `sky.time_parameter()`.
    pub fn time_parameter(&self) -> f32 {
        self.time
    }
    /// The fog intensity; `sky.fogginess()`.
    pub fn fogginess(&self) -> f32 {
        self.fogginess
    }
    /// The cube-map size in pixels; `sky.size()`.
    pub fn size(&self) -> i32 {
        self.size
    }
    /// Sets the sun direction; `sky.set_sun_direction(dir)`.
    pub fn set_sun_direction(&mut self, sun_direction: Direction3) {
        self.sun_direction = sun_direction;
    }
    /// Sets the cloudiness; `Standard_RangeError` when negative; `sky.set_cloudiness(0.5)?`.
    pub fn set_cloudiness(&mut self, cloudiness: f32) -> Result<(), Failure> {
        let message = "Aspect_SkydomeBackground::SetCloudiness() theCloudiness must be >= 0";
        range_check(cloudiness < 0.0, message, f64::from(cloudiness))?;
        self.cloudiness = cloudiness;
        Ok(())
    }
    /// Sets the time; `sky.set_time_parameter(1.5)`.
    pub fn set_time_parameter(&mut self, time: f32) {
        self.time = time;
    }
    /// Sets the fog; `Standard_RangeError` when negative; `sky.set_fogginess(0.1)?`.
    pub fn set_fogginess(&mut self, fogginess: f32) -> Result<(), Failure> {
        let message = "Aspect_SkydomeBackground::SetFogginess() theFoggines must be >= 0";
        range_check(fogginess < 0.0, message, f64::from(fogginess))?;
        self.fogginess = fogginess;
        Ok(())
    }
    /// Sets the cube-map size; `Standard_RangeError` when `<= 0`; `sky.set_size(256)?`.
    pub fn set_size(&mut self, size: i32) -> Result<(), Failure> {
        let message = "Aspect_SkydomeBackground::SetSize() theSize must be > 0";
        range_check(size <= 0, message, f64::from(size))?;
        self.size = size;
        Ok(())
    }
    /// `DumpJson`. Native quirk kept: the class name written is `Aspect_GradientBackground`.
    ///
    /// Example: `sky.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Aspect_GradientBackground")?;
        let sun = self.sun_direction;
        field_dumped(stream, depth, "&mySunDirection", &|s, _| sun.dump_json(s))?;
        field_real(stream, "myTime", f64::from(self.time))?;
        field_real(stream, "myFogginess", f64::from(self.fogginess))?;
        field_real(stream, "myCloudiness", f64::from(self.cloudiness))?;
        field_integer(stream, "mySize", self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters_follow_the_native_checks() {
        let mut sky = AspectSkydomeBackground::default();
        assert_eq!((sky.cloudiness(), sky.size()), (0.2, 512));
        assert_eq!(
            sky.set_size(0).unwrap_err().exception_type(),
            "Standard_RangeError"
        );
        assert!(sky.set_cloudiness(-0.1).is_err());
        assert!(sky.set_fogginess(-0.1).is_err());
        sky.set_fogginess(0.5).unwrap();
        sky.set_time_parameter(2.0);
        assert_eq!((sky.fogginess(), sky.time_parameter()), (0.5, 2.0));
        let sun = Direction3::new(0.0, 0.0, 1.0).unwrap();
        assert!(AspectSkydomeBackground::new(sun, 0.1, 0.0, -1.0, 8).is_err());
        assert!(AspectSkydomeBackground::new(sun, 0.1, 0.0, 0.0, 0).is_err());
    }

    #[test]
    fn dump_names_the_gradient_class_as_natively() {
        let mut stream = DumpStream::default();
        AspectSkydomeBackground::default()
            .dump_json(&mut stream, -1)
            .unwrap();
        assert!(
            stream
                .text()
                .starts_with("\"className\": \"Aspect_GradientBackground\", \"SunDirection\": {")
        );
        assert!(
            stream
                .text()
                .ends_with("\"Time\": 0, \"Fogginess\": 0, \"Cloudiness\": 0.2, \"Size\": 512")
        );
    }
}
