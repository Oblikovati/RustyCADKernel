//! `Aspect_GenId`: a generator of unique integer identifiers in a closed interval.
use super::exceptions::ident_definition_error;
use crate::json_dump::field_integer;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use std::collections::VecDeque;
use std::io;

/// Hands out identifiers of `[lower, upper]` in increasing order and reuses freed ones first.
///
/// Example: `let mut ids = AspectGenId::new(1, 3)?; assert_eq!(ids.next_id()?, 1);`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AspectGenId {
    /// `myFreeCount`: identifiers never handed out.
    free_count: i32,
    /// `myLength`: size of the interval.
    length: i32,
    /// `myLowerBound`.
    lower_bound: i32,
    /// `myUpperBound`.
    upper_bound: i32,
    /// `myFreeIds`: freed identifiers, most recently freed first.
    free_ids: VecDeque<i32>,
}
impl Default for AspectGenId {
    /// The interval `[0, INT_MAX / 2]` of `Aspect_GenId()`.
    fn default() -> Self {
        let length = i32::MAX / 2 + 1;
        Self::with_bounds(0, i32::MAX / 2, length)
    }
}
impl AspectGenId {
    /// The interval `[low, upper]`; `Aspect_IdentDefinitionError` when `low > upper`.
    ///
    /// Example: `AspectGenId::new(5, 1)` fails with "GenId Create Error: wrong interval".
    pub fn new(low: i32, upper: i32) -> Result<Self, Failure> {
        // The native constructor computes the length (wrapping, as int arithmetic does on
        // the supported compilers) before it checks the interval.
        let length = upper.wrapping_sub(low).wrapping_add(1);
        if low > upper {
            return Err(ident_definition_error(&format!(
                "GenId Create Error: wrong interval [{low}, {upper}]; expected low <= upper"
            )));
        }
        Ok(Self::with_bounds(low, upper, length))
    }
    fn with_bounds(low: i32, upper: i32, length: i32) -> Self {
        Self {
            free_count: length,
            length,
            lower_bound: low,
            upper_bound: upper,
            free_ids: VecDeque::new(),
        }
    }
    /// Frees every identifier; `ids.free_all()` is `Free()`.
    pub fn free_all(&mut self) {
        self.free_count = self.length;
        self.free_ids.clear();
    }
    /// Frees one identifier; values outside the interval are ignored, and freeing the last
    /// one in use resets the generator. Native quirk kept: an identifier that was never
    /// handed out or is already free is queued again.
    ///
    /// Example: `ids.free(1)`.
    pub fn free(&mut self, id: i32) {
        if id < self.lower_bound || id > self.upper_bound {
            return;
        }
        if self.free_count + self.free_ids.len() as i32 + 1 == self.length {
            self.free_all();
        } else {
            self.free_ids.push_front(id);
        }
    }
    /// True when an identifier is available; `ids.has_free()`.
    pub fn has_free(&self) -> bool {
        self.free_count > 0 || !self.free_ids.is_empty()
    }
    /// Number of available identifiers; `ids.available()`.
    pub fn available(&self) -> i32 {
        self.free_count + self.free_ids.len() as i32
    }
    /// The lower bound; `ids.lower()`.
    pub fn lower(&self) -> i32 {
        self.lower_bound
    }
    /// The upper bound; `ids.upper()`.
    pub fn upper(&self) -> i32 {
        self.upper_bound
    }
    /// The next identifier, or `Aspect_IdentDefinitionError` when none is left.
    ///
    /// Example: `ids.next_id()?`.
    pub fn next_id(&mut self) -> Result<i32, Failure> {
        self.try_next_id().ok_or_else(|| {
            ident_definition_error(&format!(
                "Aspect_GenId::Next(), Error: Available == 0 in [{}, {}]",
                self.lower_bound, self.upper_bound
            ))
        })
    }
    /// `Next(int&)`: the next identifier, or `None` when none is left; `ids.try_next_id()`.
    pub fn try_next_id(&mut self) -> Option<i32> {
        if let Some(id) = self.free_ids.pop_front() {
            return Some(id);
        }
        if self.free_count < 1 {
            return None;
        }
        self.free_count -= 1;
        // OCCT 3d097a0 uses [1, INT_MAX] for viewer layers; the final ID fits even
        // though adding the lower bound before subtracting the free count would overflow.
        Some(self.lower_bound + (self.length - self.free_count - 1))
    }
    /// `DumpJson`: counters and bounds, then every free identifier under the key `FreeId`.
    ///
    /// Example: `ids.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        field_integer(stream, "myFreeCount", self.free_count)?;
        field_integer(stream, "myLength", self.length)?;
        field_integer(stream, "myLowerBound", self.lower_bound)?;
        field_integer(stream, "myUpperBound", self.upper_bound)?;
        for free_id in &self.free_ids {
            field_integer(stream, "aFreeId", *free_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maximal_positive_interval_allocates_and_recycles_viewer_layer_ids() {
        let mut ids = AspectGenId::new(1, i32::MAX).unwrap();
        assert_eq!(ids.available(), i32::MAX);
        assert_eq!(ids.next_id().unwrap(), 1);
        assert_eq!(ids.next_id().unwrap(), 2);
        ids.free(1);
        assert_eq!(ids.next_id().unwrap(), 1);
        assert_eq!(ids.next_id().unwrap(), 3);
    }

    #[test]
    fn identifiers_come_in_order_and_freed_ones_first() {
        let mut ids = AspectGenId::new(1, 3).unwrap();
        assert_eq!((ids.next_id().unwrap(), ids.next_id().unwrap()), (1, 2));
        ids.free(1);
        assert_eq!(ids.available(), 2);
        assert_eq!(ids.next_id().unwrap(), 1);
        assert_eq!(ids.next_id().unwrap(), 3);
        assert!(!ids.has_free());
        let failure = ids.next_id().unwrap_err();
        assert_eq!(failure.exception_type(), "Aspect_IdentDefinitionError");
        assert_eq!(ids.try_next_id(), None);
    }

    #[test]
    fn freeing_the_last_identifier_in_use_resets() {
        let mut ids = AspectGenId::new(10, 12).unwrap();
        assert_eq!(ids.next_id().unwrap(), 10);
        ids.free(10);
        assert_eq!(ids.available(), 3);
        assert_eq!(ids.next_id().unwrap(), 10);
        ids.free(99);
        assert_eq!(ids.available(), 2);
    }

    #[test]
    fn wrong_interval_and_default_bounds() {
        let failure = AspectGenId::new(5, 1).unwrap_err();
        assert_eq!(failure.exception_type(), "Aspect_IdentDefinitionError");
        let ids = AspectGenId::default();
        assert_eq!((ids.lower(), ids.upper()), (0, i32::MAX / 2));
        assert_eq!(ids.available(), i32::MAX / 2 + 1);
    }

    #[test]
    fn dump_lists_counters_and_free_ids() {
        let mut ids = AspectGenId::new(0, 3).unwrap();
        ids.next_id().unwrap();
        ids.next_id().unwrap();
        ids.free(0);
        let mut stream = DumpStream::default();
        ids.dump_json(&mut stream, -1).unwrap();
        assert_eq!(
            stream.text(),
            "\"FreeCount\": 2, \"Length\": 4, \"LowerBound\": 0, \"UpperBound\": 3, \"FreeId\": 0"
        );
    }
}
