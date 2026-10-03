use super::super::split_by_line::helpers::{IOR_BOTH, ITP_BEGSEG, ITP_ENDSEG, ITP_INTER, ITP_TANG};
use super::LineEvent;
use rustycad_tkernel::precision;

pub(super) fn sort_and_merge(events: &mut Vec<LineEvent>) {
    sort_events(events);
    merge_events(events);
}

fn sort_events(events: &mut [LineEvent]) {
    // Native stable bubble ordering preserves equal/NaN parameter event order.
    for end in (1..events.len()).rev() {
        for index in 0..end {
            if events[index].parameter > events[index + 1].parameter {
                events.swap(index, index + 1);
            }
        }
    }
}

fn merge_events(events: &mut Vec<LineEvent>) {
    let mut index = 0;
    while index + 1 < events.len() {
        if tangency_pair(&events[index], &events[index + 1]) {
            let sides = (events[index].code | events[index + 1].code) & IOR_BOTH;
            events[index].code = sides
                | if sides == IOR_BOTH {
                    ITP_INTER
                } else {
                    ITP_TANG
                };
            events.remove(index + 1);
        }
        index += 1;
    }
}

fn tangency_pair(first: &LineEvent, second: &LineEvent) -> bool {
    if (second.parameter - first.parameter).abs() > precision::p_confusion()
        && !first.vertex.is_same(&second.vertex)
    {
        return false;
    }
    (first.code & ITP_ENDSEG != 0 && second.code & ITP_BEGSEG != 0)
        || (first.code & ITP_BEGSEG != 0 && second.code & ITP_ENDSEG != 0)
}

#[derive(Default)]
pub(super) struct LineParity {
    crossings: i32,
    half: i32,
    tangency_level: i32,
}

impl LineParity {
    pub(super) fn advance(&mut self, code: i32) -> bool {
        let interior = self.tangency_level == 0 && self.is_odd();
        if code & ITP_INTER != 0 {
            self.crossings += 1;
        } else if code & ITP_BEGSEG != 0 {
            self.tangency_level += 1;
            self.adjust_half(code);
        } else if code & ITP_ENDSEG != 0 {
            self.tangency_level -= 1;
            self.adjust_half(code);
        }
        interior
    }

    fn adjust_half(&mut self, code: i32) {
        if self.half == 0 {
            self.half = code & IOR_BOTH;
        } else if self.half != code & IOR_BOTH {
            self.crossings += 1;
        }
    }

    pub(super) fn is_odd(&self) -> bool {
        self.crossings % 2 != 0
    }
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
