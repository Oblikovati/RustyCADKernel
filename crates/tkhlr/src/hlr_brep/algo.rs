//! HLRBRep_Algo.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::InternalAlgo;
use crate::hlr_topo_brep::OutLiner;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

/// HLRBRep_Algo: the exact hidden line algorithm on shapes: each added shape is
/// outlined (HLRTopoBRep_OutLiner) and loaded in the internal algorithm, reached
/// through `Deref`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkhlr::hlr_brep::Algo;
/// let algo = Algo::new()?;
/// algo.add(&Shape::default(), 0);
/// assert_eq!(algo.nb_shapes(), 1);
/// assert_eq!(algo.index(&Shape::default()), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Algo {
    parent: InternalAlgo,
}
impl_transient!(Algo, "HLRBRep_Algo", InternalAlgo, parent);

impl std::ops::Deref for Algo {
    type Target = InternalAlgo;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl Algo {
    /// HLRBRep_Algo(); `Algo::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            parent: InternalAlgo::new()?,
        })
    }

    /// HLRBRep_Algo(A): the data structure, projector and shapes of `a`;
    /// `Algo::from_algo(&a)?`.
    pub fn from_algo(a: &Algo) -> Result<Self, Failure> {
        Ok(Self {
            parent: InternalAlgo::from_algo(a)?,
        })
    }

    /// Add(S, SData, nbIso): outlines `s` and loads it with user data;
    /// `algo.add_with_data(&shape, &data, 0)`.
    pub fn add_with_data(&self, s: &Shape, s_data: &Handle<StandardTransient>, nb_iso: i32) {
        self.load_with_data(&Handle::new(OutLiner::with_original(s)), s_data, nb_iso);
    }

    /// Add(S, nbIso): outlines `s` and loads it; `algo.add(&shape, 0)`.
    pub fn add(&self, s: &Shape, nb_iso: i32) {
        self.load(&Handle::new(OutLiner::with_original(s)), nb_iso);
    }

    /// Index(S): the index of the shape whose original or outlined shape is `s`
    /// (equal: same shape and orientation), 0 when absent; `algo.index(&shape)`.
    pub fn index(&self, s: &Shape) -> i32 {
        let shapes = self.seq_of_shape_bounds();
        for (k, sb) in shapes.iter().enumerate() {
            let Some(o) = sb.shape().get() else { continue };
            if o.original_shape() == *s || o.out_lined_shape() == *s {
                return k as i32 + 1;
            }
        }
        0
    }

    /// OutLinedShapeNullify(): forgets the outlined shapes and their data
    /// structures; `algo.out_lined_shape_nullify()`.
    pub fn out_lined_shape_nullify(&self) {
        let shapes = self.seq_of_shape_bounds();
        for sb in shapes.iter() {
            if let Some(o) = sb.shape().get() {
                o.set_out_lined_shape(&Shape::default());
                o.data_structure().clear();
            }
        }
    }
}
