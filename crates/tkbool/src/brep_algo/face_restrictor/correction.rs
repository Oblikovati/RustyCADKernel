// Rust translation of PerformWithCorrection and the static IsClosed, IsInside, Store and
// BuildFaceIn of ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_FaceRestrictor.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces of the wires by their nesting: each closed wire turned to bound a finite
//! region, the wires inside each one recorded, then a face per outermost wire with the
//! wires right inside it as holes, and so on inwards.
use super::{BRepAlgoFaceRestrictor, WireNesting};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// `IsClosed(W)` (static): the closed flag of the wire, else its end vertices are
/// the same.
fn is_closed(w: &Shape) -> Result<bool, Failure> {
    let wire = topods::wire(w)?;
    if wire.closed()? {
        return Ok(true);
    }
    let (v1, v2) = topexp::wire_vertices(&wire);
    Ok(v1.as_ref().is_same(v2.as_ref()))
}

/// The parameter of IsInside on a p-curve range: its middle, else 0 for a range
/// infinite on both sides, else one unit inside the finite bound.
fn inside_parameter(f: f64, l: f64) -> f64 {
    let (neg, pos) = (
        precision::is_negative_infinite(f),
        precision::is_positive_infinite(l),
    );
    match (neg, pos) {
        (false, false) => (f + l) / 2.,
        (true, true) => 0.,
        (true, false) => l - 1.,
        (false, true) => f + 1.,
    }
}

/// `IsInside(wir, F, FClass2d)` (static): the point of the first edge of `wir` is IN
/// the face `f` (a new classifier is built; the given one is unused natively).
fn is_inside(wir: &Shape, f: &Shape) -> Result<bool, Failure> {
    let Some(first) = Explorer::new(wir, ShapeEnum::Edge, ShapeEnum::Shape)?
        .take(1)
        .collect::<Vec<Shape>>()
        .pop()
    else {
        return Ok(false);
    };
    let face = topods::face(f)?;
    let pc = BRepTool::curve_on_face(&topods::edge(&first)?, &face)?;
    let c2d = pc.pcurve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepAlgo_FaceRestrictor IsInside: the edge has no p-curve on the face; expected one",
            "BRepAlgo_FaceRestrictor",
        )
    })?;
    let pt2d = c2d.value(inside_parameter(pc.first, pc.last))?;
    let fclass2d = BRepTopAdaptorFClass2d::new(&face, precision::p_confusion())?;
    Ok(fclass2d.perform(pt2d, false)? == State::In)
}

/// `Store(W2, W1, keyIsIn, keyContains)` (static): `w2` is in `w1`.
fn store(w2: &Shape, w1: &Shape, key_is_in: &mut WireNesting, key_contains: &mut WireNesting) {
    key_is_in
        .try_emplaced(w2.clone(), List::default)
        .append(w1.clone());
    key_contains
        .try_emplaced(w1.clone(), List::default)
        .append(w2.clone());
}

/// The wires bound to `w` in a nesting map (empty when unbound).
fn nested(map: &WireNesting, w: &Shape) -> Vec<Shape> {
    map.seek(w)
        .map(|l| l.iter().cloned().collect())
        .unwrap_or_default()
}

/// Whether `w` has no wire left around it.
fn is_outermost(key_is_in: &WireNesting, w: &Shape) -> bool {
    key_is_in.seek(w).is_none_or(|l| l.is_empty())
}

/// An empty FORWARD copy of `f` bounded by the wire `w`.
fn face_on(f: &Shape, w: &Shape) -> Result<Shape, Failure> {
    let mut nf = f.empty_copied()?;
    nf.set_orientation(Orientation::Forward);
    BRepBuilder::default().add(&mut nf, w)?;
    Ok(nf)
}

/// The first wire same as `w` removed from the wires around each wire inside `w`.
fn remove_from_holders(
    w: &Shape,
    key_contains: &WireNesting,
    key_is_in: &mut WireNesting,
) -> Result<(), Failure> {
    for wi in nested(key_contains, w) {
        let l2 = key_is_in.change_seek(&wi).ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepAlgo_FaceRestrictor BuildFaceIn: a nested wire is unbound; expected a key",
                "BRepAlgo_FaceRestrictor",
            )
        })?;
        let mut removed = false;
        l2.retain(|x| {
            let drop = !removed && x.is_same(w);
            removed |= drop;
            !drop
        });
    }
    Ok(())
}

