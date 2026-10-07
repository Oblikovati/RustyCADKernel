// Rust translation of ChFiDS_FilSpine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The guideline of a fillet with its radius: constant per edge or a law of evolution.
mod law;
mod radius;

use super::spine::{Spine, SpineObject};
use super::{ElSpine, errors::domain_error};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::gp::Xy;
use std::ops::{Deref, DerefMut};

/// A fillet guideline (`ChFiDS_FilSpine`): a [`Spine`] with the radius given as
/// `(abscissa, radius)` pairs sorted by abscissa, and one radius law per elementary spine.
///
/// It dereferences to its [`Spine`] part, as the native class inherits it.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::FilSpine;
/// use rustycad_tkmath::gp::Xy;
/// let mut spine = FilSpine::new();
/// spine.set_radius_at(Xy::new(0.0, 2.0), 0)?;
/// spine.set_radius_at(Xy::new(1.0, 2.0), 0)?;
/// assert!(spine.is_constant());
/// assert_eq!(spine.radius()?, 2.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FilSpine {
    spine: Spine,
    par_and_rad: Vec<Xy>,
    laws: Vec<Handle<dyn LawFunction>>,
}
impl_transient!(FilSpine, "ChFiDS_FilSpine", Spine, spine);

impl Default for FilSpine {
    fn default() -> Self {
        Self::from_spine(Spine::new())
    }
}

impl FilSpine {
    /// `ChFiDS_FilSpine()`: an empty fillet guideline; `FilSpine::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFiDS_FilSpine(Tol)`: an empty fillet guideline of tolerance `tolerance`;
    /// `FilSpine::with_tolerance(1e-4)`.
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self::from_spine(Spine::with_tolerance(tolerance))
    }

    fn from_spine(spine: Spine) -> Self {
        Self {
            spine,
            par_and_rad: Vec::new(),
            laws: Vec::new(),
        }
    }

    /// The `(abscissa, radius)` pairs, sorted by abscissa (the native private `parandrad`);
    /// `spine.par_and_rad().len()`.
    pub fn par_and_rad(&self) -> &[Xy] {
        &self.par_and_rad
    }

    /// The radius laws, one per elementary spine; `spine.laws().len()`.
    pub fn laws(&self) -> &[Handle<dyn LawFunction>] {
        &self.laws
    }

    /// The `Reset(false)` completion: the pairs are extended to the bounds of the
    /// guideline, and a periodic guideline ends with the start radius.
    fn complete_par_and_rad(&mut self) -> Result<(), Failure> {
        let start = self.spine.first_parameter();
        let end = self.spine.last_parameter()?;
        let (Some(&first), Some(&last)) = (self.par_and_rad.first(), self.par_and_rad.last())
        else {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "ChFiDS_FilSpine::Reset: no radius is set; expected at least one radius",
            ));
        };
        if (start - first.x()).abs() > rustycad_tkmath::gp::resolution() {
            self.par_and_rad.insert(0, Xy::new(start, first.y()));
        }
        if (end - last.x()).abs() > rustycad_tkmath::gp::resolution() {
            self.par_and_rad.push(Xy::new(end, last.y()));
        }
        if self.spine.is_periodic() {
            let first_radius = self.par_and_rad[0].y();
            let tail = self.par_and_rad.len() - 1;
            self.par_and_rad[tail].set_y(first_radius);
        }
        Ok(())
    }
}

impl SpineObject for FilSpine {
    fn spine(&self) -> &Spine {
        &self.spine
    }
    fn spine_mut(&mut self) -> &mut Spine {
        &mut self.spine
    }
    /// `Reset(AllData)`: the base reset and the laws cleared; the radii are cleared with
    /// `all_data`, else completed to the bounds of the guideline (Standard_NoSuchObject
    /// when no radius is set).
    fn reset(&mut self, all_data: bool) -> Result<(), Failure> {
        SpineObject::reset(&mut self.spine, all_data)?;
        self.laws.clear();
        if all_data {
            self.par_and_rad.clear();
            return Ok(());
        }
        self.complete_par_and_rad()
    }
    /// `AppendElSpine(Els)`: the base append, then the law of the new elementary spine.
    fn append_el_spine(&mut self, el_spine: &Handle<ElSpine>) -> Result<(), Failure> {
        SpineObject::append_el_spine(&mut self.spine, el_spine)?;
        self.append_law(el_spine)
    }
    fn as_fil_spine(&self) -> Option<&FilSpine> {
        Some(self)
    }
    fn as_fil_spine_mut(&mut self) -> Option<&mut FilSpine> {
        Some(self)
    }
}

impl Deref for FilSpine {
    type Target = Spine;
    fn deref(&self) -> &Spine {
        &self.spine
    }
}

impl DerefMut for FilSpine {
    fn deref_mut(&mut self) -> &mut Spine {
        &mut self.spine
    }
}

/// Standard_DomainError for a radius query on a non-constant span.
fn not_constant(what: &str) -> Failure {
    domain_error(what)
}

#[cfg(test)]
#[path = "fil_spine_tests.rs"]
mod tests;
