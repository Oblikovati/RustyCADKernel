// Rust translation of BRepLib_MakeShell::Init (BRepLib_MakeShell.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The grid of faces of `BRepLib_MakeShell::Init`, built row by row with the edges and
//! vertices of the previous row, then the 3d curves, regularities and degenerated flags.
use crate::brep_lib::build_curve3d::build_curves3d;
use crate::brep_lib::degenerated_curve::degenerated_tolerance;
use crate::brep_lib::encode_regularity::{ENCODE_REGULARITY_TOL_ANG, encode_regularity};
use crate::brep_lib::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Shell, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::{Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Point2, StandardDirection2};

/// The parameters, isolines and flags of the C2 grid of the surface.
struct Grid {
    surface: Handle<dyn GeomSurface>,
    basis: Handle<dyn GeomSurface>,
    segment_bspline: bool,
    upars: Vec<f64>,
    vpars: Vec<f64>,
    uisos: Vec<Handle<dyn Geom2dCurve>>,
    visos: Vec<Handle<dyn Geom2dCurve>>,
    uperiodic: bool,
    vperiodic: bool,
}

/// The C2 interval bounds of one direction and their isolines (null at infinite bounds).
fn isolines(bounds: &[f64], u_direction: bool) -> Vec<Handle<dyn Geom2dCurve>> {
    bounds
        .iter()
        .map(|&t| {
            if precision::is_infinite(t) {
                return Handle::default();
            }
            let line = if u_direction {
                Geom2dLine::from_point(
                    Point2::new(t, 0.),
                    Direction2::from_standard(StandardDirection2::Y),
                )
            } else {
                Geom2dLine::from_point(
                    Point2::new(0., t),
                    Direction2::from_standard(StandardDirection2::X),
                )
            };
            Handle::new(line).cast()
        })
        .collect()
}

/// The interval bounds of one direction (`nb + 1` values).
fn interval_bounds(
    adaptor: &GeomAdaptorSurface,
    nb: i32,
    u_direction: bool,
) -> Result<Vec<f64>, Failure> {
    let mut pars = Array1::<f64>::with_bounds(1, nb + 1)
        .map_err(|e| e.into_failure("BRepLib_MakeShell::Init"))?;
    if u_direction {
        adaptor.u_intervals(&mut pars, Continuity::C2)?;
    } else {
        adaptor.v_intervals(&mut pars, Continuity::C2)?;
    }
    Ok(pars.as_slice().to_vec())
}

