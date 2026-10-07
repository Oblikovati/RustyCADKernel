// Rust translation of BRepFill_Sweep::PerformCorner, EvalExtrapol and
// RebuildTopOrBottomEdge (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corners of a sweep along a path with sharp vertices: the trimming of the faces
//! met there, the round fillings and the extension lengths of the right corners.
use super::filling::{CornerFrame, FillingInput, filling};
use super::helpers::{section_box, translate};
use super::topology::keep_edge;
use super::{BRepFillSweep, null};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, new_table, to_handle};
use crate::brep_fill::transition_style::TransitionStyle;
use crate::brep_fill::trim_shell_corner::BRepFillTrimShellCorner;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::geom_fill::{GeomFillLocationLaw, LocationD0};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};
use std::f64::consts::PI;

/// The angular tolerance under which two path tangents are opposite (no corner).
const AN_ANGULAR_TOL: f64 = 0.025;

/// The point and normalized tangent of a location law at the last (`at_end`) or first
/// parameter of its domain.
fn law_tangent(law: &dyn GeomFillLocationLaw, at_end: bool) -> Result<(Point3, Vector3), Failure> {
    let (f, l) = law.get_domain()?;
    let curve = law.get_curve();
    let (p, mut t) = curve
        .get()
        .ok_or_else(|| null("path curve"))?
        .d1(if at_end { l } else { f })?;
    t.normalize()?;
    Ok((p, t))
}

/// The third column (the tangent) of the placement of a location law at a parameter.
fn law_column3(law: &dyn GeomFillLocationLaw, param: f64) -> Result<Vector3, Failure> {
    let mut placement = LocationD0::default();
    law.d0(param, &mut placement)?;
    Ok(Vector3::from(placement.m.column(3)?))
}

/// The paths before and after the vertex `index` of the path (the last and the first
/// at the first vertex).
fn corner_laws(index: i32, nb_law: i32) -> (i32, i32) {
    if index > 1 {
        (index - 1, index)
    } else {
        (nb_law, 1)
    }
}

/// The frames of a corner: the axis of the round filling, the bisector plane axis and
/// the cross direction of the path tangents.
struct CornerAxes {
    axe: Axis2,
    axe_of_bis_plane: Axis2,
    cross_direction: Vector3,
    t1: Vector3,
}

/// The kind of a vertex of the path.
enum CornerKind {
    /// Tangent paths with tangent placements.
    NotACorner,
    /// The paths turn back (opposite tangents).
    TurnsBack,
    /// A corner with its axes and transition.
    Corner(Box<CornerAxes>, TransitionStyle),
}

impl BRepFillSweep {
    /// `PerformCorner(Index, Transition, Bounds)` (protected): the faces of the two
    /// paths at the vertex `index` trimmed by each other (right and round corners) and
    /// the round fillings; false when the paths turn back.
    /// `sweep.perform_corner(2, TransitionStyle::Round, &bounds)?`.
    pub fn perform_corner(
        &mut self,
        index: i32,
        transition: TransitionStyle,
        bounds: &ShapeTable,
    ) -> Result<bool, Failure> {
        if transition == TransitionStyle::Modified {
            return Ok(true);
        }
        let (i1, i2) = corner_laws(index, self.loc()?.nb_law()?);
        match self.corner_axes((i1, i2), transition)? {
            CornerKind::NotACorner => Ok(true),
            CornerKind::TurnsBack => Ok(false),
            CornerKind::Corner(axes, the_transition) => {
                self.trim_corner(index, (i1, i2), transition, the_transition, &axes, bounds)
            }
        }
    }

