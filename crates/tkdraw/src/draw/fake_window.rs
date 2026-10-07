//! A recording window system for the tests of the viewer and of the drawables.
use super::display::DrawXSegment;
use super::window::{DrawWindow, DrawWindowEvent, DrawWindowSystem};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// The calls every fake window made, shared with the test.
pub(crate) type CallLog = Rc<RefCell<Vec<String>>>;

/// A window that records its drawing calls.
pub(crate) struct FakeWindow {
    id: u64,
    size: (i32, i32),
    mapped: bool,
    log: CallLog,
}

impl DrawWindow for FakeWindow {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }
    fn set_position(&mut self, _x: i32, _y: i32) {}
    fn width(&self) -> i32 {
        self.size.0
    }
    fn height(&self) -> i32 {
        self.size.1
    }
    fn set_dimension(&mut self, width: i32, height: i32) {
        self.size = (width, height);
    }
    fn title(&self) -> String {
        String::new()
    }
    fn set_title(&mut self, title: &str) {
        self.log
            .borrow_mut()
            .push(format!("{} title {title}", self.id));
    }
    fn is_mapped(&self) -> bool {
        self.mapped
    }
    fn display(&mut self) {
        self.mapped = true;
    }
    fn hide(&mut self) {
        self.mapped = false;
    }
    fn clear(&mut self) {
        self.log.borrow_mut().push(format!("{} clear", self.id));
    }
    fn set_color(&mut self, color: i32) {
        self.log
            .borrow_mut()
            .push(format!("{} color {color}", self.id));
    }
    fn set_mode(&mut self, mode: i32) {
        self.log
            .borrow_mut()
            .push(format!("{} mode {mode}", self.id));
    }
    fn draw_string(&mut self, x: i32, y: i32, text: &str) {
        self.log
            .borrow_mut()
            .push(format!("{} string {x} {y} {text}", self.id));
    }
    fn draw_segments(&mut self, segments: &[DrawXSegment]) {
        for segment in segments {
            self.log
                .borrow_mut()
                .push(format!("{} segment {:?}", self.id, segment.points));
        }
    }
    fn redraw(&mut self) {}
    fn save(&self, _file_name: &str) -> bool {
        true
    }
    fn wait(&mut self, to_wait: bool) {
        self.log
            .borrow_mut()
            .push(format!("{} wait {to_wait}", self.id));
    }
    fn is_window(&self, window: u64) -> bool {
        window == self.id
    }
}

/// A window system that creates [`FakeWindow`]s and replays scripted pointer events.
pub(crate) struct FakeWindowSystem {
    next_id: RefCell<u64>,
    /// The pointer events `next_event` replays, in order.
    pub(crate) events: RefCell<VecDeque<DrawWindowEvent>>,
    /// Every call of every window, as text.
    pub(crate) log: CallLog,
}

impl FakeWindowSystem {
    /// A window system without windows or events.
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            next_id: RefCell::new(100),
            events: RefCell::new(VecDeque::new()),
            log: CallLog::default(),
        })
    }
}

impl DrawWindowSystem for FakeWindowSystem {
    fn create_window(
        &self,
        _title: &str,
        _position: (i32, i32),
        size: (i32, i32),
        _existing: u64,
    ) -> Box<dyn DrawWindow> {
        let mut next = self.next_id.borrow_mut();
        *next += 1;
        Box::new(FakeWindow {
            id: *next,
            size,
            mapped: true,
            log: self.log.clone(),
        })
    }
    fn define_color(&self, _index: usize, _name: &str) -> bool {
        true
    }
    fn flush(&self) {}
    fn next_event(&self) -> DrawWindowEvent {
        self.events
            .borrow_mut()
            .pop_front()
            .expect("the test scripted enough events")
    }
}
