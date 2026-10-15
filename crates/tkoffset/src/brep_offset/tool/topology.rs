// Rust translation of BRepOffset_Tool::MapVertexEdges, BuildNeighbour, Deboucle3D and
// CorrectOrientation (with the static IsInOut) of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The topological services of `BRepOffset_Tool`: vertex-edge maps, the neighbours of
//! the edges of a wire, the removal of open shells and the orientation of new edges.
use super::BRepOffsetTool;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator, Solid, Wire};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d as Adaptor2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The edges incident to each vertex (`MVE`).
pub type VertexEdges = DataMap<Shape, List<Shape>, ShapeMapHasher>;
/// The neighbour edge of each edge at one of its vertices (`NOnV1`, `NOnV2`).
pub type EdgeNeighbours = DataMap<Shape, Shape, ShapeMapHasher>;

impl BRepOffsetTool {
    /// `MapVertexEdges(S, MEV)`: appends each edge of the shape (once) to the lists of
    /// its vertices; `BRepOffsetTool::map_vertex_edges(&shape, &mut mev)?`.
    pub fn map_vertex_edges(s: &Shape, mev: &mut VertexEdges) -> Result<(), Failure> {
        let mut deja_vu = MapOfShape::default();
        let mut exp = Explorer::new(
            &s.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )?;
        while exp.more() {
            let e = topods::edge(exp.current())?;
            exp.next()?;
            if !deja_vu.add(e.as_ref().clone()) {
                continue;
            }
            let (v1, v2) = topexp::vertices(&e, false);
            mev.try_emplaced(v1.as_ref().clone(), List::default)
                .append(e.as_ref().clone());
            if !v1.is_same(&v2) {
                mev.try_emplaced(v2.as_ref().clone(), List::default)
                    .append(e.as_ref().clone());
            }
        }
        Ok(())
    }

    /// `BuildNeighbour(W, F, NOnV1, NOnV2)`: the edge following each edge of the wire
    /// (in the wire explorer order on the face) at its first and its last vertex;
    /// `BRepOffsetTool::build_neighbour(&wire, &face, &mut n1, &mut n2)?`.
    pub fn build_neighbour(
        w: &Wire,
        f: &Face,
        n_on_v1: &mut EdgeNeighbours,
        n_on_v2: &mut EdgeNeighbours,
    ) -> Result<(), Failure> {
        let local_face = topods::face(&f.oriented(Orientation::Forward))?;
        let local_wire = topods::wire(&w.oriented(Orientation::Forward))?;
        let mut wexp = WireExplorer::with_wire_on_face(&local_wire, &local_face)?;
        let first_e = wexp.current().clone();
        let (mut cur_e, mut prec_e) = (first_e.clone(), first_e.clone());
        let (mut v1, mut v2) = topexp::vertices(&cur_e, false);
        let (fv1, fv2) = (v1.clone(), v2.clone());
        let (mut vp1, mut vp2) = (v1.clone(), v2.clone());
        wexp.next()?;
        while wexp.more() {
            cur_e = wexp.current().clone();
            (v1, v2) = topexp::vertices(&cur_e, false);
            bind_neighbours(
                (&prec_e, &cur_e),
                (&v1, &v2),
                (&vp1, &vp2),
                (n_on_v1, n_on_v2),
            );
            prec_e = cur_e.clone();
            vp1 = v1.clone();
            vp2 = v2.clone();
            wexp.next()?;
        }
        bind_neighbours(
            (&first_e, &cur_e),
            (&v1, &v2),
            (&fv1, &fv2),
            (n_on_v1, n_on_v2),
        );
        Ok(())
    }

    /// `Deboucle3D(S, Boundary)`: a shell kept only when each of its free edges is a
    /// boundary edge, an internal edge of an internal face or degenerated; a compound
    /// or solid of its kept sub-shapes (null when none); null for any other type;
    /// `BRepOffsetTool::deboucle_3d(&shape, &boundary)?`.
    pub fn deboucle_3d(s: &Shape, boundary: &MapOfShape) -> Result<Shape, Failure> {
        match s.shape_type()? {
            ShapeEnum::Shell => {
                if keeps_shell(s, boundary)? {
                    return Ok(s.clone());
                }
                Ok(Shape::default())
            }
            ShapeEnum::Compound | ShapeEnum::Solid => deboucle_children(s, boundary),
            _ => Ok(Shape::default()),
        }
    }

    /// `CorrectOrientation(SI, NewEdges, AsDes, InitOffset, Offset)`: for a positive
    /// offset, reverses the new edges of each offset face whose p-curve lies outside
    /// its initial face; `BRepOffsetTool::correct_orientation(&si, &new_edges, &as_des, &init_offset, 2.)?`.
    pub fn correct_orientation(
        si: &Shape,
        new_edges: &IndexedMapOfShape,
        as_des: &BRepAlgoAsDes,
        init_offset: &BRepAlgoImage,
        offset: f64,
    ) -> Result<(), Failure> {
        let mut exp = Explorer::new(si, ShapeEnum::Face, ShapeEnum::Shape)?;
        while exp.more() {
            let fi = topods::face(exp.current())?;
            exp.next()?;
            for of in init_offset.image(fi.as_ref()) {
                correct_offset_face(&fi, &topods::face(&of)?, new_edges, as_des, offset)?;
            }
        }
        Ok(())
    }
}

