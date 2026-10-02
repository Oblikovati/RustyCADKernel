//! Common pave blocks and face indices for coincidence groups.
use super::{PaveBlock, PaveBlockHandle};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

/// `BOPDS_CommonBlock`; groups coincident edge intervals and incident faces.
#[derive(Default)]
pub struct CommonBlock {
    pave_blocks: Vec<PaveBlockHandle>,
    faces: Vec<i32>,
    tolerance: f64,
}

/// Shared ownership mirrors an OCCT handle to `BOPDS_CommonBlock`.
pub type CommonBlockHandle = Arc<Mutex<CommonBlock>>;

impl CommonBlock {
    /// Creates an empty group; `CommonBlock::default().tolerance() == 0.0`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a block, prepending it only when its original edge beats the current first.
    pub fn add_pave_block(&mut self, block: PaveBlockHandle) {
        let prepend = self
            .pave_blocks
            .first()
            .is_some_and(|first| source_edge(&block) < source_edge(first));
        if prepend {
            self.pave_blocks.insert(0, block);
            return;
        }
        self.pave_blocks.push(block);
    }

    /// Replaces blocks while applying `add_pave_block` ordering; `group.set_pave_blocks(blocks)`.
    pub fn set_pave_blocks(&mut self, blocks: &[PaveBlockHandle]) {
        self.pave_blocks.clear();
        for block in blocks {
            self.add_pave_block(Arc::clone(block));
        }
    }

    /// Borrows grouped pave blocks in their native list order.
    pub fn pave_blocks(&self) -> &[PaveBlockHandle] {
        &self.pave_blocks
    }

    /// Appends one face index, retaining duplicates.
    pub fn add_face(&mut self, face: i32) {
        self.faces.push(face);
    }

    /// Replaces the face list, retaining duplicates and order.
    pub fn set_faces(&mut self, faces: &[i32]) {
        self.faces.clear();
        self.faces.extend_from_slice(faces);
    }

    /// Appends and empties `faces`, matching `NCollection_List::Append` move semantics.
    pub fn append_faces(&mut self, faces: &mut Vec<i32>) {
        self.faces.append(faces);
    }

    /// Borrows face indices in their native list order.
    pub fn faces(&self) -> &[i32] {
        &self.faces
    }

    /// Returns the first grouped block, or Standard_NoSuchObject when the native list is empty.
    pub fn pave_block1(&self) -> Result<&PaveBlockHandle, Failure> {
        self.pave_blocks.first().ok_or_else(|| {
            Failure::with_message(
                "Standard_NoSuchObject",
                "NCollection_List::First on empty common block; expected at least one pave block",
            )
        })
    }

    /// Clones the block handle whose source edge matches `edge`.
    pub fn pave_block_on_edge(&self, edge: i32) -> Option<PaveBlockHandle> {
        self.pave_blocks
            .iter()
            .find(|block| source_edge(block) == edge)
            .map(Arc::clone)
    }

    /// Tests whether a grouped block has source edge `edge`.
    pub fn is_pave_block_on_edge(&self, edge: i32) -> bool {
        self.pave_blocks
            .iter()
            .any(|block| source_edge(block) == edge)
    }

    /// Tests handle identity, as OCCT handle equality does.
    pub fn contains_pave_block(&self, target: &PaveBlockHandle) -> bool {
        self.pave_blocks
            .iter()
            .any(|block| Arc::ptr_eq(block, target))
    }

    /// Tests whether the face list contains `face`.
    pub fn contains_face(&self, face: i32) -> bool {
        self.faces.contains(&face)
    }

    /// Sets the result edge on every grouped pave block.
    pub fn set_edge(&mut self, edge: i32) {
        for block in &self.pave_blocks {
            block
                .lock()
                .expect("pave-block lock must not be poisoned")
                .set_edge(edge);
        }
    }

    /// Reads the result edge from the first block, or `-1` for empty/null first entries.
    pub fn edge(&self) -> Result<i32, Failure> {
        let block = self.pave_block1()?;
        Ok(block
            .lock()
            .expect("pave-block lock must not be poisoned")
            .edge())
    }

    /// Tests whether a face is present in the grouped face list.
    pub fn is_pave_block_on_face(&self, face: i32) -> bool {
        self.contains_face(face)
    }

    /// Moves an identical handle to the front; handles not already present are ignored.
    pub fn set_real_pave_block(&mut self, target: &PaveBlockHandle) {
        let Some(position) = self
            .pave_blocks
            .iter()
            .position(|block| Arc::ptr_eq(block, target))
        else {
            return;
        };
        let block = self.pave_blocks.remove(position);
        self.pave_blocks.insert(0, block);
    }

    /// Sets the common tolerance; `group.set_tolerance(1.0e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }

    /// Reads the common tolerance; `CommonBlock::default().tolerance() == 0.0`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// Writes native `Dump` text; `group.dump(&mut String::new())?`.
    pub fn dump(&self, output: &mut dyn Write) -> std::fmt::Result {
        output.write_str(" -- CB:\n")?;
        self.dump_pave_blocks(output)?;
        self.dump_faces(output)
    }

    /// Writes each block's dump followed by the native newline.
    fn dump_pave_blocks(&self, output: &mut dyn Write) -> std::fmt::Result {
        for block in &self.pave_blocks {
            block
                .lock()
                .expect("pave-block lock must not be poisoned")
                .dump(output)?;
            output.write_char('\n')?;
        }
        Ok(())
    }

    /// Writes faces only when present, preserving list order and duplicates.
    fn dump_faces(&self, output: &mut dyn Write) -> std::fmt::Result {
        if self.faces.is_empty() {
            return Ok(());
        }
        output.write_str(" Faces:")?;
        for face in &self.faces {
            write!(output, " {face}")?;
        }
        output.write_char('\n')
    }
}

/// Allocates one mutable shared pave-block handle; `pave_block_handle(PaveBlock::default())`.
pub fn pave_block_handle(block: PaveBlock) -> PaveBlockHandle {
    Arc::new(Mutex::new(block))
}

/// Allocates one mutable shared common-block handle.
pub fn common_block_handle(block: CommonBlock) -> CommonBlockHandle {
    Arc::new(Mutex::new(block))
}

/// Reads one shared block's source edge index.
fn source_edge(block: &PaveBlockHandle) -> i32 {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge()
}
