// Rust translation of BRepPrim_GWedge.hxx/.cxx (constructors, accessors, open/close,
// the shell and the existence tests) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_GWedge`: the generic wedge (boxes and wedges), its data and the tests of
//! existence of its sub-shapes. The geometry of the sub-shapes is in `gwedge_geometry.rs`
//! and their construction in `gwedge_topology.rs`.
use super::{BRepPrimBuilder, BRepPrimDirection};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shell, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Axis2;

/// `NBFACES`, `NBWIRES`, `NBEDGES`, `NBVERTICES`.
const NB_FACES: usize = 6;
const NB_EDGES: usize = 12;
const NB_VERTICES: usize = 8;

/// `val`: the contribution of a direction to the index of a vertex.
const VAL: [usize; 6] = [0, 4, 0, 2, 0, 1];
/// `tab`: the index of the edge between two faces, -1 between faces of the same axis.
const TAB: [[i32; 6]; 6] = [
    [-1, -1, 0, 1, 8, 9],
    [-1, -1, 2, 3, 10, 11],
    [0, 2, -1, -1, 4, 5],
    [1, 3, -1, -1, 6, 7],
    [8, 10, 4, 6, -1, -1],
    [9, 11, 5, 7, -1, -1],
];

/// The Standard_DomainError of the wedge, with the offending directions.
pub(crate) fn wedge_domain_error(member: &str, detail: &str) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("BRepPrim_GWedge::{member}: {detail}"),
    )
}

/// `BRepPrim_Wedge_NumDir1`: the index of the face in direction `d1`.
pub(crate) fn num_dir1(d1: BRepPrimDirection) -> usize {
    d1 as usize
}

/// `BRepPrim_Wedge_NumDir2`: the index of the edge between the faces `d1` and `d2`;
/// Standard_DomainError when they are normal to the same axis.
pub(crate) fn num_dir2(d1: BRepPrimDirection, d2: BRepPrimDirection) -> Result<usize, Failure> {
    let (i1, i2) = (num_dir1(d1), num_dir1(d2));
    if i1 / 2 == i2 / 2 {
        return Err(wedge_domain_error(
            "Edge",
            &format!("{d1:?} and {d2:?} are normal to the same axis; expected two axes"),
        ));
    }
    Ok(TAB[i1][i2] as usize)
}

/// `BRepPrim_Wedge_NumDir3`: the index of the vertex of the faces `d1`, `d2`, `d3`;
/// Standard_DomainError when two of them are normal to the same axis.
pub(crate) fn num_dir3(
    d1: BRepPrimDirection,
    d2: BRepPrimDirection,
    d3: BRepPrimDirection,
) -> Result<usize, Failure> {
    let (i1, i2, i3) = (num_dir1(d1), num_dir1(d2), num_dir1(d3));
    if i1 / 2 == i2 / 2 || i2 / 2 == i3 / 2 || i3 / 2 == i1 / 2 {
        return Err(wedge_domain_error(
            "Vertex",
            &format!("{d1:?}, {d2:?} and {d3:?} share an axis; expected three axes"),
        ));
    }
    Ok(VAL[i1] + VAL[i2] + VAL[i3])
}

/// `BRepPrim_GWedge`: a wedge in the frame `Axes`, limited by XMin..XMax, YMin..YMax,
/// ZMin..ZMax on its YMin face and by X2Min..X2Max, Z2Min..Z2Max on its YMax face; each
/// of the six directions can be opened (docs/prim-design.md section 2).
///
/// ```
/// use rustycad_tkmath::gp::Axis2;
/// use rustycad_tkprim::brep_prim::{BRepPrimBuilder, BRepPrimDirection, BRepPrimGWedge};
/// let mut wedge = BRepPrimGWedge::with_box(BRepPrimBuilder::new(), Axis2::default(), 1.0, 2.0, 3.0);
/// assert!(wedge.has_face(BRepPrimDirection::ZMax));
/// assert!(!wedge.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepPrimGWedge {
    pub(crate) my_builder: BRepPrimBuilder,
    pub(crate) my_axes: Axis2,
    pub(crate) xmin: f64,
    pub(crate) xmax: f64,
    pub(crate) ymin: f64,
    pub(crate) ymax: f64,
    pub(crate) zmin: f64,
    pub(crate) zmax: f64,
    pub(crate) z2min: f64,
    pub(crate) z2max: f64,
    pub(crate) x2min: f64,
    pub(crate) x2max: f64,
    pub(crate) my_shell: Shell,
    pub(crate) shell_built: bool,
    pub(crate) my_vertices: [Vertex; NB_VERTICES],
    pub(crate) vertices_built: [bool; NB_VERTICES],
    pub(crate) my_edges: [Edge; NB_EDGES],
    pub(crate) edges_built: [bool; NB_EDGES],
    pub(crate) my_wires: [Wire; NB_FACES],
    pub(crate) wires_built: [bool; NB_FACES],
    pub(crate) my_faces: [Face; NB_FACES],
    pub(crate) faces_built: [bool; NB_FACES],
    pub(crate) my_infinite: [bool; NB_FACES],
}

