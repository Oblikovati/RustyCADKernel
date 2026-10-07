//! The edges of a guideline, their vertices and the elementary spines cut from them.
use super::Spine;
use crate::chfi_ds::ElSpine;
use crate::chfi_ds::errors::{null_object, out_of_range};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;

impl Spine {
    /// `SetEdges(E)`: appends `edge` to the guideline; `spine.set_edges(&edge)`.
    pub fn set_edges(&mut self, edge: &Edge) {
        self.edges.push(edge.clone());
    }

    /// `SetOffsetEdges(E)`: appends `edge` to the offset guideline;
    /// `spine.set_offset_edges(&edge)`.
    pub fn set_offset_edges(&mut self, edge: &Edge) {
        self.offset_edges.push(edge.clone());
    }

    /// `PutInFirst(E)`: puts `edge` before all the others; `spine.put_in_first(&edge)`.
    pub fn put_in_first(&mut self, edge: &Edge) {
        self.edges.insert(0, edge.clone());
    }

    /// `PutInFirstOffset(E)`: puts `edge` before all the other offset edges;
    /// `spine.put_in_first_offset(&edge)`.
    pub fn put_in_first_offset(&mut self, edge: &Edge) {
        self.offset_edges.insert(0, edge.clone());
    }

    /// `NbEdges()`; `spine.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.edges.len() as i32
    }

    /// `Edges(I)`: the 1-based `index`-th edge; Standard_OutOfRange outside
    /// `[1, NbEdges]`; `spine.edges(1)?`.
    pub fn edges(&self, index: i32) -> Result<&Edge, Failure> {
        sequence_value(&self.edges, index, "ChFiDS_Spine::Edges")
    }

    /// `OffsetEdges(I)`: the 1-based `index`-th offset edge; `spine.offset_edges(1)?`.
    pub fn offset_edges(&self, index: i32) -> Result<&Edge, Failure> {
        sequence_value(&self.offset_edges, index, "ChFiDS_Spine::OffsetEdges")
    }

    /// `FirstVertex()`: the start vertex of the first edge in its orientation;
    /// Standard_NoSuchObject for an empty guideline; `spine.first_vertex()?`.
    pub fn first_vertex(&self) -> Result<Vertex, Failure> {
        let edge = end_edge(self.edges.first(), "ChFiDS_Spine::FirstVertex")?;
        if edge.orientation() == Orientation::Forward {
            return Ok(topexp::first_vertex(edge, false));
        }
        Ok(topexp::last_vertex(edge, false))
    }

    /// `LastVertex()`: the end vertex of the last edge in its orientation;
    /// Standard_NoSuchObject for an empty guideline; `spine.last_vertex()?`.
    pub fn last_vertex(&self) -> Result<Vertex, Failure> {
        let edge = end_edge(self.edges.last(), "ChFiDS_Spine::LastVertex")?;
        if edge.orientation() == Orientation::Forward {
            return Ok(topexp::last_vertex(edge, false));
        }
        Ok(topexp::first_vertex(edge, false))
    }

    /// `IsClosed()`: whether the first and last vertices are the same; `spine.is_closed()?`.
    pub fn is_closed(&self) -> Result<bool, Failure> {
        let last: Vertex = self.last_vertex()?;
        Ok(self.first_vertex()?.is_same(&last))
    }

    /// `Absc(V)`: the abscissa of `vertex` on the first edge holding it (as an oriented
    /// start or end), or -1 when no edge holds it; `spine.absc_of_vertex(&v)?`.
    pub fn absc_of_vertex(&self, vertex: &Vertex) -> Result<f64, Failure> {
        for (offset, edge) in self.edges.iter().enumerate() {
            let index = offset as i32 + 1;
            if let Some(at_start) = vertex_end(edge, vertex) {
                return if at_start {
                    self.first_parameter_of(index)
                } else {
                    self.last_parameter_of(index)
                };
            }
        }
        Ok(-1.0)
    }

    /// `Index(E)`: the 1-based index of the first edge that is the same as `edge`, or 0;
    /// `spine.index_of_edge(&edge)`.
    pub fn index_of_edge(&self, edge: &Edge) -> i32 {
        self.edges
            .iter()
            .position(|candidate| edge.is_same(candidate))
            .map_or(0, |offset| offset as i32 + 1)
    }

    /// `ElSpine(E)`: the elementary spine over the middle of `edge`;
    /// `spine.el_spine_of_edge(&edge)?`.
    pub fn el_spine_of_edge(&self, edge: &Edge) -> Result<Handle<ElSpine>, Failure> {
        self.el_spine_of_index(self.index_of_edge(edge))
    }

    /// `ElSpine(IE)`: the elementary spine over the middle of the `index`-th edge, the
    /// middle brought into the period of a periodic guideline; `spine.el_spine_of_index(1)?`.
    pub fn el_spine_of_index(&self, index: i32) -> Result<Handle<ElSpine>, Failure> {
        let mut middle = 0.5 * (self.first_parameter_of(index)? + self.last_parameter_of(index)?);
        if self.is_periodic() {
            middle = elclib::in_period(middle, self.first_parameter(), self.last_parameter()?);
        }
        self.el_spine_at(middle)
    }

    /// `ElSpine(W)`: the only elementary spine, else the first whose range holds `w`, else
    /// a null handle; `spine.el_spine_at(0.5)?`.
    pub fn el_spine_at(&self, w: f64) -> Result<Handle<ElSpine>, Failure> {
        if self.el_spines.len() == 1 {
            return Ok(self.el_spines[0].clone());
        }
        for candidate in &self.el_spines {
            let el_spine = candidate
                .get()
                .ok_or_else(|| null_object("ChFiDS_Spine::ElSpine"))?;
            let (first, last) = (el_spine.first_parameter()?, el_spine.last_parameter()?);
            if first <= w && w <= last {
                return Ok(candidate.clone());
            }
        }
        Ok(Handle::default())
    }
}

/// Whether `vertex` is the oriented start (`Some(true)`) or end (`Some(false)`) of `edge`,
/// tested in the native order: start FORWARD, start REVERSED, end FORWARD, end REVERSED.
fn vertex_end(edge: &Edge, vertex: &Vertex) -> Option<bool> {
    let (start, end) = topexp::vertices(edge, false);
    let orientation = edge.orientation();
    let checks = [
        (&start, Orientation::Forward, true),
        (&start, Orientation::Reversed, false),
        (&end, Orientation::Forward, false),
        (&end, Orientation::Reversed, true),
    ];
    checks
        .into_iter()
        .find(|(candidate, wanted, _)| candidate.is_same(vertex) && orientation == *wanted)
        .map(|(_, _, at_start)| at_start)
}

/// `NCollection_Sequence::Value(index)` over a 1-based view of `items`.
fn sequence_value<'a>(items: &'a [Edge], index: i32, member: &str) -> Result<&'a Edge, Failure> {
    let length = items.len() as i32;
    if index < 1 || index > length {
        return Err(out_of_range(member, index, length));
    }
    Ok(&items[(index - 1) as usize])
}

/// `spine.First()` / `spine.Last()`: Standard_NoSuchObject on an empty sequence.
fn end_edge<'a>(edge: Option<&'a Edge>, member: &str) -> Result<&'a Edge, Failure> {
    edge.ok_or_else(|| {
        Failure::with_message(
            "Standard_NoSuchObject",
            &format!("{member}: the guideline has no edge; expected at least one edge"),
        )
    })
}
