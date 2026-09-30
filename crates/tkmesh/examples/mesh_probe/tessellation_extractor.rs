//! Native existing polygon extraction with location and parameter reuse.
use super::super::{flag, int, point3, real};
use super::post_processor::fixtures;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshModelPostProcessor, EdgeTessellationExtractor},
    imesh_tools::{CurveTessellator, MeshParameters, ModelAlgo},
};
pub fn tessellation_extractor_case(v: &[i32]) {
    let model = fixtures::post_model(&[v[0], 0, 0, 3, 1, 1, 0]);
    BRepMeshModelPostProcessor::new().perform(
        &model,
        &MeshParameters::default(),
        &ProgressRange::default(),
    );
    let edge = model.get_edge(0).unwrap();
    let face = model.get_face(0).unwrap();
    edge.set_same_param(v[1] != 0);
    let location = Location::new(Transform3::translation_vector(Vector3::new(
        f64::from(v[2]) * 7.0,
        0.0,
        0.0,
    )));
    edge.set_shape(&edge.get_shape().located(&location, false).unwrap());
    face.set_shape(&face.get_shape().located(&location, false).unwrap());
    let extractor = Handle::new(EdgeTessellationExtractor::new(&edge, &face).unwrap());
    int(extractor.points_nb());
    flag(
        extractor
            .get()
            .unwrap()
            .is_kind_name("IMeshTools_CurveTessellator"),
    );
    let mut point = Point3::default();
    let mut parameter = 0.0;
    let order = if v[3] == 0 {
        [1, 2, 1, 2]
    } else {
        [2, 1, 2, 1]
    };
    for index in order {
        flag(extractor.value(index, &mut point, &mut parameter).unwrap());
        point3(point);
        real(parameter);
    }
    println!();
}
