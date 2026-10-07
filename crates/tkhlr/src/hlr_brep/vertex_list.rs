//! HLRBRep_VertexList.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::EdgeInterferenceTool;
use crate::hlr_algo::{Interference, Intersection};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// HLRBRep_VertexList: the vertices of an edge in parameter order: its two ends
/// (from the tool) merged with its sorted interferences. The native list iterator
/// is the slice of the interferences from the iterator position on.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_brep::{Data, EdgeInterferenceTool, VertexList};
/// let tool = EdgeInterferenceTool::new(&Handle::new(Data::new(0, 0, 0)?));
/// let mut list = VertexList::new(&tool, &[])?;
/// assert!(list.more() && list.is_boundary() && !list.is_interference());
/// list.next()?;
/// list.next()?;
/// assert!(!list.more());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct VertexList<'a> {
    interferences: &'a [Interference],
    position: usize,
    tool: EdgeInterferenceTool,
    from_edge: bool,
    from_interf: bool,
}

fn domain_error(member: &str) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("HLRBRep_VertexList::{member}"),
    )
}

impl<'a> VertexList<'a> {
    /// HLRBRep_VertexList(T, I): a copy of the tool and the interferences from `i`;
    /// `VertexList::new(&tool, &list)?`.
    pub fn new(t: &EdgeInterferenceTool, i: &'a [Interference]) -> Result<Self, Failure> {
        let mut list = Self {
            interferences: i,
            position: 0,
            tool: t.clone(),
            from_edge: false,
            from_interf: false,
        };
        list.tool.init_vertices();
        list.next()?;
        Ok(list)
    }

    fn interference(&self) -> Option<&'a Interference> {
        self.interferences.get(self.position)
    }

    /// IsPeriodic(); `list.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.tool.is_periodic()
    }

    /// More(); `list.more()`.
    pub fn more(&self) -> bool {
        self.from_edge || self.from_interf
    }

    /// Next(): the next vertex: the end of the edge or the interference of lesser
    /// parameter (both when the interference is at that end); `list.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        if self.from_interf {
            self.position += 1;
        }
        if self.from_edge {
            self.tool.next_vertex();
        }
        self.from_interf = self.interference().is_some();
        self.from_edge = self.tool.more_vertices();
        if let (true, Some(i)) = (self.from_edge, self.interference())
            && !self.tool.same_vertex_and_interference(i)
        {
            if self.tool.current_parameter()? < self.tool.parameter_of_interference(i) {
                self.from_interf = false;
            } else {
                self.from_edge = false;
            }
        }
        Ok(())
    }

    /// Current(): the vertex of the edge end, else of the interference;
    /// `list.current()?`.
    pub fn current(&self) -> Result<&Intersection, Failure> {
        if self.from_edge {
            return self.tool.current_vertex();
        }
        match (self.from_interf, self.interference()) {
            (true, Some(i)) => Ok(i.intersection()),
            _ => Err(Failure::with_message(
                "Standard_NoSuchObject",
                "HLRBRep_VertexList::Current",
            )),
        }
    }

    /// IsBoundary(): the vertex is an end of the edge; `list.is_boundary()`.
    pub fn is_boundary(&self) -> bool {
        self.from_edge
    }

    /// IsInterference(); `list.is_interference()`.
    pub fn is_interference(&self) -> bool {
        self.from_interf
    }

    /// Orientation(): of the edge end; Standard_DomainError on an interference;
    /// `list.orientation()?`.
    pub fn orientation(&self) -> Result<Orientation, Failure> {
        if self.from_edge {
            return Ok(self.tool.current_orientation());
        }
        Err(domain_error("Orientation"))
    }

    /// Transition(): of the interference; `list.transition()?`.
    pub fn transition(&self) -> Result<Orientation, Failure> {
        match (self.from_interf, self.interference()) {
            (true, Some(i)) => Ok(i.transition()),
            _ => Err(domain_error("Transition")),
        }
    }

    /// BoundaryTransition(): of the interference; `list.boundary_transition()?`.
    pub fn boundary_transition(&self) -> Result<Orientation, Failure> {
        match (self.from_interf, self.interference()) {
            (true, Some(i)) => Ok(i.boundary_transition()),
            _ => Err(domain_error("BoundaryTransition")),
        }
    }
}