impl BRepPrimGWedge {
    /// `BRepPrim_GWedge()`: every limit 0 in the default frame, nothing built;
    /// `BRepPrimGWedge::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepPrim_GWedge(B, Axes, dx, dy, dz)`: the box 0..dx, 0..dy, 0..dz;
    /// `BRepPrimGWedge::with_box(builder, axes, 1.0, 2.0, 3.0)`.
    pub fn with_box(builder: BRepPrimBuilder, axes: Axis2, dx: f64, dy: f64, dz: f64) -> Self {
        Self::with_limits(
            builder,
            axes,
            [0.0, 0.0, 0.0, 0.0, 0.0],
            [dx, dy, dz, dz, dx],
        )
    }

    /// `BRepPrim_GWedge(B, Axes, dx, dy, dz, ltx)`: the right angular wedge whose YMax
    /// face spans 0..ltx in X; `BRepPrimGWedge::with_ltx(builder, axes, 1.0, 2.0, 3.0, 0.5)`.
    pub fn with_ltx(
        builder: BRepPrimBuilder,
        axes: Axis2,
        dx: f64,
        dy: f64,
        dz: f64,
        ltx: f64,
    ) -> Self {
        Self::with_limits(
            builder,
            axes,
            [0.0, 0.0, 0.0, 0.0, 0.0],
            [dx, dy, dz, dz, ltx],
        )
    }

    /// `BRepPrim_GWedge(B, Axes, xmin, ymin, zmin, z2min, x2min, xmax, ymax, zmax, z2max,
    /// x2max)`: every limit given, the minima as `[xmin, ymin, zmin, z2min, x2min]` and
    /// the maxima as `[xmax, ymax, zmax, z2max, x2max]`;
    /// `BRepPrimGWedge::with_limits(builder, axes, [0.0; 5], [1.0; 5])`.
    pub fn with_limits(
        builder: BRepPrimBuilder,
        axes: Axis2,
        mins: [f64; 5],
        maxs: [f64; 5],
    ) -> Self {
        let [xmin, ymin, zmin, z2min, x2min] = mins;
        let [xmax, ymax, zmax, z2max, x2max] = maxs;
        Self {
            my_builder: builder,
            my_axes: axes,
            xmin,
            xmax,
            ymin,
            ymax,
            zmin,
            zmax,
            z2min,
            z2max,
            x2min,
            x2max,
            ..Self::default()
        }
    }

    /// `Axes()`; `wedge.axes()`.
    pub fn axes(&self) -> Axis2 {
        self.my_axes
    }

    /// `GetXMin()`; `wedge.get_xmin()`.
    pub fn get_xmin(&self) -> f64 {
        self.xmin
    }

    /// `GetYMin()`; `wedge.get_ymin()`.
    pub fn get_ymin(&self) -> f64 {
        self.ymin
    }

    /// `GetZMin()`; `wedge.get_zmin()`.
    pub fn get_zmin(&self) -> f64 {
        self.zmin
    }

    /// `GetZ2Min()`; `wedge.get_z2min()`.
    pub fn get_z2min(&self) -> f64 {
        self.z2min
    }

    /// `GetX2Min()`; `wedge.get_x2min()`.
    pub fn get_x2min(&self) -> f64 {
        self.x2min
    }

    /// `GetXMax()`; `wedge.get_xmax()`.
    pub fn get_xmax(&self) -> f64 {
        self.xmax
    }

    /// `GetYMax()`; `wedge.get_ymax()`.
    pub fn get_ymax(&self) -> f64 {
        self.ymax
    }

    /// `GetZMax()`; `wedge.get_zmax()`.
    pub fn get_zmax(&self) -> f64 {
        self.zmax
    }

    /// `GetZ2Max()`; `wedge.get_z2max()`.
    pub fn get_z2max(&self) -> f64 {
        self.z2max
    }

    /// `GetX2Max()`; `wedge.get_x2max()`.
    pub fn get_x2max(&self) -> f64 {
        self.x2max
    }

    /// `BRepPrim_Wedge_Check`: Standard_DomainError once a vertex, an edge, a wire or a
    /// face is built (the shell flag is not checked, as natively).
    fn check_nothing_built(&self, member: &str) -> Result<(), Failure> {
        let built = self.vertices_built.iter().any(|&b| b)
            || self.edges_built.iter().any(|&b| b)
            || self.wires_built.iter().any(|&b| b)
            || self.faces_built.iter().any(|&b| b);
        if built {
            return Err(wedge_domain_error(
                member,
                "sub-shapes are already built; expected an unbuilt wedge",
            ));
        }
        Ok(())
    }

