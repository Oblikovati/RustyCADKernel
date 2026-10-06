use super::*;

pub(crate) struct InsertSpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub index: usize,
    pub after: bool,
    pub moving: bool,
    pub fail_shift: i32,
    pub fail_insert: bool,
}

pub(crate) struct InsertFixture {
    pub(super) vector: ManuallyDrop<LinearVector<InsertItem>>,
    pub(super) donor: InsertItem,
    pub(super) trace: Rc<InsertTrace>,
    pub(super) before: *const InsertItem,
    pub(super) payload: *const i32,
}

impl InsertFixture {
    pub fn new(spec: &InsertSpec) -> Self {
        let trace = Rc::new(InsertTrace::default());
        let mut vector = LinearVector::with_capacity(spec.capacity);
        for index in 0..spec.count {
            vector.emplace_append(|| InsertItem::new(10 + i32::try_from(index).unwrap(), &trace));
        }
        let donor = InsertItem::new(99, &trace);
        let payload = std::ptr::from_ref(&**donor.resource.as_ref().unwrap());
        let before = vector.data_ptr();
        configure_trace(&trace, spec);
        Self {
            vector: ManuallyDrop::new(vector),
            donor,
            trace,
            before,
            payload,
        }
    }

    pub fn apply(&mut self, spec: &InsertSpec) -> &'static str {
        self.trace.active.set(true);
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.insert(spec)));
        self.trace.active.set(false);
        match failed {
            Ok(Ok(())) => "ok",
            Ok(Err(_)) => "range",
            Err(_) => "throw",
        }
    }

    fn insert(
        &mut self,
        spec: &InsertSpec,
    ) -> Result<(), rustycad_tkernel::ncollection::ArrayError> {
        let relocate = |source: &mut InsertItem| source.copied('G');
        let construct = || InsertItem::inserted(&mut self.donor, spec.moving);
        // SAFETY: receiver automatic Drop is suspended, hooks keep sources valid and Drop does
        // not unwind. Our bitmap tracks exact holes/tail; recovery repairs before normal Drop.
        unsafe {
            if spec.after {
                self.vector.insert_after_with_constructors(
                    spec.index,
                    relocate,
                    InsertItem::shift,
                    construct,
                )
            } else {
                self.vector.insert_before_with_constructors(
                    spec.index,
                    relocate,
                    InsertItem::shift,
                    construct,
                )
            }
        }
    }

    pub fn row(&self, spec: &InsertSpec, outcome: &str) -> String {
        let header = format!(
            "{}:{outcome}:{}:{}:{}:{}",
            spec.name,
            self.vector.len(),
            self.vector.capacity(),
            u8::from(self.before == self.vector.data_ptr()),
            self.trace.events.borrow()
        );
        let slots: String = (0..=spec.count)
            .map(|index| self.slot_state(index))
            .collect();
        let payload_same =
            outcome == "ok" && self.payload_matches(spec.index + usize::from(spec.after));
        format!(
            "{header}:{slots}:{}:{}",
            self.donor.state(),
            u8::from(payload_same)
        )
    }

    fn payload_matches(&self, index: usize) -> bool {
        // SAFETY: called only after successful insertion has published every prefix slot.
        let item = unsafe { &*self.vector.data_ptr().add(index) };
        item.resource
            .as_deref()
            .is_some_and(|value| std::ptr::eq(self.payload, value))
    }

    fn slot_state(&self, index: usize) -> String {
        if !self.trace.live.borrow()[index] {
            return "X,".into();
        }
        // SAFETY: the callback/destructor bitmap marks only the successfully constructed slots.
        let item = unsafe { &*self.vector.data_ptr().add(index) };
        format!("{},", item.state())
    }
}

fn configure_trace(trace: &InsertTrace, spec: &InsertSpec) {
    *trace.live.borrow_mut() = vec![true; spec.count];
    trace.live.borrow_mut().push(false);
    trace.count.set(spec.count);
    trace.insertion.set(spec.index + usize::from(spec.after));
    trace.fail_shift.set(spec.fail_shift);
    trace.fail_insert.set(spec.fail_insert);
}
