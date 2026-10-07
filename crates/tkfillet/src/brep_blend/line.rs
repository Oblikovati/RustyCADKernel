// Rust translation of BRepBlend_Line.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_Line`: a blend line, the sequence of the sections that the walking found,
//! with its extremities and its transitions on the surfaces.
use super::BRepBlendExtremity;
use crate::blend::BlendPoint;
use crate::chfi_ds::Shared;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;

/// A shared, mutable `occ::handle<BRepBlend_Line>`: the walkings fill the line their
/// callers hold.
pub type BRepBlendLineHandle = Shared<BRepBlendLine>;

/// `BRepBlend_Line`: the points (1-based, as the native sequence), the start and end
/// extremities on both surfaces, and the transitions of the line on them.
///
/// ```
/// use rustycad_tkfillet::blend::BlendPoint;
/// use rustycad_tkfillet::brep_blend::BRepBlendLine;
/// let mut line = BRepBlendLine::new();
/// line.append(BlendPoint::default());
/// assert_eq!(line.nb_points(), 1);
/// assert!(line.transition_on_s1().is_err());
/// ```
#[derive(Clone)]
pub struct BRepBlendLine {
    parent: StandardTransient,
    seqpt: Vec<BlendPoint>,
    tras1: IntSurfTypeTrans,
    tras2: IntSurfTypeTrans,
    stp1: BRepBlendExtremity,
    stp2: BRepBlendExtremity,
    endp1: BRepBlendExtremity,
    endp2: BRepBlendExtremity,
    hass1: bool,
    hass2: bool,
}
impl_transient!(BRepBlendLine, "BRepBlend_Line", StandardTransient, parent);

impl Default for BRepBlendLine {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            seqpt: Vec::new(),
            tras1: IntSurfTypeTrans::Undecided,
            tras2: IntSurfTypeTrans::Undecided,
            stp1: BRepBlendExtremity::new(),
            stp2: BRepBlendExtremity::new(),
            endp1: BRepBlendExtremity::new(),
            endp2: BRepBlendExtremity::new(),
            hass1: false,
            hass2: false,
        }
    }
}

fn out_of_range(member: &str, index: String, bounds: String) -> Failure {
    Failure::out_of_range(&format!(
        "BRepBlend_Line::{member} index {index}; expected {bounds}"
    ))
}

fn no_transition(member: &str) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("BRepBlend_Line::{member}: no transition set"),
    )
}

impl BRepBlendLine {
    /// `BRepBlend_Line()`: empty, undecided transitions.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Clear()`: no point and no transition; the extremities are kept.
    pub fn clear(&mut self) {
        self.seqpt.clear();
        (self.hass1, self.hass2) = (false, false);
        (self.tras1, self.tras2) = (IntSurfTypeTrans::Undecided, IntSurfTypeTrans::Undecided);
    }

    /// `Append(P)`.
    pub fn append(&mut self, p: BlendPoint) {
        self.seqpt.push(p);
    }

    /// `Prepend(P)`.
    pub fn prepend(&mut self, p: BlendPoint) {
        self.seqpt.insert(0, p);
    }

    /// `InsertBefore(Index, P)`: Standard_OutOfRange outside `[1, NbPoints + 1]`.
    pub fn insert_before(&mut self, index: i32, p: BlendPoint) -> Result<(), Failure> {
        let len = self.seqpt.len();
        match usize::try_from(index) {
            Ok(i) if (1..=len + 1).contains(&i) => {
                self.seqpt.insert(i - 1, p);
                Ok(())
            }
            _ => Err(out_of_range(
                "InsertBefore",
                index.to_string(),
                format!("one in [1, {}]", len + 1),
            )),
        }
    }

    /// `Remove(FromIndex, ToIndex)`: Standard_OutOfRange unless
    /// `1 <= FromIndex <= ToIndex <= NbPoints`.
    pub fn remove(&mut self, from_index: i32, to_index: i32) -> Result<(), Failure> {
        let len = self.seqpt.len();
        match (usize::try_from(from_index), usize::try_from(to_index)) {
            (Ok(from), Ok(to)) if from >= 1 && from <= to && to <= len => {
                self.seqpt.drain(from - 1..to);
                Ok(())
            }
            _ => Err(out_of_range(
                "Remove",
                format!("[{from_index}, {to_index}]"),
                format!("a range in [1, {len}]"),
            )),
        }
    }

