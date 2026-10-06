use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct AssignmentTrace(pub Rc<RefCell<String>>);
impl AssignmentTrace {
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }
    pub fn text(&self) -> String {
        self.0.borrow().clone()
    }
}
pub struct AssignmentItem {
    pub value: i32,
    pub fail_assign: bool,
    pub fail_copy: bool,
    pub trace: AssignmentTrace,
}
impl AssignmentItem {
    pub fn new(value: i32, trace: &AssignmentTrace) -> Self {
        Self {
            value,
            fail_assign: false,
            fail_copy: false,
            trace: trace.clone(),
        }
    }
    pub fn assign_self(&mut self) {
        self.trace
            .0
            .borrow_mut()
            .push_str(&format!("A{}>{};", self.value, self.value));
    }
}
impl Clone for AssignmentItem {
    fn clone(&self) -> Self {
        self.trace
            .0
            .borrow_mut()
            .push_str(&format!("C{};", self.value));
        assert!(
            !self.fail_copy,
            "copy value {} failed; expected successful copy",
            self.value
        );
        Self::new(self.value, &self.trace)
    }
    fn clone_from(&mut self, source: &Self) {
        self.trace
            .0
            .borrow_mut()
            .push_str(&format!("A{}>{};", self.value, source.value));
        self.value = source.value;
        assert!(
            !source.fail_assign,
            "assignment value {} failed; expected successful assignment",
            source.value
        );
    }
}
impl Drop for AssignmentItem {
    fn drop(&mut self) {
        self.trace
            .0
            .borrow_mut()
            .push_str(&format!("D{};", self.value));
    }
}