    /// The axes of the corner and the transition used (a right corner too sharp is
    /// round), or whether the corner is not one (tangent paths with tangent
    /// placements) or the paths turn back.
    fn corner_axes(
        &self,
        (i1, i2): (i32, i32),
        transition: TransitionStyle,
    ) -> Result<CornerKind, Failure> {
        let (law1, law2) = (self.loc()?.law(i1)?, self.loc()?.law(i2)?);
        let (l1, l2) = (
            law1.get().ok_or_else(|| null("location law"))?,
            law2.get().ok_or_else(|| null("location law"))?,
        );
        let (mut p1, t1) = law_tangent(l1, true)?;
        let (p2, t2) = law_tangent(l2, false)?;
        let mut sortant = Vector3::default();
        let is_tangent: bool = t1.angle(t2)? < self.my_ang_min;
        if is_tangent {
            let t1c: Vector3 = law_column3(l1, l1.get_domain()?.1)?;
            // The native code reads the second placement at the last parameter too.
            let t2c: Vector3 = law_column3(l2, l2.get_domain()?.1)?;
            if t1c.angle(t2c)? < self.my_ang_min {
                return Ok(CornerKind::NotACorner);
            }
            sortant = t2c - t1c;
        }
        if t1.angle(t2)? >= PI - AN_ANGULAR_TOL {
            return Ok(CornerKind::TurnsBack);
        }
        let the_transition =
            if transition == TransitionStyle::Right && t1.angle(t2)? > self.my_ang_max {
                TransitionStyle::Round
            } else {
                transition
            };
        let mut tang: Vector3 = t1 + t2;
        let normal_of_bis_plane = Direction3::from_vector(tang)?;
        let cross_direction: Vector3 = t1.crossed(t2);
        if is_tangent {
            sortant -= tang.dot(tang) * tang;
        } else {
            sortant = t2 - t1;
            sortant *= -1.;
            tang -= tang.dot(t2) * t2;
        }
        p1.barycenter(0.5, p2, 0.5);
        let axe = Axis2::with_x_direction(
            p1,
            Direction3::from_vector(sortant)?,
            Direction3::from_vector(tang)?,
        )?;
        let axe_of_bis_plane = Axis2::new(p1, normal_of_bis_plane)?;
        Ok(CornerKind::Corner(
            Box::new(CornerAxes {
                axe,
                axe_of_bis_plane,
                cross_direction,
                t1,
            }),
            the_transition,
        ))
    }

    /// The faces of the two paths trimmed at the corner and their edges replaced by the
    /// trimmed ones (the v-edges of the corner by their first new shape, null when they
    /// vanish), then the round fillings; nothing is changed when the trimming of a
    /// right corner fails.
    fn trim_corner(
        &mut self,
        index: i32,
        (i1, i2): (i32, i32),
        transition: TransitionStyle,
        the_transition: TransitionStyle,
        axes: &CornerAxes,
        bounds: &ShapeTable,
    ) -> Result<bool, Failure> {
        // The native copy of the u-edge table and the plane face at the corner are never
        // read; they are not built here.
        let faces: &ShapeTable = self.my_faces.as_ref().ok_or_else(|| null("face table"))?;
        let mut a_faces: ShapeTable = new_table((faces.lower_row(), faces.upper_row()), (1, 2))?;
        translate(faces, i1, &mut a_faces, 1)?;
        translate(faces, i2, &mut a_faces, 2)?;
        let u_edges: &ShapeTable = self
            .my_u_edges
            .as_ref()
            .ok_or_else(|| null("u-edge table"))?;
        let mut a_u_edges: ShapeTable =
            new_table((u_edges.lower_row(), u_edges.upper_row()), (1, 2))?;
        translate(u_edges, i1, &mut a_u_edges, 1)?;
        translate(u_edges, i2, &mut a_u_edges, 2)?;
        let v_edges: &ShapeTable = self
            .my_v_edges
            .as_ref()
            .ok_or_else(|| null("v-edge table"))?;
        let mut a_trim = BRepFillTrimShellCorner::new(
            &to_handle(&a_faces),
            transition,
            &axes.axe_of_bis_plane,
            &axes.cross_direction,
        )?;
        a_trim.add_bounds(&to_handle(bounds))?;
        a_trim.add_u_edges(&to_handle(&a_u_edges))?;
        a_trim.add_v_edges(&to_handle(v_edges), index)?;
        a_trim.perform()?;
        if a_trim.is_done() {
            self.take_trimmed(&a_trim, index, (i1, i2))?;
        } else if the_transition == TransitionStyle::Right || a_trim.has_section() {
            return Ok(true);
        }
        let nb_law: i32 = self.sec()?.nb_law()?;
        if self.sec()?.is_u_closed() {
            let u_edges: &mut ShapeTable = self.u_edges_mut()?;
            for i in [i1, i2] {
                *cell_mut(u_edges, 1, i)? = cell(u_edges, nb_law + 1, i)?.clone();
            }
        }
        if the_transition == TransitionStyle::Round {
            self.round_corner((i1, i2), axes, bounds)?;
        }
        Ok(true)
    }

