// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepMAT2d/BRepMAT2d_LinkTopoBilo.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMAT2d_LinkTopoBilo`: the links between the edges and vertices of the face of an
//! explorer and the basic elements of its bisecting locus.
use super::bisecting_locus::BRepMAT2dBisectingLocus;
use super::explorer::BRepMAT2dExplorer;
use crate::mat::MatBasicElt;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use std::collections::HashMap;

/// The basic elements of each edge or vertex.
type ShapeBasicElts = DataMap<Shape, Sequence<Handle<MatBasicElt>>, ShapeMapHasher>;

/// `BRepMAT2d_LinkTopoBilo` (docs/topalgo-design.md section 14): for each edge and vertex
/// of the face of an explorer, the basic elements of the bisecting locus it generates
/// (a curve element links to its edge, a point element to the vertex at the end of the
/// previous curve), and for each basic element its generating shape (reversed when the
/// element runs against the edge).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_mat2d::BRepMAT2dLinkTopoBilo;
/// let mut link = BRepMAT2dLinkTopoBilo::new();
/// link.init(&Shape::default());
/// assert!(!link.more()?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepMAT2dLinkTopoBilo {
    my_map: ShapeBasicElts,
    // `NCollection_DataMap<occ::handle<MAT_BasicElt>, TopoDS_Shape>`, only searched by key.
    my_be_shape: HashMap<Handle<MatBasicElt>, Shape>,
    my_key: Shape,
    current: i32,
    is_empty: bool,
}

impl Default for BRepMAT2dLinkTopoBilo {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepMAT2dLinkTopoBilo {
    /// `BRepMAT2d_LinkTopoBilo()`: no link; `BRepMAT2dLinkTopoBilo::new()`.
    pub fn new() -> Self {
        Self {
            my_map: ShapeBasicElts::default(),
            my_be_shape: HashMap::new(),
            my_key: Shape::default(),
            current: 0,
            is_empty: true,
        }
    }

    /// `BRepMAT2d_LinkTopoBilo(Explo, BiLo)`: the links after `Perform` (the iterator
    /// is not initialized natively; here it is empty until `Init`);
    /// `BRepMAT2dLinkTopoBilo::with_locus(&explorer, &locus)?`.
    pub fn with_locus(
        explo: &BRepMAT2dExplorer,
        bilo: &BRepMAT2dBisectingLocus,
    ) -> Result<Self, Failure> {
        let mut link = Self::new();
        link.perform(explo, bilo)?;
        Ok(link)
    }

    /// `Perform(Explo, BiLo)`: clears the links, then links each wire of the explorer's
    /// face (in explorer order) to the basic elements of its contour;
    /// Standard_ConstructionError when the explorer's shape is not a face;
    /// `link.perform(&explorer, &locus)?`.
    pub fn perform(
        &mut self,
        explo: &BRepMAT2dExplorer,
        bilo: &BRepMAT2dBisectingLocus,
    ) -> Result<(), Failure> {
        self.my_map.clear(false);
        self.my_be_shape.clear();
        let shape = explo.shape();
        if shape.shape_type()? != ShapeEnum::Face {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "BRepMAT2d_LinkTopoBilo::Perform: the explorer shape is not a face; expected a face",
                "",
            ));
        }
        let wires = Explorer::new(&shape, ShapeEnum::Wire, ShapeEnum::Shape)?;
        for (ind_contour, wire) in (1_i32..).zip(wires) {
            self.link_to_wire(&topods::wire(&wire)?, explo, ind_contour, bilo)?;
        }
        Ok(())
    }

    /// `Init(S)`: starts the iteration on the basic elements of an edge or vertex (empty
    /// when it has none); `link.init(&edge)`.
    pub fn init(&mut self, shape: &Shape) {
        self.is_empty = false;
        self.current = 1;
        if self.my_map.is_bound(shape) {
            self.my_key = shape.clone();
        } else {
            self.is_empty = true;
        }
    }

    /// `More()`: whether there is a current basic element (Standard_NoSuchObject when
    /// the links were recomputed since `Init`); `link.more()?`.
    pub fn more(&self) -> Result<bool, Failure> {
        if self.is_empty {
            return Ok(false);
        }
        Ok(self.current <= self.my_map.find(&self.my_key)?.length())
    }

    /// `Next()`; `link.next()`.
    pub fn next(&mut self) {
        self.current += 1;
    }

    /// `Value()`: the current basic element; Standard_OutOfRange past the end;
    /// `link.value()?`.
    pub fn value(&self) -> Result<Handle<MatBasicElt>, Failure> {
        let elements = self.my_map.find(&self.my_key)?;
        Ok(elements
            .value(usize::try_from(self.current).unwrap_or(0))?
            .clone())
    }

    /// `GeneratingShape(aBE)`: the edge or vertex of a basic element (reversed when the
    /// element runs against its edge); Standard_NoSuchObject for an unknown element;
    /// `link.generating_shape(&element)?`.
    pub fn generating_shape(&self, basic_elt: &Handle<MatBasicElt>) -> Result<Shape, Failure> {
        self.my_be_shape.get(basic_elt).cloned().ok_or_else(|| {
            let index = basic_elt.get().map_or(-1, MatBasicElt::index);
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!("NCollection_DataMap::operator(): basic element {index} is not linked; expected one of the locus"),
                "",
            )
        })
    }

    /// `LinkToWire` (private): each basic element of the contour `ind_c` (in the order of
    /// the map of `LinkToContour`) linked to its edge of the wire, or to the vertex at
    /// the end (positive link) or start (negative link) of that edge for a point.
    fn link_to_wire(
        &mut self,
        wire: &Wire,
        explo: &BRepMAT2dExplorer,
        ind_c: i32,
        bilo: &BRepMAT2dBisectingLocus,
    ) -> Result<(), Failure> {
        let mut topo_seq: Sequence<Shape> = Sequence::default();
        let mut the_exp = WireExplorer::with_wire(wire)?;
        while the_exp.more() {
            topo_seq.append(the_exp.current().clone().into());
            the_exp.next()?;
        }
        let link = link_to_contour(explo, ind_c, bilo)?;
        let graph = bilo.graph();
        let graph = graph.get().ok_or_else(null_graph)?;
        for (key, kc) in link.items() {
            let be = graph.basic_elt(*key)?;
            let is_point = bilo
                .geom_elt_of_basic_elt(&be)?
                .is_instance_name("Geom2d_CartesianPoint");
            let kc: i32 = *kc;
            let mut shape: Shape = topo_seq.value(kc.unsigned_abs() as usize)?.clone();
            if is_point {
                shape = end_vertex(&shape, kc)?;
            }
            self.bind(shape, be, kc)?;
        }
        Ok(())
    }

    /// Appends `be` to the elements of `shape` and records `shape` (reversed for a
    /// negative link) as its generating shape.
    fn bind(&mut self, shape: Shape, be: Handle<MatBasicElt>, kc: i32) -> Result<(), Failure> {
        if !self.my_map.is_bound(&shape) {
            self.my_map.bind(shape.clone(), Sequence::default());
        }
        self.my_map.change_find(&shape)?.append(be.clone());
        let generating = match kc < 0 {
            true => shape.oriented(top_abs::reverse(shape.orientation())),
            false => shape,
        };
        self.my_be_shape.insert(be, generating);
        Ok(())
    }
}