    /// `Open(d1)`: the face in direction `d1` and its edges and vertices do not exist;
    /// Standard_DomainError once something is built; `wedge.open(BRepPrimDirection::ZMax)?`.
    pub fn open(&mut self, d1: BRepPrimDirection) -> Result<(), Failure> {
        self.check_nothing_built("Open")?;
        self.my_infinite[num_dir1(d1)] = true;
        Ok(())
    }

    /// `Close(d1)`: the face in direction `d1` exists again; Standard_DomainError once
    /// something is built; `wedge.close(BRepPrimDirection::ZMax)?`.
    pub fn close(&mut self, d1: BRepPrimDirection) -> Result<(), Failure> {
        self.check_nothing_built("Close")?;
        self.my_infinite[num_dir1(d1)] = false;
        Ok(())
    }

    /// `IsInfinite(d1)`: whether the wedge is open in direction `d1`;
    /// `wedge.is_infinite(BRepPrimDirection::XMin)`.
    pub fn is_infinite(&self, d1: BRepPrimDirection) -> bool {
        self.my_infinite[num_dir1(d1)]
    }

    /// `Shell()`: the shell of the existing faces, built once; Standard_DomainError for
    /// a degenerated wedge; `wedge.shell()?`.
    pub fn shell(&mut self) -> Result<Shell, Failure> {
        if self.is_degenerated_shape() {
            return Err(wedge_domain_error(
                "Shell",
                &format!(
                    "degenerated wedge (X {}..{}, Y {}..{}, Z {}..{}, X2 {}..{}, Z2 {}..{}); \
                     expected positive extents",
                    self.xmin,
                    self.xmax,
                    self.ymin,
                    self.ymax,
                    self.zmin,
                    self.zmax,
                    self.x2min,
                    self.x2max,
                    self.z2min,
                    self.z2max
                ),
            ));
        }
        if !self.shell_built {
            self.build_shell()?;
        }
        Ok(self.my_shell.clone())
    }

    /// The body of `Shell()` once the wedge is checked: the faces in direction order.
    fn build_shell(&mut self) -> Result<(), Failure> {
        let mut shell = Shell::default();
        self.my_builder.make_shell(&mut shell)?;
        for d1 in BRepPrimDirection::ALL {
            if self.has_face(d1) {
                let face = self.face(d1)?;
                self.my_builder.add_shell_face(&mut shell, &face)?;
            }
        }
        shell.set_closed(BRepTool::is_closed(&shell)?)?;
        self.my_builder.complete_shell(&shell)?;
        self.my_shell = shell;
        self.shell_built = true;
        Ok(())
    }

    /// `HasFace(d1)`: the face exists when its direction is not open; the YMax face also
    /// needs a non-null extent in X2 and Z2; `wedge.has_face(BRepPrimDirection::YMax)`.
    pub fn has_face(&self, d1: BRepPrimDirection) -> bool {
        let mut state = !self.my_infinite[num_dir1(d1)];
        if d1 == BRepPrimDirection::YMax {
            state = state && (self.z2max != self.z2min) && (self.x2max != self.x2min);
        }
        state
    }

    /// `HasEdge(d1, d2)`: the edge exists when neither direction is open; the edges of
    /// the YMax face also need a non-null extent (in X2 for 6 and 7, in Z2 for 1 and 3);
    /// Standard_DomainError for two directions of the same axis;
    /// `wedge.has_edge(BRepPrimDirection::XMin, BRepPrimDirection::YMin)?`.
    pub fn has_edge(&self, d1: BRepPrimDirection, d2: BRepPrimDirection) -> Result<bool, Failure> {
        let mut state = !(self.my_infinite[num_dir1(d1)] || self.my_infinite[num_dir1(d2)]);
        let i = num_dir2(d1, d2)?;
        if i == 6 || i == 7 {
            state = state && (self.x2max != self.x2min);
        } else if i == 1 || i == 3 {
            state = state && (self.z2max != self.z2min);
        }
        Ok(state)
    }

    /// `HasVertex(d1, d2, d3)`: the vertex exists when no direction is open;
    /// `wedge.has_vertex(XMin, YMin, ZMin)`.
    pub fn has_vertex(
        &self,
        d1: BRepPrimDirection,
        d2: BRepPrimDirection,
        d3: BRepPrimDirection,
    ) -> bool {
        !(self.my_infinite[num_dir1(d1)]
            || self.my_infinite[num_dir1(d2)]
            || self.my_infinite[num_dir1(d3)])
    }

    /// `IsDegeneratedShape()`: a null or negative extent in X, Y or Z (within
    /// Precision::Confusion()) or a negative extent in X2 or Z2;
    /// `wedge.is_degenerated_shape()`.
    pub fn is_degenerated_shape(&self) -> bool {
        (self.xmax - self.xmin <= precision::confusion())
            || (self.ymax - self.ymin <= precision::confusion())
            || (self.zmax - self.zmin <= precision::confusion())
            || (self.z2max - self.z2min < 0.0)
            || (self.x2max - self.x2min < 0.0)
    }
}