    /// The edges and faces of the corner replaced by their trimmed shapes.
    fn take_trimmed(
        &mut self,
        a_trim: &BRepFillTrimShellCorner,
        index: i32,
        (i1, i2): (i32, i32),
    ) -> Result<(), Failure> {
        let nb_law: i32 = self.sec()?.nb_law()?;
        let v_edges: &mut ShapeTable = self.v_edges_mut()?;
        for ii in 1..=nb_law {
            let listmodif: List<Shape> = a_trim.modified(cell(v_edges, ii, index)?);
            *cell_mut(v_edges, ii, index)? = listmodif
                .first()
                .cloned()
                .unwrap_or_else(|| Edge::default().into_shape());
        }
        for column in [i1, i2] {
            let faces: &mut ShapeTable = self.faces_mut()?;
            for ii in 1..=nb_law {
                replace_by_first(a_trim, faces, ii, column)?;
            }
            let u_edges: &mut ShapeTable = self.u_edges_mut()?;
            for ii in u_edges.lower_row()..=u_edges.upper_row() {
                replace_by_first(a_trim, u_edges, ii, column)?;
            }
        }
        Ok(())
    }

    /// The round fillings between the free borders of the faces of the two paths
    /// (section by section, with the borders of the previous filling as transversal
    /// constraints), added to the auxiliary shapes, the v-edges and the tapes.
    fn round_corner(
        &mut self,
        (i1, i2): (i32, i32),
        axes: &CornerAxes,
        bounds: &ShapeTable,
    ) -> Result<(), Failure> {
        let frame = CornerFrame {
            axe: axes.axe,
            tangent_on_part1: axes.t1,
        };
        let (mut bord1, mut bord2, mut bord_first) =
            (Edge::default(), Edge::default(), Edge::default());
        let mut has_filling: bool = false;
        for ii in 1..=self.sec()?.nb_law()? {
            let faces: &ShapeTable = self.my_faces.as_ref().ok_or_else(|| null("face table"))?;
            let (f1, f2) = (cell(faces, ii, i1)?.clone(), cell(faces, ii, i2)?.clone());
            let list1: Vec<Shape> = keep_edge(&f1, cell(bounds, ii, 1)?)?;
            let list2: Vec<Shape> = keep_edge(&f2, cell(bounds, ii, 2)?)?;
            if list1.len() != list2.len() {
                continue;
            }
            for (e1, e2) in list1.iter().zip(&list2) {
                if has_filling {
                    (bord1, bord2) = transversal_borders(e1, [&bord1, &bord2, &bord_first])?;
                }
                let input = FillingInput {
                    ef: e1,
                    f1: &f1,
                    el: e2,
                    f2: &f2,
                    tol: self.my_tol3d,
                };
                let mut ff = Face::default();
                if filling(
                    &input,
                    &mut self.my_v_edges_modified,
                    &frame,
                    (&mut bord1, &mut bord2),
                    &mut ff,
                )? {
                    self.add_filling(ii, i2, ff.into_shape())?;
                    has_filling = true;
                }
                if ii == 1 {
                    bord_first = bord1.clone();
                }
            }
        }
        Ok(())
    }

