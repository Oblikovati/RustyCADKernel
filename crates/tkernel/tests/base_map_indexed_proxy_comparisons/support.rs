use rustycad_tkernel::ncollection::BaseMapIndexedIterator;
use std::{cell::RefCell, rc::Rc};
pub type Trace = Rc<RefCell<String>>;
pub type Cursor<const M: bool> = BaseMapIndexedIterator<ComparisonOwner, M>;

pub struct ComparisonValue(pub i32);

pub struct ComparisonOwner {
    tag: char,
    pub extent: usize,
    calls: Trace,
}
impl ComparisonOwner {
    /// Creates an observable owner; `ComparisonOwner::new('a', &calls)`.
    pub fn new(tag: char, calls: &Trace) -> Self {
        Self {
            tag,
            extent: 2,
            calls: Rc::clone(calls),
        }
    }
    /// Supplies an owned non-Clone result; `owner.owned_value().0`.
    pub fn owned_value(&self) -> ComparisonValue {
        self.calls.borrow_mut().push('v');
        ComparisonValue(42)
    }
    /// Supplies an owned key; `owner.key()`.
    pub fn key(&self) -> i32 {
        self.calls.borrow_mut().push('k');
        7
    }
    /// Selects the native const/nonconst Size trace; `owner.size::<true>()`.
    pub fn size<const M: bool>(&self) -> usize {
        let tag = if M {
            self.tag.to_ascii_uppercase()
        } else {
            self.tag
        };
        self.calls.borrow_mut().push(tag);
        self.extent
    }
}
/// Takes the ordered native callback trace; `trace(&calls)`.
pub fn trace(calls: &Trace) -> String {
    let previous = std::mem::take(&mut *calls.borrow_mut());
    if previous.is_empty() {
        "-".to_owned()
    } else {
        previous
    }
}
/// Creates attached or default state; `cursor::<false>(&owner, Some(1))`.
pub fn cursor<const M: bool>(owner: &ComparisonOwner, index: Option<usize>) -> Cursor<M> {
    index.map_or_else(Cursor::default, |index| Cursor::new(owner, index))
}
/// Computes both operators from explicit native Size policies; `row(name, left, right, calls)`.
pub fn row<const L: bool, const R: bool>(
    name: &str,
    left: (&Cursor<L>, &ComparisonOwner),
    right: (&Cursor<R>, &ComparisonOwner),
    calls: &Trace,
) -> String {
    let ((left, owner), (right, other)) = (left, right);
    let (ls, rs) = (ComparisonOwner::size::<L>, ComparisonOwner::size::<R>);
    let equal = u8::from(left.is_equal_with(owner, right, other, ls, rs).unwrap());
    let equal_calls = trace(calls);
    let unequal = u8::from(left.is_not_equal_with(owner, right, other, ls, rs).unwrap());
    let unequal_calls = trace(calls);
    format!("{name} {equal} {equal_calls} {unequal} {unequal_calls}\n")
}
/// Builds a comparison fixture with shared ordered trace; `Fixture::new()`.
pub struct Fixture {
    pub calls: Trace,
    pub first: ComparisonOwner,
    pub second: ComparisonOwner,
}
impl Fixture {
    /// Keeps both owners together before iterator construction; `Fixture::new()`.
    pub fn new() -> Self {
        let calls = Rc::new(RefCell::new(String::new()));
        let first = ComparisonOwner::new('a', &calls);
        let second = ComparisonOwner::new('b', &calls);
        Self {
            calls,
            first,
            second,
        }
    }
    /// Runs both operators for a fixture row; `fixture.pair_row::<true,false>(name, indices, false)`.
    pub fn pair_row<const L: bool, const R: bool>(
        &self,
        name: &str,
        indices: (Option<usize>, Option<usize>),
        same: bool,
    ) -> String {
        let owner = if same { &self.first } else { &self.second };
        let left = cursor::<L>(&self.first, indices.0);
        let right = cursor::<R>(owner, indices.1);
        row(name, (&left, &self.first), (&right, owner), &self.calls)
    }
}