/// The vertex of a point element: the last vertex of the edge (in the edge's
/// orientation) for a positive link, the first for a negative one.
fn end_vertex(edge_shape: &Shape, kc: i32) -> Result<Shape, Failure> {
    let edge = topods::edge(edge_shape)?;
    let (first, last) = match edge_shape.orientation() == Orientation::Reversed {
        true => {
            let (vl, vf) = topexp::vertices(&edge, false);
            (vf, vl)
        }
        false => topexp::vertices(&edge, false),
    };
    Ok(if kc > 0 { last.into() } else { first.into() })
}

/// `LinkToContour`: for each basic element of the contour `ind_c` (by the index of the
/// element), the index of the curve of the contour it comes from, negative when the
/// element was met on the way back of an open line; a point element takes the curve
/// before it.
fn link_to_contour(
    explo: &BRepMAT2dExplorer,
    ind_c: i32,
    bilo: &BRepMAT2dBisectingLocus,
) -> Result<DataMap<i32, i32>, Failure> {
    let mut link: DataMap<i32, i32> = DataMap::default();
    let cont_length: i32 = explo.contour(ind_c)?.length();
    let mut walk = ContourWalk {
        direct_sense: true,
        last_point: false,
        ind_on_cont: 1,
        prec_ind_on_cont: -1,
        nb_sect: bilo.number_of_sections(ind_c, 1)?,
        i_sect: 0,
    };
    for i in 1..=bilo.number_of_elts(ind_c)? {
        let be = bilo.basic_elt(ind_c, i)?;
        let is_point = bilo
            .geom_elt_of_basic_elt(&be)?
            .is_instance_name("Geom2d_CartesianPoint");
        let index = be.get().ok_or_else(null_graph)?.index();
        link.bind(index, walk.link_of(is_point));
        walk.prec_ind_on_cont = walk.ind_on_cont;
        if !is_point && walk.i_sect == walk.nb_sect {
            walk.advance(cont_length, |ind_on_cont| {
                bilo.number_of_sections(ind_c, ind_on_cont)
            })?;
        }
    }
    Ok(link)
}

/// The state of `LinkToContour` along the curves of a contour.
struct ContourWalk {
    direct_sense: bool,
    last_point: bool,
    ind_on_cont: i32,
    prec_ind_on_cont: i32,
    nb_sect: i32,
    i_sect: i32,
}

impl ContourWalk {
    /// The link of the next element: a curve section counts and links to the current
    /// curve (negative on the way back), a point links to the previous curve (negative
    /// on the way back unless it is the last point).
    fn link_of(&mut self, is_point: bool) -> i32 {
        if !is_point {
            self.i_sect += 1;
            return if self.direct_sense {
                self.ind_on_cont
            } else {
                -self.ind_on_cont
            };
        }
        if self.direct_sense || self.last_point {
            self.prec_ind_on_cont
        } else {
            -self.prec_ind_on_cont
        }
    }

    /// After the last section of a curve: the next curve, or the turn back of an open
    /// line (the way back walks the curves in reverse).
    fn advance(
        &mut self,
        cont_length: i32,
        sections: impl Fn(i32) -> Result<i32, Failure>,
    ) -> Result<(), Failure> {
        if self.ind_on_cont < cont_length && self.direct_sense {
            self.ind_on_cont += 1;
            self.nb_sect = sections(self.ind_on_cont)?;
            self.i_sect = 0;
            return Ok(());
        }
        if !self.direct_sense {
            self.ind_on_cont -= 1;
            if self.ind_on_cont != 0 {
                self.nb_sect = sections(self.ind_on_cont)?;
            }
            self.last_point = false;
        } else {
            self.last_point = true;
        }
        self.i_sect = 0;
        self.direct_sense = false;
        Ok(())
    }
}

fn null_graph() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepMAT2d_LinkTopoBilo: null graph or basic element; expected a computed locus",
        "",
    )
}
