//! Named fakes for the I/O GeomTools touches: the console, the text streams, the
//! progress indicator and an undefined-type handler; plus geometry builders.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{ProgressIndicatorHooks, ProgressScope};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomCurve, GeomLine};
use rustycad_tkg3d::geom_eval::GeomEvalCircularHelixCurve;
use rustycad_tkgeombase::geom_tools::{
    ConsoleOutput, UndefinedTypeHandler, UndefinedTypeHandlerBase,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// A console that records every write instead of printing it.
#[derive(Default)]
pub struct RecordingConsole {
    written: Mutex<Vec<u8>>,
}
impl RecordingConsole {
    pub fn text(&self) -> String {
        String::from_utf8(self.written.lock().unwrap().clone()).unwrap()
    }
}
impl ConsoleOutput for RecordingConsole {
    fn write(&self, bytes: &[u8]) {
        self.written.lock().unwrap().extend_from_slice(bytes);
    }
}

/// An in-memory text stream: an `std::stringstream` over `text`.
pub fn text_stream(text: &str) -> IoStream {
    IoStream::from_bytes(text.as_bytes().to_vec())
}

/// A fresh, empty output stream.
pub fn output_stream() -> IoStream {
    IoStream::from_bytes(Vec::new())
}

/// Everything written to an in-memory stream.
pub fn written_text(stream: IoStream) -> String {
    let mut device = stream.into_device().unwrap();
    device.seek(SeekFrom::Start(0)).unwrap();
    let mut bytes = Vec::new();
    device.read_to_end(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}

/// What a read left unread, after clearing the state.
pub fn unread(stream: &mut IoStream) -> String {
    stream.clear(0);
    let (mut rest, mut byte) = (Vec::new(), 0);
    loop {
        stream.get(&mut byte);
        if stream.fail() {
            return String::from_utf8(rest).unwrap();
        }
        rest.push(byte);
    }
}

/// A progress indicator that breaks after `limit` UserBreak questions and counts them.
pub struct BreakingIndicator {
    pub calls: Arc<AtomicI32>,
    pub limit: i32,
}
impl ProgressIndicatorHooks for BreakingIndicator {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        self.calls.fetch_add(1, Ordering::SeqCst) + 1 > self.limit
    }
}

/// A handler that stores the GeomEval helix under type code 42 and fails on code 13.
pub struct HelixHandler {
    parent: StandardTransient,
    base: UndefinedTypeHandlerBase,
}
impl_transient!(
    HelixHandler,
    "HelixHandler",
    StandardTransient,
    parent,
    interfaces: [dyn UndefinedTypeHandler]
);
impl HelixHandler {
    pub fn new(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            parent: StandardTransient::default(),
            base: UndefinedTypeHandlerBase::with_console(console),
        }
    }
}
impl UndefinedTypeHandler for HelixHandler {
    fn console(&self) -> Arc<dyn ConsoleOutput> {
        self.base.console()
    }
    fn print_curve(
        &self,
        curve: &Handle<dyn GeomCurve>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        let helix: Handle<GeomEvalCircularHelixCurve> = curve.cast();
        if helix.is_null() {
            return self.base.print_curve(curve, stream, compact);
        }
        stream
            .insert_bytes(b"42 ")
            .insert_real(helix.radius())
            .insert_bytes(b" ");
        stream.insert_real(helix.pitch()).insert_bytes(b"\n");
        Ok(())
    }
    fn read_curve(
        &self,
        ctype: i32,
        stream: &mut IoStream,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        match ctype {
            42 => {
                let radius = rustycad_tkgeombase::geom_tools::get_real(stream);
                let pitch = rustycad_tkgeombase::geom_tools::get_real(stream);
                let helix = GeomEvalCircularHelixCurve::new(Axis2::default(), radius, pitch)?;
                Ok(Handle::new(helix).into_cast())
            }
            13 => Err(Failure::with_type(
                "Standard_Failure",
                "code 13 always fails",
                "",
            )),
            _ => self.base.read_curve(ctype, stream),
        }
    }
}

/// Serialises the tests that replace the process-wide undefined-type handler.
pub fn handler_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn line(x: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(
        Point3::new(x, 0.0, 0.0),
        z_direction(),
    ))
    .into_cast()
}

pub fn circle(radius: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomCircle::new(Axis2::default(), radius).unwrap()).into_cast()
}

pub fn helix() -> Handle<dyn GeomCurve> {
    Handle::new(GeomEvalCircularHelixCurve::new(Axis2::default(), 1.5, 0.25).unwrap()).into_cast()
}

/// A clamped cubic B-spline with an interior knot and irrational coordinates.
pub fn bspline() -> Handle<dyn GeomCurve> {
    let coords = [
        (0.0, 0.0, 0.0),
        (1.0 / 3.0, 2.0, 0.1),
        (2.0, -1.0, 0.5),
        (3.0, 3.0, 1.0),
        (4.0, 0.0, 0.0),
    ];
    let poles: Vec<Point3> = coords
        .iter()
        .map(|&(x, y, z)| Point3::new(x, y, z))
        .collect();
    let mut pole_array = Array1::with_bounds(1, 5).unwrap();
    for (index, pole) in poles.into_iter().enumerate() {
        pole_array.set_value(index as i32 + 1, pole).unwrap();
    }
    let knots = array(&[0.0, 0.1f64.sqrt(), 1.0]);
    let mut mults = Array1::with_bounds(1, 3).unwrap();
    for (index, mult) in [4, 1, 4].into_iter().enumerate() {
        mults.set_value(index as i32 + 1, mult).unwrap();
    }
    Handle::new(GeomBSplineCurve::new(&pole_array, &knots, &mults, 3, false).unwrap()).into_cast()
}

pub fn array(values: &[f64]) -> Array1<'static, f64> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (index, value) in values.iter().enumerate() {
        array.set_value(index as i32 + 1, *value).unwrap();
    }
    array
}

/// The Z direction (the port's `Direction3::default()` is X).
pub fn z_direction() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}
