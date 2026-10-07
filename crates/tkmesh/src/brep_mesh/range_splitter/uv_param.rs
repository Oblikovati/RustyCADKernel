// Rust translation of BRepMesh_UVParamRangeSplitter.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkernel::ncollection::IndexedMap;

/// `IMeshData::IMapOfReal`: the indexed, insertion-ordered set of parameters.
pub type IMapOfReal = IndexedMap<f64>;

/// `BRepMesh_UVParamRangeSplitter`: the default splitter plus the U and V parameters of
/// the boundary points; `UVParamRangeSplitter::default()`.
pub struct UVParamRangeSplitter {
    base: DefaultRangeSplitter,
    u_params: IMapOfReal,
    v_params: IMapOfReal,
}
impl Default for UVParamRangeSplitter {
    fn default() -> Self {
        Self {
            base: DefaultRangeSplitter::default(),
            u_params: IMapOfReal::new(1),
            v_params: IMapOfReal::new(1),
        }
    }
}
impl std::ops::Deref for UVParamRangeSplitter {
    type Target = DefaultRangeSplitter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for UVParamRangeSplitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl UVParamRangeSplitter {
    /// `Reset`: the default reset, then both parameter maps cleared;
    /// `splitter.reset_params(&face, &parameters)`.
    pub fn reset_params(&mut self, face: &FaceHandle, parameters: &MeshParameters) {
        self.base.reset(face, parameters);
        self.u_params.clear(false);
        self.v_params.clear(false);
    }
    /// `GetParametersU() const`; `splitter.get_parameters_u().extent()`.
    pub fn get_parameters_u(&self) -> &IMapOfReal {
        &self.u_params
    }
    /// `GetParametersU()`; `splitter.change_parameters_u().add(0.5)`.
    pub fn change_parameters_u(&mut self) -> &mut IMapOfReal {
        &mut self.u_params
    }
    /// `GetParametersV() const`; `splitter.get_parameters_v().extent()`.
    pub fn get_parameters_v(&self) -> &IMapOfReal {
        &self.v_params
    }
    /// `GetParametersV()`; `splitter.change_parameters_v().add(0.5)`.
    pub fn change_parameters_v(&mut self) -> &mut IMapOfReal {
        &mut self.v_params
    }
    /// The base AddPoint, then the point's U and V recorded (the torus and
    /// boundary-parameters overrides); `splitter.add_point_params(point)`.
    pub fn add_point_params(&mut self, point: Point2) {
        self.base.add_point(point);
        self.u_params.add(point.x());
        self.v_params.add(point.y());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_point_params_records_each_distinct_coordinate_once() {
        let mut splitter = UVParamRangeSplitter::default();
        splitter.add_point_params(Point2::new(1.0, 2.0));
        splitter.add_point_params(Point2::new(1.0, 3.0));
        assert_eq!(splitter.get_parameters_u().extent(), 1);
        assert_eq!(splitter.get_parameters_v().extent(), 2);
        assert_eq!(splitter.get_parameters_v().find_index(&3.0), 2);
    }
}
