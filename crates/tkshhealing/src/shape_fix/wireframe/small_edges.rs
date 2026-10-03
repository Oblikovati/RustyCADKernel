use super::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists};
use crate::{shape_analysis::ShapeAnalysisWire, shape_extend::ShapeExtendWireData};
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Face, Shape, ShapeIterator},
    toptools::{MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{
    ncollection::{DataMap, List},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
struct SmallEdgeMaps<'a> {
    small: &'a mut MapOfShape,
    edge_faces: &'a mut ShapeFixWireframeShapeLists,
    face_small: &'a mut ShapeFixWireframeShapeLists,
    multiple: &'a mut MapOfShape,
}
impl ShapeFixWireframe {
    /// Append small-edge and adjacency observations; `tool.check_small_edges(&mut small,&mut edge_faces,&mut face_small,&mut multiple)?`.
    pub fn check_small_edges(
        &self,
        small: &mut MapOfShape,
        edge_faces: &mut ShapeFixWireframeShapeLists,
        face_small: &mut ShapeFixWireframeShapeLists,
        multiple: &mut MapOfShape,
    ) -> Result<bool, Failure> {
        let mut maps = SmallEdgeMaps {
            small,
            edge_faces,
            face_small,
            multiple,
        };
        self.scan_small_edge_maps(&mut maps)
    }
    fn scan_small_edge_maps(&self, maps: &mut SmallEdgeMaps<'_>) -> Result<bool, Failure> {
        let analyzer = ShapeAnalysisWire::new();
        let shape = self.shape();
        for face in Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.scan_small_face(&topods::face(&face)?, &analyzer, maps)?;
        }
        for wire in Explorer::new(&shape, ShapeEnum::Wire, ShapeEnum::Face)? {
            analyzer.set_precision(self.precision());
            analyzer.load_wire(&topods::wire(&wire)?)?;
            // OCCT stops even if caller-owned maps already contain small edges.
            if !analyzer.is_loaded() {
                return Ok(false);
            }
            scan_small_wire(&analyzer, None, self.precision(), maps)?;
        }
        Ok(!maps.small.is_empty())
    }
    fn scan_small_face(
        &self,
        original: &Face,
        analyzer: &ShapeAnalysisWire,
        maps: &mut SmallEdgeMaps<'_>,
    ) -> Result<(), Failure> {
        let face = if original.orientation() == Orientation::Reversed {
            topods::face(&original.oriented(Orientation::Forward))?
        } else {
            original.clone()
        };
        let mut small = List::default();
        for child in ShapeIterator::new(&face, true, true) {
            if child.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            initialize_small_wire(analyzer, &child, &face, self.precision())?;
            let found = scan_small_wire(analyzer, Some(original), self.precision(), maps)?;
            for edge in &found {
                small.append(edge.clone());
            }
        }
        if !small.is_empty() {
            maps.face_small.bind(original.clone().into_shape(), small);
        }
        Ok(())
    }
}
fn scan_small_wire(
    analyzer: &ShapeAnalysisWire,
    face: Option<&Face>,
    precision: f64,
    maps: &mut SmallEdgeMaps<'_>,
) -> Result<List<Shape>, Failure> {
    let wire = analyzer.wire_data();
    let counts = edge_multiplicity(&wire)?;
    let mut small = List::default();
    for index in 1..=wire.nb_edges() {
        let edge = wire.edge(index)?.into_shape();
        if skip_repeated_edge(&wire, index, &edge, &counts, maps.multiple)? {
            continue;
        }
        if let Some(face) = face {
            append_face_link(maps.edge_faces, edge.clone(), face)?;
        }
        if maps.small.contains(&edge) || analyzer.check_small_at(index, precision)? {
            maps.small.add(edge.clone());
            small.append(edge);
        }
    }
    Ok(small)
}
fn edge_multiplicity(
    wire: &ShapeExtendWireData,
) -> Result<DataMap<Shape, i32, ShapeMapHasher>, Failure> {
    let mut counts = DataMap::default();
    for index in 1..=wire.nb_edges() {
        let edge = wire.edge(index)?.into_shape();
        if counts.is_bound(&edge) {
            *counts.change_find(&edge)? += 1;
        } else {
            counts.bind(edge, 1);
        }
    }
    Ok(counts)
}
fn append_face_link(
    links: &mut ShapeFixWireframeShapeLists,
    edge: Shape,
    face: &Face,
) -> Result<(), Failure> {
    if !links.is_bound(&edge) {
        links.bind(edge.clone(), List::default());
    }
    links.change_find(&edge)?.append(face.clone().into_shape());
    Ok(())
}

fn initialize_small_wire(
    analyzer: &ShapeAnalysisWire,
    shape: &Shape,
    face: &Face,
    precision: f64,
) -> Result<(), Failure> {
    let wire = Handle::new(ShapeExtendWireData::from_wire(
        &topods::wire(shape)?,
        true,
        false,
    )?);
    analyzer.init_wire_data(&wire, face, precision)
}
fn skip_repeated_edge(
    wire: &ShapeExtendWireData,
    index: i32,
    edge: &Shape,
    counts: &DataMap<Shape, i32, ShapeMapHasher>,
    multiple: &mut MapOfShape,
) -> Result<bool, Failure> {
    if *counts.find(edge)? == 1 {
        return Ok(false);
    }
    if !wire.is_seam(index) {
        multiple.add(edge.clone());
    }
    Ok(true)
}
