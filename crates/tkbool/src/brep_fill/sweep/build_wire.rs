// Rust translation of BRepFill_Sweep::BuildWire
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sweep of a vertex section: a wire of edges on the iso curves of the swept
//! surfaces.
use super::topology::build_vertex;
use super::{BRepFillSweep, null};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, new_table};
use crate::brep_fill::transition_style::TransitionStyle;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkgeomalgo::geom_fill::GeomFillSweep;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The iso curve of a swept surface across the section, with its parameter range.
struct SweptIso {
    iso: Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
}

/// The iso of the start of the section on the surface (a V iso when the parameters are
/// exchanged), on the last bound when the U direction is reversed.
fn swept_iso(
    s: &Handle<dyn GeomSurface>,
    exchange_uv: bool,
    u_reversed: bool,
) -> Result<SweptIso, Failure> {
    let surface = s.get().ok_or_else(|| null("swept surface"))?;
    let (u1, u2, v1, v2) = surface.bounds();
    if exchange_uv {
        let iso = surface.v_iso(if u_reversed { v2 } else { v1 })?;
        return Ok(SweptIso {
            iso,
            first: u1,
            last: u2,
        });
    }
    let iso = surface.u_iso(if u_reversed { u2 } else { u1 })?;
    Ok(SweptIso {
        iso,
        first: v1,
        last: v2,
    })
}

/// The tolerance of the vertex raised to its distance to the point of the iso.
fn enlarge_to(v: &Shape, iso: &Handle<dyn GeomCurve>, param: f64) -> Result<(), Failure> {
    let p1 = iso.get().ok_or_else(|| null("iso curve"))?.d0(param)?;
    let vertex: Vertex = topods::vertex(v)?;
    let tol: f64 = p1.distance(&BRepTool::pnt(&vertex)?);
    BRepBuilder::default().update_vertex_tolerance(&vertex, tol)
}

impl BRepFillSweep {
    /// `BuildWire(Transition)` (protected): the wire of the edges on the isos of the
    /// surfaces swept by the first law of a vertex section (the faces table holds the
    /// edges); false when a sweep or an edge fails.
    /// `sweep.build_wire(TransitionStyle::Modified)?`.
    pub fn build_wire(&mut self, _transition: TransitionStyle) -> Result<bool, Failure> {
        let nb_path: i32 = self.loc()?.nb_law()?;
        let vclose: bool =
            self.loc()?.is_closed()? && self.loc()?.is_g1(0, self.my_tol3d, 1.0e-4)? >= 0;
        self.error = 0.;
        let builder = BRepBuilder::default();
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        self.my_faces = Some(new_table((1, 1), (1, nb_path))?);
        self.my_u_edges = Some(new_table((1, 2), (1, nb_path))?);
        self.my_v_edges = Some(new_table((1, 1), (1, nb_path + 1))?);
        for ipath in 1..=nb_path {
            let Some((swept, error_on_surface)) = self.swept_section_iso(ipath)? else {
                return Ok(false);
            };
            self.place_path_vertices(ipath, nb_path, vclose, &swept)?;
            let v_edges: &ShapeTable = self.v_edges_mut()?;
            let (vf, vl) = (
                topods::vertex(cell(v_edges, 1, ipath)?)?,
                topods::vertex(cell(v_edges, 1, ipath + 1)?)?,
            );
            let iso = swept.iso.get().ok_or_else(|| null("iso curve"))?;
            let mut mk_e = BRepLibMakeEdge::new();
            mk_e.init_curve_vertices_params(
                &swept.iso,
                &vf,
                &vl,
                iso.first_parameter(),
                iso.last_parameter(),
            )?;
            if !mk_e.is_done() {
                return Ok(false);
            }
            let e: Edge = mk_e.edge()?;
            builder.update_edge_tolerance(&e, error_on_surface)?;
            builder.add(&mut wire, &e)?;
            *cell_mut(self.faces_mut()?, 1, ipath)? = e.into_shape();
        }
        self.my_shape = wire.into_shape();
        Ok(true)
    }

    /// The iso of the surface swept by the first section law along the law `ipath` and
    /// the approximation error; `None` when the sweep fails.
    fn swept_section_iso(&self, ipath: i32) -> Result<Option<(SweptIso, f64)>, Failure> {
        let mut sweep = GeomFillSweep::new(&self.loc()?.law(ipath)?, self.k_part)?;
        sweep.set_tolerance(
            self.my_tol3d,
            self.my_bound_tol,
            self.my_tol2d,
            self.my_tol_angular,
        );
        sweep.set_force_approx_c1(self.my_force_approx_c1);
        sweep.build(
            &self.sec()?.law(1)?,
            self.my_approx_style,
            self.my_continuity,
            self.my_degmax,
            self.my_segmax,
        )?;
        if !sweep.is_done() {
            return Ok(None);
        }
        let swept = swept_iso(&sweep.surface(), sweep.exchange_uv(), sweep.u_reversed())?;
        Ok(Some((swept, sweep.error_on_surface())))
    }

    /// The vertices of the path at the ends of the iso: the next one built (the first
    /// one reused when the path is closed and G1, the first bound put at `NbPath` when
    /// there is a last bound, as natively), the current one enlarged to the iso, and the
    /// first one (the first bound when there is one).
    fn place_path_vertices(
        &mut self,
        ipath: i32,
        nb_path: i32,
        vclose: bool,
        swept: &SweptIso,
    ) -> Result<(), Failure> {
        let (first_shape, last_shape) = (self.first_shape.clone(), self.last_shape.clone());
        let v_edges: &mut ShapeTable = self.v_edges_mut()?;
        if ipath < nb_path {
            *cell_mut(v_edges, 1, ipath + 1)? =
                build_vertex(&swept.iso, false, swept.first, swept.last)?.into_shape();
        } else if vclose {
            let v: Shape = cell(v_edges, 1, 1)?.clone();
            *cell_mut(v_edges, 1, ipath + 1)? = v.clone();
            enlarge_to(&v, &swept.iso, swept.last)?;
        } else if !last_shape.is_null() {
            *cell_mut(v_edges, 1, nb_path)? = first_shape.as_ref().clone();
        } else {
            *cell_mut(v_edges, 1, nb_path + 1)? =
                build_vertex(&swept.iso, false, swept.first, swept.last)?.into_shape();
        }
        if ipath > 1 {
            enlarge_to(cell(v_edges, 1, ipath)?, &swept.iso, swept.first)?;
        }
        if ipath == 1 {
            *cell_mut(v_edges, 1, 1)? = if first_shape.is_null() {
                build_vertex(&swept.iso, true, swept.first, swept.last)?.into_shape()
            } else {
                first_shape.into_shape()
            };
        }
        Ok(())
    }
}