impl Grid {
    /// The grid, or None when a direction has no C2 interval.
    fn new(
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<Option<Self>, Failure> {
        let basis: Handle<dyn GeomSurface> =
            match surface.cast::<GeomRectangularTrimmedSurface>().get() {
                Some(trimmed) => trimmed.basis_surface(),
                None => surface.clone(),
            };
        let adaptor = GeomAdaptorSurface::with_bounds(&basis, bounds)?;
        let (nu, nv) = (
            adaptor.nb_u_intervals(Continuity::C2)?,
            adaptor.nb_v_intervals(Continuity::C2)?,
        );
        let (uperiodic, vperiodic) = (adaptor.is_u_periodic()?, adaptor.is_v_periodic()?);
        if nu == 0 || nv == 0 {
            return Ok(None);
        }
        let upars = interval_bounds(&adaptor, nu, true)?;
        let uisos = isolines(&upars, true);
        let vpars = interval_bounds(&adaptor, nv, false)?;
        let visos = isolines(&vpars, false);
        let segment_bspline: bool = adaptor.get_type()? == SurfaceType::BSplineSurface && segment;
        Ok(Some(Self {
            surface: surface.clone(),
            basis,
            segment_bspline,
            upars,
            vpars,
            uisos,
            visos,
            uperiodic,
            vperiodic,
        }))
    }

    fn nu(&self) -> usize {
        self.upars.len() - 1
    }

    fn nv(&self) -> usize {
        self.vpars.len() - 1
    }

    /// `upars(i)` with the native 1-based index.
    fn u(&self, i: usize) -> f64 {
        self.upars[i - 1]
    }

    /// `vpars(i)` with the native 1-based index.
    fn v(&self, i: usize) -> f64 {
        self.vpars[i - 1]
    }

    /// A vertex at `S->Value(u, v)` (the surface itself, not its basis) of tolerance
    /// Precision::Confusion(), into `vertex`.
    fn make_vertex(&self, vertex: &mut Vertex, u: f64, v: f64) -> Result<(), Failure> {
        let s = self
            .surface
            .get()
            .ok_or_else(|| null_object("BRepLib_MakeShell: null surface"))?;
        BRepBuilder::default().make_vertex_at(vertex, s.value(u, v)?, precision::confusion())
    }

    /// A vertex at the grid point `(iu, iv)` when both parameters are finite, else null.
    fn grid_vertex(&self, iu: usize, iv: usize) -> Result<Vertex, Failure> {
        let mut vertex = Vertex::default();
        if !precision::is_infinite(self.v(iv)) && !precision::is_infinite(self.u(iu)) {
            self.make_vertex(&mut vertex, self.u(iu), self.v(iv))?;
        }
        Ok(vertex)
    }

    /// The face of patch `(iu, iv)` on a copy of the basis surface (segmented to the
    /// patch for a B-spline when asked).
    fn patch_face(&self, iu: usize, iv: usize) -> Result<Face, Failure> {
        let basis = self
            .basis
            .get()
            .ok_or_else(|| null_object("BRepLib_MakeShell: null surface"))?;
        let copy: Handle<dyn GeomSurface> = basis.copy()?.cast();
        if self.segment_bspline
            && let Some(bspline) = copy.cast::<GeomBSplineSurface>().get()
        {
            let tolerances = (precision::p_confusion(), precision::p_confusion());
            bspline.segment(
                (self.u(iu), self.u(iu + 1)),
                (self.v(iv), self.v(iv + 1)),
                tolerances,
            )?;
        }
        let mut face = Face::default();
        BRepBuilder::default().make_face_on_surface(&mut face, &copy, precision::confusion())?;
        Ok(face)
    }
}

/// A new edge between two vertices (FORWARD then REVERSED, each when not null); the
/// orientations are set on the variables themselves, as natively.
fn edge_between(first: &mut Vertex, last: &mut Vertex) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    if !first.is_null() {
        first.set_orientation(Orientation::Forward);
        builder.add(&mut edge, first)?;
    }
    if !last.is_null() {
        last.set_orientation(Orientation::Reversed);
        builder.add(&mut edge, last)?;
    }
    Ok(edge)
}

/// The state carried from row to row: the edges and vertices of the bottom of the next
/// row, and copies of the first row for the periodic closure.
struct Rows {
    botedges: Vec<Edge>,
    botvertices: Vec<Vertex>,
    fbotedges: Vec<Edge>,
    fbotvertices: Vec<Vertex>,
}

/// `BRepLib_MakeShell::Init` up to the faces: `shape` becomes the new shell before the
/// faces are added (it is left unchanged when a direction has no C2 interval, and the
/// function returns false).
pub(super) fn shell_of_patches(
    shape: &mut Shape,
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
    segment: bool,
) -> Result<bool, Failure> {
    let Some(grid) = Grid::new(surface, bounds, segment)? else {
        return Ok(false);
    };
    let mut shell = Shell::default();
    BRepBuilder::default().make_shell(&mut shell);
    *shape = shell.into_shape();
    let mut rows = first_row(&grid)?;
    for iv in 1..=grid.nv() {
        row_of_faces(&grid, &mut rows, iv, shape)?;
    }
    Ok(true)
}