/// `BuildFaceIn(F, W, KeyContains, KeyIsIn, Orientation, Faces)` (static): the wires
/// right inside `w`, as holes of `f` (reversed) for FORWARD, else as the outer wires of
/// new copies of the first face of `faces`, recursively.
fn build_face_in(
    f: &mut Shape,
    w: &Shape,
    key_contains: &WireNesting,
    key_is_in: &mut WireNesting,
    orientation: Orientation,
    faces: &mut List<Shape>,
) -> Result<(), Failure> {
    let inside = nested(key_contains, w);
    if inside.is_empty() {
        return Ok(());
    }
    remove_from_holders(w, key_contains, key_is_in)?;
    let wire_ext: Vec<Shape> = inside
        .into_iter()
        .filter(|wi| is_outermost(key_is_in, wi))
        .collect();
    for wi in &wire_ext {
        if !is_outermost(key_is_in, wi) {
            continue;
        }
        build_one_face_in(f, wi, key_contains, key_is_in, orientation, faces)?;
    }
    Ok(())
}

/// One wire right inside: a hole of `f`, or the outer wire of a new face.
fn build_one_face_in(
    f: &mut Shape,
    wi: &Shape,
    key_contains: &WireNesting,
    key_is_in: &mut WireNesting,
    orientation: Orientation,
    faces: &mut List<Shape>,
) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    if orientation == Orientation::Forward {
        b.add(f, &wi.reversed())?;
        return build_face_in(f, wi, key_contains, key_is_in, Orientation::Reversed, faces);
    }
    let first = faces.first().ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "BRepAlgo_FaceRestrictor BuildFaceIn: no face; expected the outer one",
            "BRepAlgo_FaceRestrictor",
        )
    })?;
    let mut nf = first.empty_copied()?;
    b.add(&mut nf, wi)?;
    faces.append(nf.clone());
    build_face_in(
        &mut nf,
        wi,
        key_contains,
        key_is_in,
        Orientation::Forward,
        faces,
    )
}

impl BRepAlgoFaceRestrictor {
    /// The closed wires turned so that the infinite point is OUT of their face.
    fn orient_closed_wires(&mut self) -> Result<(), Failure> {
        let face = self.my_face.as_ref().clone();
        for w in self.wires.iter_mut() {
            let nf = face_on(&face, w)?;
            if !is_closed(w)? {
                continue;
            }
            let fclass2d =
                BRepTopAdaptorFClass2d::new(&topods::face(&nf)?, precision::p_confusion())?;
            if fclass2d.perform_infinite_point()? != State::Out {
                w.reverse();
            }
        }
        Ok(())
    }

    /// The wires inside each closed wire recorded in the nesting maps.
    fn classify_wires(&mut self) -> Result<(), Failure> {
        let wires: Vec<Shape> = self.wires.iter().cloned().collect();
        for w1 in &wires {
            if !is_closed(w1)? {
                continue;
            }
            let nf = face_on(self.my_face.as_ref(), w1)?;
            for w2 in &wires {
                if !w1.is_same(w2) && is_inside(w2, &nf)? {
                    store(w2, w1, &mut self.key_is_in, &mut self.key_contains);
                }
            }
        }
        Ok(())
    }

    /// `PerformWithCorrection()` (private): the faces by the nesting of the wires.
    pub(super) fn perform_with_correction(&mut self) -> Result<(), Failure> {
        self.my_done = false;
        self.orient_closed_wires()?;
        self.classify_wires()?;
        let wire_ext: Vec<Shape> = self
            .wires
            .iter()
            .filter(|w| is_outermost(&self.key_is_in, w))
            .cloned()
            .collect();
        for w in &wire_ext {
            if !is_outermost(&self.key_is_in, w) {
                continue;
            }
            let mut new_face = face_on(self.my_face.as_ref(), w)?;
            self.faces.append(new_face.clone());
            build_face_in(
                &mut new_face,
                w,
                &self.key_contains,
                &mut self.key_is_in,
                Orientation::Forward,
                &mut self.faces,
            )?;
        }
        self.my_done = true;
        Ok(())
    }
}
