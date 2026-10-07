// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Scope.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Scope`: the labels a naming may use (the whole framework, or a valid map).
use super::{TNamingNamedShape, TNamingTool};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::{TdfChildIterator, TdfLabel, TdfLabelMap};

/// `TNaming_Scope`.
///
/// ```
/// use rustycad_tkcaf::tnaming::TNamingScope;
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let mut scope = TNamingScope::new();
/// assert!(scope.is_valid(&data.root()));
/// scope.set_with_valid(true);
/// assert!(!scope.is_valid(&data.root()));
/// ```
#[derive(Default)]
pub struct TNamingScope {
    with_valid: bool,
    valid: TdfLabelMap,
}

impl TNamingScope {
    /// `TNaming_Scope()`: the whole framework.
    pub fn new() -> Self {
        Self::default()
    }
    /// `TNaming_Scope(WithValid)`.
    pub fn with_mode(with_valid: bool) -> Self {
        Self {
            with_valid,
            valid: TdfLabelMap::default(),
        }
    }
    /// `TNaming_Scope(valid)`: the scope of the map (a copy).
    pub fn with_map(valid: &TdfLabelMap) -> Self {
        let mut scope = Self::with_mode(true);
        scope.valid.assign(valid);
        scope
    }
    /// `WithValid()`.
    pub fn with_valid(&self) -> bool {
        self.with_valid
    }
    /// `WithValid(mode)`.
    pub fn set_with_valid(&mut self, mode: bool) {
        self.with_valid = mode;
    }
    /// `ClearValid`.
    pub fn clear_valid(&mut self) {
        self.valid.clear(false);
    }
    /// `Valid(L)`.
    pub fn valid(&mut self, label: &TdfLabel) {
        self.valid.add(label.clone());
    }
    /// `ValidChildren(L, withroot)`: every descendant (and the label itself).
    pub fn valid_children(&mut self, label: &TdfLabel, with_root: bool) -> Result<(), Failure> {
        for child in descendants(label)? {
            self.valid.add(child);
        }
        if with_root {
            self.valid.add(label.clone());
        }
        Ok(())
    }
    /// `Unvalid(L)`.
    pub fn unvalid(&mut self, label: &TdfLabel) {
        self.valid.remove(label);
    }
    /// `UnvalidChildren(L, withroot)`.
    pub fn unvalid_children(&mut self, label: &TdfLabel, with_root: bool) -> Result<(), Failure> {
        for child in descendants(label)? {
            self.valid.remove(&child);
        }
        if with_root {
            self.valid.remove(label);
        }
        Ok(())
    }
    /// `IsValid(L)`: in the map, or anywhere without a map.
    pub fn is_valid(&self, label: &TdfLabel) -> bool {
        !self.with_valid || self.valid.contains(label)
    }
    /// `GetValid`.
    pub fn get_valid(&self) -> &TdfLabelMap {
        &self.valid
    }
    /// `ChangeValid`.
    pub fn change_valid(&mut self) -> &mut TdfLabelMap {
        &mut self.valid
    }
    /// `CurrentShape(NS)`: `TNaming_Tool::CurrentShape` in the scope.
    pub fn current_shape(&self, named: &Handle<TNamingNamedShape>) -> Result<Shape, Failure> {
        if self.with_valid {
            return TNamingTool::current_shape_updated(named, &self.valid);
        }
        TNamingTool::current_shape(named)
    }
}

/// The labels `TDF_ChildIterator(L, true)` walks, when the label has children.
fn descendants(label: &TdfLabel) -> Result<Vec<TdfLabel>, Failure> {
    let mut labels = Vec::new();
    if !label.has_child() {
        return Ok(labels);
    }
    let mut iterator = TdfChildIterator::new(label, true)?;
    while iterator.more() {
        labels.push(iterator.value());
        iterator.next();
    }
    Ok(labels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tklcaf::tdf::TdfData;

    #[test]
    fn children_are_validated_and_unvalidated_with_their_root() {
        let data = TdfData::new();
        let label = data.root().find_child(1, true).unwrap();
        let grandchild = label
            .find_child(1, true)
            .unwrap()
            .find_child(2, true)
            .unwrap();
        let mut scope = TNamingScope::with_mode(true);
        scope.valid_children(&label, true).unwrap();
        assert!(scope.is_valid(&label) && scope.is_valid(&grandchild));
        assert_eq!(scope.get_valid().extent(), 3);
        scope.unvalid_children(&label, false).unwrap();
        assert!(scope.is_valid(&label) && !scope.is_valid(&grandchild));
        scope.unvalid(&label);
        scope.valid(&grandchild);
        assert!(scope.is_valid(&grandchild));
        let copy = TNamingScope::with_map(scope.get_valid());
        assert!(copy.with_valid() && copy.is_valid(&grandchild));
        scope.clear_valid();
        assert!(scope.get_valid().is_empty());
        scope.change_valid().add(label.clone());
        assert!(scope.is_valid(&label));
    }
}
