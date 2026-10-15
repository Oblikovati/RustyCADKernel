// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_DocumentExplorer.cxx (Next) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The step of `XCAFPrs_DocumentExplorer`: into the components of an assembly, to the next
//! component, back up to the parent assembly, or to the next root.
use super::document_explorer::XCAFPrsDocumentExplorer;
use crate::xcaf_doc::XCAFDocShapeTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tklcaf::tdf::TdfLabel;

/// What the walk does with the next child of the stack top.
enum Step {
    /// Skip it (null, empty, or an empty assembly).
    Skip,
    /// Stop on it (a part).
    Part,
    /// Enter it (an assembly with contents).
    Assembly,
}

impl XCAFPrsDocumentExplorer {
    /// `Next`: the next node in depth-first order (`Standard_ProgramError` past the end).
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        if !self.more() {
            return Err(Failure::with_message(
                "Standard_ProgramError",
                "XCAFPrs_DocumentExplorer::Next() - out of range",
            ));
        }
        if self.top_index() < 0 {
            if !self.enter_root()? || !self.leaf_only() {
                return Ok(());
            }
        } else if !self.current().is_assembly {
            self.advance_top();
        }
        self.walk()
    }
    /// Enters the current root: the next root for a part (false), else the root assembly
    /// pushed and, unless only leaves are wanted, made current (true).
    fn enter_root(&mut self) -> Result<bool, Failure> {
        let root = self.root_label();
        let ref_label = referred_label(&root)?;
        if !XCAFDocShapeTool::is_assembly(&ref_label)? {
            self.next_root()?;
            return Ok(false);
        }
        let node = self.assembly_node_of(&root, None)?;
        self.push(node);
        if !self.leaf_only() {
            self.make_current_assembly()?;
        }
        Ok(true)
    }
    /// The loop of `Next` over the children of the stack top.
    fn walk(&mut self) -> Result<(), Failure> {
        loop {
            let top = self.top_node().clone();
            if !top.child_iter.more() {
                if self.pop()? {
                    return Ok(());
                }
                continue;
            }
            let child = top.child_iter.value();
            match classify(&child)? {
                Step::Skip => self.advance_top(),
                Step::Part => {
                    self.set_more(true);
                    return self.make_current_part();
                }
                Step::Assembly => {
                    let node = self.assembly_node_of(&child, Some(&top))?;
                    self.push(node);
                    if !self.leaf_only() {
                        return self.make_current_assembly();
                    }
                }
            }
        }
    }
    /// Pops the stack top: true (done) when the stack is empty and the next root is set,
    /// else advances the new top.
    fn pop(&mut self) -> Result<bool, Failure> {
        self.clear_top();
        if self.top_index() < 0 {
            self.next_root()?;
            return Ok(true);
        }
        self.advance_top();
        Ok(false)
    }
}

/// The step of a child: skipped when null or empty, a part when its referred label is not
/// an assembly, entered when that assembly has contents.
fn classify(child: &TdfLabel) -> Result<Step, Failure> {
    if child.is_null() || (!child.has_child() && !child.has_attribute()?) {
        return Ok(Step::Skip);
    }
    let ref_label = referred_label(child)?;
    if !XCAFDocShapeTool::is_assembly(&ref_label)? {
        return Ok(Step::Part);
    }
    if ref_label.has_attribute()? || ref_label.has_child() {
        return Ok(Step::Assembly);
    }
    Ok(Step::Skip)
}

/// The referred label of a label, else the label itself.
fn referred_label(label: &TdfLabel) -> Result<TdfLabel, Failure> {
    let mut ref_label = label.clone();
    XCAFDocShapeTool::get_referred_shape(label, &mut ref_label)?;
    Ok(ref_label)
}