    /// A filling face added to the auxiliary shapes, to the compound of the v-edge
    /// `(ii, i2)` and to the tape `ii`.
    fn add_filling(&mut self, ii: i32, i2: i32, ff: Shape) -> Result<(), Failure> {
        self.my_aux_shape.append(ff.clone());
        let builder = BRepBuilder::default();
        let v_edges: &mut ShapeTable = self.v_edges_mut()?;
        let a_vshape: Shape = cell(v_edges, ii, i2)?.clone();
        let mut a_compound = Compound::default();
        builder.make_compound(&mut a_compound);
        if !a_vshape.is_null() {
            builder.add(&mut a_compound, &a_vshape)?;
        }
        builder.add(&mut a_compound, &ff)?;
        *cell_mut(v_edges, ii, i2)? = a_compound.into_shape();
        let tapes = self.my_tapes.as_mut().ok_or_else(|| null("tape array"))?;
        let len: usize = tapes.len();
        let tape = usize::try_from(ii - 1)
            .ok()
            .and_then(|i| tapes.get_mut(i))
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!("BRepFill_Sweep::PerformCorner: tape {ii}; expected 1..={len}"),
                    "NCollection_Array1",
                )
            })?;
        builder.add(tape, &ff)
    }

    /// `EvalExtrapol(Index, Transition)` (protected): the length by which the surfaces
    /// are extended at the vertex `index` of the path for a right corner (0 at the ends
    /// of an open sweep and for too small or too large angles).
    /// `sweep.eval_extrapol(2, TransitionStyle::Right)?`.
    pub fn eval_extrapol(&self, index: i32, transition: TransitionStyle) -> Result<f64, Failure> {
        if transition != TransitionStyle::Right {
            return Ok(0.);
        }
        let nb_law: i32 = self.loc()?.nb_law()?;
        if (index == 1 || index == nb_law + 1)
            && (!self.loc()?.is_closed()? || !self.sec()?.is_v_closed())
        {
            return Ok(0.);
        }
        let (i1, i2) = if index == 1 || index == nb_law + 1 {
            (nb_law, 1)
        } else {
            (index - 1, index)
        };
        let (law1, law2) = (self.loc()?.law(i1)?, self.loc()?.law(i2)?);
        let (l1, l2) = (
            law1.get().ok_or_else(|| null("location law"))?,
            law2.get().ok_or_else(|| null("location law"))?,
        );
        let t1: Vector3 = law_column3(l1, l1.get_domain()?.1)?;
        let t2: Vector3 = law_column3(l2, l2.get_domain()?.0)?;
        let alpha: f64 = t1.angle(t2)?;
        if alpha > self.my_ang_max || alpha < self.my_ang_min {
            return Ok(0.);
        }
        let sec = self.sec()?.concatened_law()?;
        let (_, length) = self.loc()?.curvilinear_bounds(nb_law)?;
        let (sec_first, sec_last) = self
            .sec()?
            .law(1)?
            .get()
            .ok_or_else(|| null("section law"))?
            .get_domain()?;
        let (_, ll) = self.loc()?.curvilinear_bounds(i1)?;
        let u: f64 = sec_first + (ll / length) * (sec_last - sec_first);
        let [xmin, ymin, zmin, xmax, ymax, zmax] = section_box(&sec, u)?.get()?;
        let r: f64 = xmin.abs().max(xmax.abs()).max(ymin.abs().max(ymax.abs())) * 2.;
        let extrap: f64 = zmin.abs().max(zmax.abs()) + 100. * self.my_tol3d;
        Ok(extrap + r * (alpha / 2.).tan())
    }
}

/// The shape of a table replaced by the first of its trimmed shapes, if any.
fn replace_by_first(
    a_trim: &BRepFillTrimShellCorner,
    table: &mut ShapeTable,
    row: i32,
    col: i32,
) -> Result<(), Failure> {
    let listmodif: List<Shape> = a_trim.modified(cell(table, row, col)?);
    if let Some(first) = listmodif.first() {
        *cell_mut(table, row, col)? = first.clone();
    }
    Ok(())
}

/// The borders of the previous fillings sharing a vertex with the edge: the one at its
/// first vertex, the one at its other vertex (null when none; later ones win).
fn transversal_borders(e: &Shape, borders: [&Edge; 3]) -> Result<(Edge, Edge), Failure> {
    let edge: Edge = topods::edge(e)?;
    let (vf, _) = topexp::vertices(&edge, false);
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    for bord in borders {
        if bord.is_null() {
            continue;
        }
        if let Some(vc) = topexp::common_vertex(&edge, bord) {
            if vc.is_same(&vf) {
                e1 = bord.clone();
            } else {
                e2 = bord.clone();
            }
        }
    }
    Ok((e1, e2))
}
