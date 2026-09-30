// Rust translation of BRepClass3d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass3d::OuterShell`: the shell of a solid that bounds it from outside.
use super::BRepClass3dSolidClassifier;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, Shell, Solid};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// `BRepClass3d`: the package functions.
///
/// ```
/// use rustycad_tkbrep::topods::Solid;
/// use rustycad_tktopalgo::brep_class3d::BRepClass3d;
/// // A null solid has no outer shell.
/// assert!(BRepClass3d::outer_shell(&Solid::default())?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepClass3d;

impl BRepClass3d {
    /// `OuterShell(S)`: the only shell of the solid (without checking its
    /// orientation), else the first shell whose first face is not INTERNAL and for
    /// which the solid of that shell alone classifies the infinite point OUT
    /// (tolerance 1e-7); a null shell for a null solid, a solid without shells or when
    /// no shell qualifies; `BRepClass3d::outer_shell(&solid)?`.
    pub fn outer_shell(solid: &Solid) -> Result<Shell, Failure> {
        if solid.is_null() {
            return Ok(Shell::default());
        }
        let shells = shells_of(solid)?;
        // if solid has one shell, it will return, without checking orientation
        match shells.len() {
            0 => return Ok(Shell::default()),
            1 => return Ok(shells[0].clone()),
            _ => {}
        }
        let mut classifier = BRepClass3dSolidClassifier::new();
        for shell in shells {
            if is_internal(&shell) {
                continue;
            }
            let mut alone = solid.empty_copied()?;
            BRepBuilder::default().add(&mut alone, &shell)?;
            classifier.load(&alone)?;
            classifier.perform_infinite_point(1.0e-7)?;
            if classifier.state() == State::Out {
                return Ok(shell);
            }
        }
        Ok(Shell::default())
    }
}

/// The shells directly under the solid (`TopoDS_Iterator`), as `TopoDS_Shell`.
fn shells_of(solid: &Solid) -> Result<Vec<Shell>, Failure> {
    let mut shells = Vec::new();
    let mut iterator = ShapeIterator::new(solid, true, true);
    while iterator.more() {
        let child: &Shape = iterator.value()?;
        if child.shape_type()? == ShapeEnum::Shell {
            shells.push(Shell::unchecked(child.clone()));
        }
        iterator.next();
    }
    Ok(shells)
}

/// `IsInternal(aSx)`: the first sub-shape of the shell is INTERNAL.
fn is_internal(shell: &Shell) -> bool {
    let iterator = ShapeIterator::new(shell, true, true);
    iterator.more()
        && iterator
            .value()
            .is_ok_and(|first| first.orientation() == Orientation::Internal)
}
