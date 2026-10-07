use rustycad_tkernel::ncollection::DynamicArray;
use std::{cell::RefCell, rc::Rc};
pub(crate) type Events = Rc<RefCell<String>>;
#[derive(Default)]
pub(crate) struct Payload {
    pub(crate) value: i32,
    events: Option<Events>,
}
impl Payload {
    pub(crate) fn new(value: i32, events: &Events) -> Self {
        events.borrow_mut().push_str(&format!("C{value},"));
        assert_ne!(
            value, -1,
            "constructor value -1: expected another probe value"
        );
        Self {
            value,
            events: Some(events.clone()),
        }
    }
    pub(crate) fn copied(source: &Self) -> Self {
        let events = source.events.as_ref().unwrap();
        events.borrow_mut().push_str(&format!("K{},", source.value));
        assert_ne!(
            source.value, -2,
            "copy value -2: expected another probe value"
        );
        Self {
            value: source.value,
            events: Some(events.clone()),
        }
    }
    pub(crate) fn moved(source: &mut Self) -> Self {
        let value = source.value;
        let events = source.events.as_ref().unwrap();
        events.borrow_mut().push_str(&format!("M{value},"));
        source.value = -9;
        assert_ne!(value, -3, "move value -3: expected another probe value");
        Self {
            value,
            events: Some(events.clone()),
        }
    }
    pub(crate) fn assign(target: &mut Self, donor: &mut Self) {
        target
            .events
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push_str(&format!("A{}:{},", target.value, donor.value));
        target.value = donor.value;
        assert_ne!(
            target.value, 99,
            "assignment value 99: expected another probe value"
        );
    }
}
impl Drop for Payload {
    fn drop(&mut self) {
        if let Some(events) = &self.events {
            events.borrow_mut().push_str(&format!("D{},", self.value));
        }
    }
}
pub(crate) fn retained_payloads() -> DynamicArray<Payload> {
    let mut array = DynamicArray::with_increment(2);
    for _ in 0..3 {
        array.append(Payload::default());
    }
    array.clear(false);
    array.set_increment(usize::MAX);
    array
}
pub(crate) fn row(
    name: &str,
    array: &DynamicArray<Payload>,
    events: &Events,
    caught: bool,
) -> String {
    let value = array.first().map_or(0, |slot| slot.value);
    let log = std::mem::take(&mut *events.borrow_mut());
    format!(
        "{name} caught={} size={} value={value} events={log}",
        u8::from(caught),
        array.size()
    )
}