/// "init the botedges and botvertices": the edges and vertices at `vpars(1)`.
fn first_row(grid: &Grid) -> Result<Rows, Failure> {
    let nu: usize = grid.nu();
    let mut rows = Rows {
        botedges: vec![Edge::default(); nu],
        botvertices: vec![Vertex::default(); nu + 1],
        fbotedges: vec![Edge::default(); nu],
        fbotvertices: vec![Vertex::default(); nu + 1],
    };
    if precision::is_infinite(grid.v(1)) {
        return Ok(rows);
    }
    let mut vrt = Vertex::default();
    if !precision::is_infinite(grid.u(1)) {
        grid.make_vertex(&mut vrt, grid.u(1), grid.v(1))?;
    }
    rows.botvertices[0] = vrt.clone();
    rows.fbotvertices[0] = vrt.clone();
    for iu in 1..=nu {
        let mut vlt = vrt.clone();
        if grid.uperiodic && iu == nu {
            vrt = rows.botvertices[0].clone();
        } else if !precision::is_infinite(grid.u(iu + 1)) {
            grid.make_vertex(&mut vrt, grid.u(iu + 1), grid.v(1))?;
        }
        rows.botvertices[iu] = vrt.clone();
        rows.fbotvertices[iu] = vrt.clone();
        let etop = edge_between(&mut vlt, &mut vrt)?;
        rows.botedges[iu - 1] = etop.clone();
        rows.fbotedges[iu - 1] = etop;
    }
    Ok(rows)
}

/// The mutable variables of the native row loop.
#[derive(Default)]
struct Cursor {
    vrb: Vertex,
    vrt: Vertex,
    vlt: Vertex,
    fvlt: Vertex,
    eleft: Edge,
    eright: Edge,
    feleft: Edge,
}

/// The row `iv` of faces, added to the shell.
fn row_of_faces(grid: &Grid, rows: &mut Rows, iv: usize, shell: &mut Shape) -> Result<(), Failure> {
    let mut c = Cursor {
        vrb: rows.botvertices[0].clone(),
        ..Cursor::default()
    };
    // compute the first edge and vertices of the line
    c.vrt = if grid.vperiodic && iv == grid.nv() {
        rows.fbotvertices[0].clone()
    } else {
        grid.grid_vertex(1, iv + 1)?
    };
    if !precision::is_infinite(grid.u(1)) {
        c.eright = edge_between(&mut c.vrb, &mut c.vrt)?;
    }
    c.fvlt = c.vrt.clone();
    c.feleft = c.eright.clone();
    for iu in 1..=grid.nu() {
        let face = grid.patch_face(iu, iv)?;
        side_edges(grid, rows, &mut c, (iu, iv))?;
        update_side_pcurves(grid, &c, (iu, iv), &face)?;
        let (ebot, etop) = bottom_top_edges(grid, rows, &mut c, (iu, iv), &face)?;
        rows.botedges[iu - 1] = etop.clone();
        let wire = patch_wire(&mut c.eleft, ebot, &mut c.eright, etop)?;
        let builder = BRepBuilder::default();
        let mut face = face;
        builder.add(&mut face, &wire)?;
        builder.add(shell, &face)?;
    }
    Ok(())
}

/// The vertices of the patch and its left and right edges (the right one shared with
/// the next patch, or the first left edge for the periodic closure).
fn side_edges(
    grid: &Grid,
    rows: &mut Rows,
    c: &mut Cursor,
    (iu, iv): (usize, usize),
) -> Result<(), Failure> {
    // the vertices
    c.vrb = rows.botvertices[iu].clone();
    c.vlt = c.vrt.clone();
    c.vrt = if grid.uperiodic && iu == grid.nu() {
        c.fvlt.clone()
    } else {
        grid.grid_vertex(iu + 1, iv + 1)?
    };
    rows.botvertices[iu - 1] = c.vlt.clone();
    rows.botvertices[iu] = c.vrt.clone();
    // the edges
    c.eleft = c.eright.clone();
    if grid.uperiodic && iu == grid.nu() {
        c.eright = c.feleft.clone();
    } else {
        c.eright = Edge::default();
        if !precision::is_infinite(grid.u(iu + 1)) {
            c.eright = edge_between(&mut c.vrb, &mut c.vrt)?;
        }
    }
    Ok(())
}

