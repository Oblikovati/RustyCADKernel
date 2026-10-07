//! `BOPAlgo_ShellSplitter`: splits a set of faces into closed shells.
use super::Options;
use crate::bop_tools::{ConnexityBlock, make_connexity_blocks_from_shapes, orient_faces_on_shell};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Shell};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[path = "shell_splitter/refine.rs"]
mod refine;
#[path = "shell_splitter/split_block.rs"]
mod split_block;

/// Builds closed shells from faces: a regular connexity block is one shell, the others are
/// walked face by face choosing the neighbour with the smallest angle.
///
/// ```
/// use rustycad_tkbo::bop_algo::ShellSplitter;
/// let mut splitter = ShellSplitter::new();
/// splitter.perform().unwrap();
/// assert!(splitter.shells().is_empty());
/// ```
#[derive(Default)]
pub struct ShellSplitter {
    options: Options,
    start_shapes: List<Shape>,
    shells: List<Shape>,
    blocks: Vec<ConnexityBlock>,
}

impl ShellSplitter {
    /// An empty splitter (`BOPAlgo_ShellSplitter()`); `ShellSplitter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one face to split (`AddStartElement`); `splitter.add_start_element(&face)`.
    pub fn add_start_element(&mut self, face: &Shape) {
        self.start_shapes.append(face.clone());
    }

    /// The faces to split (`StartElements`); `splitter.start_elements().len()`.
    pub fn start_elements(&self) -> &List<Shape> {
        &self.start_shapes
    }

    /// The built closed shells (`Shells`); `splitter.shells().len()`.
    pub fn shells(&self) -> &List<Shape> {
        &self.shells
    }

    /// The report and run options; `splitter.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutable report and run options; `splitter.options_mut().set_run_parallel(true)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Splits the faces into shells (`Perform`); `splitter.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.options.clear();
        let start: Vec<Shape> = self.start_shapes.iter().cloned().collect();
        self.blocks.clear();
        make_connexity_blocks_from_shapes(
            &start,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut self.blocks,
        )?;
        self.make_shells()
    }

    /// One closed shell per regular block, the split shells of the others (`MakeShells`).
    fn make_shells(&mut self) -> Result<(), Failure> {
        self.shells.clear();
        let mut irregular: Vec<usize> = Vec::new();
        for (position, block) in self.blocks.iter().enumerate() {
            if !block.is_regular() {
                irregular.push(position);
                continue;
            }
            let shell = make_shell(block.shapes())?;
            shell.set_closed(true)?;
            self.shells.append(shell);
        }
        for position in &irregular {
            Self::split_block(&mut self.blocks[*position])?;
        }
        for position in irregular {
            for shell in self.blocks[position].loops().iter() {
                shell.set_closed(true)?;
                self.shells.append(shell.clone());
            }
        }
        Ok(())
    }
}

/// A shell of the faces with consistently oriented faces (`MakeShell`).
fn make_shell(faces: &List<Shape>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell = shell.into_shape();
    for face in faces.iter() {
        builder.add(&mut shell, face)?;
    }
    orient_faces_on_shell(&mut shell)?;
    Ok(shell)
}
