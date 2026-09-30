// IntTools_PntOn2Faces.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Two face points and their independent validity flag.
use super::PntOnFace;
/// `IntTools_PntOn2Faces`; `PntOn2Faces::default().is_valid() == false`.
#[derive(Clone, Debug, Default)]
pub struct PntOn2Faces {
    valid: bool,
    first: PntOnFace,
    second: PntOnFace,
}
impl PntOn2Faces {
    /// Copies two face points with validity false; `PntOn2Faces::new(&first, &second)`.
    pub fn new(first: &PntOnFace, second: &PntOnFace) -> Self {
        Self {
            valid: false,
            first: first.clone(),
            second: second.clone(),
        }
    }
    /// Copies the first face point; `pair.set_p1(&point)`.
    pub fn set_p1(&mut self, point: &PntOnFace) {
        self.first = point.clone();
    }
    /// Copies the second face point; `pair.set_p2(&point)`.
    pub fn set_p2(&mut self, point: &PntOnFace) {
        self.second = point.clone();
    }
    /// Replaces the pair validity; `pair.set_valid(true)`.
    pub fn set_valid(&mut self, valid: bool) {
        self.valid = valid;
    }
    /// Borrows the first face point; `pair.p1()`.
    pub fn p1(&self) -> &PntOnFace {
        &self.first
    }
    /// Borrows the second face point; `pair.p2()`.
    pub fn p2(&self) -> &PntOnFace {
        &self.second
    }
    /// Reads the pair validity; `pair.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}