/// The u-isolines of the left and right edges on the face.
fn update_side_pcurves(
    grid: &Grid,
    c: &Cursor,
    (iu, iv): (usize, usize),
    face: &Face,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let tol: f64 = precision::confusion();
    let range = (grid.v(iv), grid.v(iv + 1));
    if grid.uperiodic && grid.nu() == 1 {
        if !c.eleft.is_null() && !c.eright.is_null() {
            builder.update_edge_pcurves_on_face(
                &c.eleft,
                (&grid.uisos[1], &grid.uisos[0]),
                face,
                tol,
            )?;
            builder.range_on_face(&c.eleft, face, range.0, range.1)?;
        }
        return Ok(());
    }
    if !c.eleft.is_null() {
        builder.update_edge_pcurve_on_face(&c.eleft, &grid.uisos[iu - 1], face, tol)?;
        builder.range_on_face(&c.eleft, face, range.0, range.1)?;
    }
    if !c.eright.is_null() {
        builder.update_edge_pcurve_on_face(&c.eright, &grid.uisos[iu], face, tol)?;
        builder.range_on_face(&c.eright, face, range.0, range.1)?;
    }
    Ok(())
}

/// The bottom edge (from the previous row) and the top edge of the patch (the first
/// row's bottom edge for the periodic closure), with their v-isolines on the face. On a
/// V-periodic surface of one interval the seam range is set from the V parameters, as
/// natively.
fn bottom_top_edges(
    grid: &Grid,
    rows: &Rows,
    c: &mut Cursor,
    (iu, iv): (usize, usize),
    face: &Face,
) -> Result<(Edge, Edge), Failure> {
    let ebot: Edge = rows.botedges[iu - 1].clone();
    let etop: Edge = if grid.vperiodic && iv == grid.nv() {
        rows.fbotedges[iu - 1].clone()
    } else if !precision::is_infinite(grid.v(iv + 1)) {
        edge_between(&mut c.vlt, &mut c.vrt)?
    } else {
        Edge::default()
    };
    let builder = BRepBuilder::default();
    let tol: f64 = precision::confusion();
    if grid.vperiodic && grid.nv() == 1 {
        if !ebot.is_null() && !etop.is_null() {
            builder.update_edge_pcurves_on_face(
                &ebot,
                (&grid.visos[0], &grid.visos[1]),
                face,
                tol,
            )?;
            builder.range_on_face(&ebot, face, grid.v(iv), grid.v(iv + 1))?;
        }
        return Ok((ebot, etop));
    }
    let range = (grid.u(iu), grid.u(iu + 1));
    if !ebot.is_null() {
        builder.update_edge_pcurve_on_face(&ebot, &grid.visos[iv - 1], face, tol)?;
        builder.range_on_face(&ebot, face, range.0, range.1)?;
    }
    if !etop.is_null() {
        builder.update_edge_pcurve_on_face(&etop, &grid.visos[iv], face, tol)?;
        builder.range_on_face(&etop, face, range.0, range.1)?;
    }
    Ok((ebot, etop))
}

/// The wire of the patch: left REVERSED, bottom FORWARD, right FORWARD, top REVERSED
/// (the orientations are set on the left and right variables, as natively).
fn patch_wire(
    eleft: &mut Edge,
    mut ebot: Edge,
    eright: &mut Edge,
    mut etop: Edge,
) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for (edge, orientation) in [
        (eleft, Orientation::Reversed),
        (&mut ebot, Orientation::Forward),
        (eright, Orientation::Forward),
        (&mut etop, Orientation::Reversed),
    ] {
        if !edge.is_null() {
            edge.set_orientation(orientation);
            builder.add(&mut wire, edge)?;
        }
    }
    Ok(wire)
}

/// "codage des courbes 3d et regularites": the 3d curves, the regularities, the Closed
/// flag, then every edge flagged degenerated when its 3d curve is a point (a null 3d
/// curve raises Standard_NullObject, as natively).
pub(super) fn finish_shell(shell: &Shape) -> Result<(), Failure> {
    let tol: f64 = precision::confusion();
    build_curves3d(shell, tol, Continuity::C1, 14, 0)?;
    encode_regularity(shell, ENCODE_REGULARITY_TOL_ANG)?;
    shell.set_closed(BRepTool::is_closed(shell)?)?;
    // Additional checking for degenerated edges
    let builder = BRepBuilder::default();
    let mut explorer = Explorer::new(shell, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let edge = Edge::unchecked(explorer.current().clone());
        let (curve, _, _) = BRepTool::curve(&edge)?;
        let degenerated: bool = degenerated_tolerance(&curve, tol)?.is_some();
        builder.degenerated(&edge, degenerated)?;
        explorer.next()?;
    }
    Ok(())
}