/// The four vertex comparisons of `BuildNeighbour` between an edge `cur` (vertices
/// `v1`, `v2`) and the previous one `prec` (vertices `vp1`, `vp2`).
fn bind_neighbours(
    (prec, cur): (&Edge, &Edge),
    (v1, v2): (
        &rustycad_tkbrep::topods::Vertex,
        &rustycad_tkbrep::topods::Vertex,
    ),
    (vp1, vp2): (
        &rustycad_tkbrep::topods::Vertex,
        &rustycad_tkbrep::topods::Vertex,
    ),
    (n_on_v1, n_on_v2): (&mut EdgeNeighbours, &mut EdgeNeighbours),
) {
    let (p, c) = (prec.as_ref().clone(), cur.as_ref().clone());
    if v1.is_same(vp1) {
        n_on_v1.bind(p.clone(), c.clone());
        n_on_v1.bind(c.clone(), p.clone());
    }
    if v1.is_same(vp2) {
        n_on_v2.bind(p.clone(), c.clone());
        n_on_v1.bind(c.clone(), p.clone());
    }
    if v2.is_same(vp1) {
        n_on_v1.bind(p.clone(), c.clone());
        n_on_v2.bind(c.clone(), p.clone());
    }
    if v2.is_same(vp2) {
        n_on_v2.bind(p, c.clone());
        n_on_v2.bind(c, prec.as_ref().clone());
    }
}

/// Whether every free edge of the shell is acceptable to `Deboucle3D`.
fn keeps_shell(s: &Shape, boundary: &MapOfShape) -> Result<bool, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(s, ShapeEnum::Edge, ShapeEnum::Face, &mut map)?;
    for i in 1..=map.extent() {
        let a_lf = map.find_from_index(i)?;
        if a_lf.len() >= 2 {
            continue;
        }
        let an_edge = topods::edge(map.find_key(i)?)?;
        if an_edge.orientation() == Orientation::Internal {
            let a_face = a_lf.first().ok_or_else(no_face)?;
            if a_face.orientation() != Orientation::Internal {
                continue;
            }
        }
        if !boundary.contains(an_edge.as_ref()) && !BRepTool::degenerated(&an_edge)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The compound or solid of the kept sub-shapes of `Deboucle3D`.
fn deboucle_children(s: &Shape, boundary: &MapOfShape) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut ss = if s.shape_type()? == ShapeEnum::Compound {
        let mut c = Compound::default();
        builder.make_compound(&mut c);
        c.into_shape()
    } else {
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        solid.into_shape()
    };
    let mut nb_sub = 0;
    let mut it = ShapeIterator::new(s, true, true);
    while it.more() {
        let sub_shape = BRepOffsetTool::deboucle_3d(it.value()?, boundary)?;
        it.next();
        if !sub_shape.is_null() {
            builder.add(&mut ss, &sub_shape)?;
            nb_sub += 1;
        }
    }
    if nb_sub == 0 {
        return Ok(Shape::default());
    }
    Ok(ss)
}

/// `IsInOut(FC, AC, S)`: whether every point of a quasi-uniform deflection sampling of
/// the curve is classified with the state.
fn is_in_out(
    fc: &BRepTopAdaptorFClass2d,
    ac: &Geom2dAdaptorCurve,
    s: State,
) -> Result<bool, Failure> {
    let def = 100. * precision::confusion();
    let qu = QuasiUniformDeflection::with_curve(ac, def, rustycad_tkmath::geom_abs::Shape::C1)?;
    for i in 1..=qu.nb_points()? {
        let p = ac.value(qu.parameter(i)?)?;
        if fc.perform(p, true)? != s {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The `CorrectOrientation` treatment of one offset face of an initial face.
fn correct_offset_face(
    fi: &Face,
    of: &Face,
    new_edges: &IndexedMapOfShape,
    as_des: &BRepAlgoAsDes,
    offset: f64,
) -> Result<(), Failure> {
    let loe: Vec<Shape> = as_des.descendant(of.as_ref()).collect();
    if !as_des.has_descendant(of.as_ref()) {
        return Err(Failure::with_message(
            "Standard_NoSuchObject",
            "BRepAlgo_AsDes::ChangeDescendant: unbound offset face",
        ));
    }
    if !loe.iter().any(|oe| new_edges.contains(oe)) {
        return Ok(());
    }
    let local_face = topods::face(&fi.oriented(Orientation::Forward))?;
    let fc = BRepTopAdaptorFClass2d::new(&local_face, precision::confusion())?;
    let mut reversed = Vec::with_capacity(loe.len());
    for oe in &loe {
        let mut reverse = false;
        if new_edges.contains(oe) && offset > 0. {
            let pc = BRepTool::curve_on_face(&topods::edge(oe)?, of)?;
            let ac = Geom2dAdaptorCurve::with_range(&pc.pcurve, pc.first, pc.last)?;
            reverse = is_in_out(&fc, &ac, State::Out)?;
        }
        reversed.push(reverse);
    }
    as_des.with_change_descendant(of.as_ref(), |list| {
        if let Some(list) = list {
            for (oe, reverse) in list.iter_mut().zip(reversed) {
                if reverse {
                    oe.reverse();
                }
            }
        }
    });
    Ok(())
}

/// Standard_NoSuchObject of a free edge without face (`aLF.First()` of an empty list).
fn no_face() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "BRepOffset_Tool::Deboucle3D: free edge without face",
    )
}