    /// `Set(TranS1, TranS2)`: the transitions on both surfaces.
    pub fn set_transitions(&mut self, tran_s1: IntSurfTypeTrans, tran_s2: IntSurfTypeTrans) {
        (self.hass1, self.hass2) = (true, true);
        (self.tras1, self.tras2) = (tran_s1, tran_s2);
    }

    /// `Set(Trans)`: the transition on the only surface (a line on a restriction).
    pub fn set_transition(&mut self, trans: IntSurfTypeTrans) {
        (self.hass1, self.hass2) = (true, false);
        self.tras1 = trans;
    }

    /// `SetStartPoints(StartPtOnS1, StartPtOnS2)`.
    pub fn set_start_points(
        &mut self,
        start_pt_on_s1: BRepBlendExtremity,
        start_pt_on_s2: BRepBlendExtremity,
    ) {
        (self.stp1, self.stp2) = (start_pt_on_s1, start_pt_on_s2);
    }

    /// `SetEndPoints(EndPtOnS1, EndPtOnS2)`.
    pub fn set_end_points(
        &mut self,
        end_pt_on_s1: BRepBlendExtremity,
        end_pt_on_s2: BRepBlendExtremity,
    ) {
        (self.endp1, self.endp2) = (end_pt_on_s1, end_pt_on_s2);
    }

    /// `NbPoints()`.
    pub fn nb_points(&self) -> i32 {
        self.seqpt.len() as i32
    }

    /// `Point(Index)`: 1-based; Standard_OutOfRange outside `[1, NbPoints]`.
    pub fn point(&self, index: i32) -> Result<&BlendPoint, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.seqpt.get(i))
            .ok_or_else(|| {
                out_of_range(
                    "Point",
                    index.to_string(),
                    format!("one in [1, {}]", self.seqpt.len()),
                )
            })
    }

    /// `TransitionOnS1()`: Standard_DomainError before a `Set`.
    pub fn transition_on_s1(&self) -> Result<IntSurfTypeTrans, Failure> {
        if !self.hass1 {
            return Err(no_transition("TransitionOnS1"));
        }
        Ok(self.tras1)
    }

    /// `TransitionOnS2()`: Standard_DomainError unless set by `Set(TranS1, TranS2)`.
    pub fn transition_on_s2(&self) -> Result<IntSurfTypeTrans, Failure> {
        if !self.hass2 {
            return Err(no_transition("TransitionOnS2"));
        }
        Ok(self.tras2)
    }

    /// `TransitionOnS()`: the transition on the only surface.
    pub fn transition_on_s(&self) -> Result<IntSurfTypeTrans, Failure> {
        if !self.hass1 {
            return Err(no_transition("TransitionOnS"));
        }
        Ok(self.tras1)
    }

    /// `StartPointOnFirst()`.
    pub fn start_point_on_first(&self) -> &BRepBlendExtremity {
        &self.stp1
    }

    /// `StartPointOnSecond()`.
    pub fn start_point_on_second(&self) -> &BRepBlendExtremity {
        &self.stp2
    }

    /// `EndPointOnFirst()`.
    pub fn end_point_on_first(&self) -> &BRepBlendExtremity {
        &self.endp1
    }

    /// `EndPointOnSecond()`.
    pub fn end_point_on_second(&self) -> &BRepBlendExtremity {
        &self.endp2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(parameter: f64) -> BlendPoint {
        let mut p = BlendPoint::default();
        p.set_parameter(parameter);
        p
    }

    #[test]
    fn the_sequence_is_one_based() {
        let mut line = BRepBlendLine::new();
        line.append(point(2.0));
        line.prepend(point(1.0));
        line.insert_before(3, point(3.0)).unwrap();
        assert!(line.insert_before(5, point(9.0)).is_err());
        assert_eq!(line.point(3).unwrap().parameter(), 3.0);
        line.remove(1, 2).unwrap();
        assert_eq!(line.point(1).unwrap().parameter(), 3.0);
        assert!(line.remove(1, 2).is_err());
    }

    #[test]
    fn clear_resets_the_transitions_only() {
        let mut line = BRepBlendLine::new();
        line.set_transition(IntSurfTypeTrans::In);
        assert_eq!(line.transition_on_s().unwrap(), IntSurfTypeTrans::In);
        assert!(line.transition_on_s2().is_err());
        line.set_start_points(
            BRepBlendExtremity::on_curve(Default::default(), 4.0, 0.0, 0.0),
            BRepBlendExtremity::new(),
        );
        line.clear();
        assert!(line.transition_on_s1().is_err());
        assert_eq!(line.start_point_on_first().parameter(), 4.0);
    }
}
