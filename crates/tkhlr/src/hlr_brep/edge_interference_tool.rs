//! HLRBRep_EdgeInterferenceTool.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Data;
use crate::hlr_algo::{Interference, Intersection};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Direction3};

/// HLRBRep_EdgeInterferenceTool: the geometry of the current edge of a data
/// structure and of its interferences for the transition and vertex tools: the two
/// end vertices of the edge (`InitVertices` ...), the local 2D geometry of the edge
/// and of the face edge of an interference.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_brep::{Data, EdgeInterferenceTool};
/// let mut tool = EdgeInterferenceTool::new(&Handle::new(Data::new(0, 0, 0)?));
/// tool.init_vertices();
/// assert!(tool.more_vertices() && !tool.is_periodic());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct EdgeInterferenceTool {
    ds: Handle<Data>,
    inter: [Intersection; 2],
    // Natively uninitialized until InitVertices; 0 here.
    cur: usize,
}

// The 3D direction of a 2D one (`gp_Dir::SetCoord(x, y, 0)`).
fn in_plane(d: Direction2) -> Result<Direction3, Failure> {
    Direction3::new(d.x(), d.y(), 0.)
}

impl EdgeInterferenceTool {
    /// HLRBRep_EdgeInterferenceTool(DS); `EdgeInterferenceTool::new(&ds)`.
    pub fn new(ds: &Handle<Data>) -> Self {
        Self {
            ds: ds.clone(),
            inter: [Intersection::new(); 2],
            cur: 0,
        }
    }

    fn data(&self) -> Result<&Data, Failure> {
        self.ds.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_EdgeInterferenceTool: no data structure; expected a HLRBRep_Data handle",
            )
        })
    }

    /// LoadEdge(): the bounds, tolerances and vertices of the current edge of the
    /// data structure; `tool.load_edge()?`.
    pub fn load_edge(&mut self) -> Result<(), Failure> {
        let (p1, t1, p2, t2, v_sta, v_end) = {
            let ds = self.data()?;
            let e = ds.edge()?;
            let ed = ds.e_data_array();
            let ed = ed
                .value(e)
                .map_err(|x| x.into_failure("HLRBRep_EdgeInterferenceTool::LoadEdge"))?;
            let (p1, t1, p2, t2) = ed.status().bounds();
            (p1, t1, p2, t2, ed.v_sta(), ed.v_end())
        };
        self.inter[0].set_parameter(p1);
        self.inter[0].set_tolerance(t1);
        self.inter[0].set_index(v_sta);
        self.inter[1].set_parameter(p2);
        self.inter[1].set_tolerance(t2);
        self.inter[1].set_index(v_end);
        Ok(())
    }

    /// InitVertices(); `tool.init_vertices()`.
    pub fn init_vertices(&mut self) {
        self.cur = 0;
    }

    /// MoreVertices(); `tool.more_vertices()`.
    pub fn more_vertices(&self) -> bool {
        self.cur < 2
    }

    /// NextVertex(); `tool.next_vertex()`.
    pub fn next_vertex(&mut self) {
        self.cur += 1;
    }

    /// CurrentVertex(): the start or end vertex of the edge; `tool.current_vertex()?`.
    pub fn current_vertex(&self) -> Result<&Intersection, Failure> {
        self.inter.get(self.cur).ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "HLRBRep_EdgeInterferenceTool: vertex {} after the two ends; expected 0 or 1",
                    self.cur
                ),
            )
        })
    }

    /// CurrentOrientation(): FORWARD at the start, REVERSED at the end;
    /// `tool.current_orientation()`.
    pub fn current_orientation(&self) -> Orientation {
        if self.cur == 0 {
            Orientation::Forward
        } else {
            Orientation::Reversed
        }
    }

    /// CurrentParameter(); `tool.current_parameter()?`.
    pub fn current_parameter(&self) -> Result<f64, Failure> {
        Ok(self.current_vertex()?.parameter())
    }

    /// IsPeriodic(): false; `tool.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        false
    }

    /// EdgeGeometry(Param, Tgt, Nrm, Curv): the tangent, normal (in the view plane)
    /// and curvature of the current edge; `let (t, n, c) = tool.edge_geometry(0.5)?;`.
    pub fn edge_geometry(&self, param: f64) -> Result<(Direction3, Direction3, f64), Failure> {
        let (tg, nm, cr) = self.data()?.local_le_geometry_2d(param)?;
        Ok((in_plane(tg)?, in_plane(nm)?, cr))
    }

    /// ParameterOfInterference(I); `tool.parameter_of_interference(&i)`.
    pub fn parameter_of_interference(&self, i: &Interference) -> f64 {
        i.intersection().parameter()
    }

    /// SameInterferences(I1, I2): both at the same vertex (non zero indices);
    /// `tool.same_interferences(&a, &b)`.
    pub fn same_interferences(&self, i1: &Interference, i2: &Interference) -> bool {
        let (ind1, ind2) = (i1.intersection().index(), i2.intersection().index());
        ind1 != 0 && ind2 != 0 && ind1 == ind2
    }

    /// SameVertexAndInterference(I): the interference is at the current vertex, or
    /// has its orientation; `tool.same_vertex_and_interference(&i)`.
    pub fn same_vertex_and_interference(&self, i: &Interference) -> bool {
        let index = self.inter.get(self.cur).map_or(0, |v| v.index());
        if i.intersection().index() == index {
            return true;
        }
        i.intersection().orientation() == self.current_orientation()
    }

    /// InterferenceBoundaryGeometry(I, Tang, Norm, Curv): the tangent, normal and
    /// curvature of the face edge of the interference;
    /// `let (t, n, c) = tool.interference_boundary_geometry(&i)?;`.
    pub fn interference_boundary_geometry(
        &self,
        i: &Interference,
    ) -> Result<(Direction3, Direction3, f64), Failure> {
        let (fe, param) = i.boundary().value_2d();
        let (tg, nm, cr) = self.data()?.local_fe_geometry_2d(fe, param)?;
        Ok((in_plane(tg)?, in_plane(nm)?, cr))
    }
}
