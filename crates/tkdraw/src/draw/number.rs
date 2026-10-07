//! `Draw_Number` (numeric variables) and `Draw_Chronometer` (timer variables).
use super::DrawInterpretor;
use super::display::DrawDisplay;
use super::drawable::{Drawable3D, Drawable3DBase, DrawableHandle};
use super::istream::DrawIStream;
use super::ostream::DrawOStream;
use rustycad_tkernel::osd::Timer;
use rustycad_tkernel::standard::Failure;
use std::cell::{Cell, RefCell, RefMut};
use std::rc::Rc;

/// `Draw_Number`: the value of a numeric DRAW variable.
///
/// ```
/// use rustycad_tkdraw::draw::DrawNumber;
/// let number = DrawNumber::new(2.0);
/// number.set_value(3.5);
/// assert_eq!(number.value(), 3.5);
/// ```
#[derive(Debug)]
pub struct DrawNumber {
    base: Drawable3DBase,
    value: Cell<f64>,
}

impl DrawNumber {
    /// `Draw_Number(double)`.
    pub fn new(value: f64) -> Self {
        Self {
            base: Drawable3DBase::new(),
            value: Cell::new(value),
        }
    }

    /// `Value()`.
    pub fn value(&self) -> f64 {
        self.value.get()
    }

    /// `Value(double)`.
    pub fn set_value(&self, value: f64) {
        self.value.set(value);
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro): registers
    /// [`DrawNumber::restore`] under `Draw_Number` in `session`.
    pub fn register_factory(session: &crate::draw::DrawSession) {
        session.register_factory("Draw_Number", Self::restore);
    }

    /// `Restore`: reads one number; a missing number restores 0 (`operator>>` stores 0
    /// on failure since C++11, which overwrites the `RealLast()` initial value).
    pub fn restore(stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let value = stream.read_real().unwrap_or(0.0);
        Some(Rc::new(Self::new(value)))
    }
}

impl Drawable3D for DrawNumber {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "Draw_Number"
    }

    /// Does nothing.
    fn draw_on(&self, _display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        Ok(())
    }

    fn is_displayable(&self) -> bool {
        false
    }

    fn copy(&self, _this: &DrawableHandle) -> DrawableHandle {
        Rc::new(Self::new(self.value()))
    }

    fn dump(&self, stream: &mut DrawOStream) {
        stream.put_real(self.value());
    }

    /// Scientific, precision 15, width 30, then a newline. Upstream quirk, kept: the
    /// closing `setf(aFlags)` ORs the old flags back instead of restoring them, so the
    /// stream stays scientific with precision 15.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream
            .set_scientific(true)
            .set_precision(15)
            .set_width(30)
            .put_real(self.value())
            .put_str("\n");
        Ok(())
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("numeric")?;
        Ok(())
    }
}

/// `Draw_Chronometer`: a timer stored in a DRAW variable.
#[derive(Default)]
pub struct DrawChronometer {
    base: Drawable3DBase,
    timer: RefCell<Timer>,
}

impl DrawChronometer {
    /// `Draw_Chronometer()`: a stopped timer.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Timer()`: the timer, borrowed for modification.
    pub fn timer(&self) -> RefMut<'_, Timer> {
        self.timer.borrow_mut()
    }
}

impl Drawable3D for DrawChronometer {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "Draw_Chronometer"
    }

    /// Does nothing.
    fn draw_on(&self, _display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        Ok(())
    }

    /// A new, stopped chronometer (the timer state is not copied, as upstream).
    fn copy(&self, _this: &DrawableHandle) -> DrawableHandle {
        Rc::new(Self::new())
    }

    fn dump(&self, stream: &mut DrawOStream) {
        stream
            .put_str("Chronometer, ")
            .put_str(&self.timer.borrow().show_to());
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("chronometer")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_save_in_scientific_notation_and_restore() {
        let number = DrawNumber::new(-2.5);
        let mut stream = DrawOStream::new();
        number.save(&mut stream).unwrap();
        assert_eq!(stream.text(), "        -2.500000000000000e+00\n");
        let mut input = DrawIStream::new(stream.take_text().into_bytes());
        let restored = DrawNumber::restore(&mut input).unwrap();
        assert_eq!(restored.downcast_ref::<DrawNumber>().unwrap().value(), -2.5);
    }

    #[test]
    fn copies_are_new_objects() {
        let handle: DrawableHandle = Rc::new(DrawNumber::new(1.0));
        let copy = handle.copy(&handle);
        assert!(!Rc::ptr_eq(&handle, &copy));
        let chrono: DrawableHandle = Rc::new(DrawChronometer::new());
        assert_eq!(chrono.copy(&chrono).type_name(), "Draw_Chronometer");
        assert!(!handle.is_displayable());
    }

    #[test]
    fn dump_uses_the_stream_precision() {
        let mut stream = DrawOStream::new();
        stream.set_precision(15);
        DrawNumber::new(1.0 / 3.0).dump(&mut stream);
        assert_eq!(stream.text(), "0.333333333333333");
    }
}
