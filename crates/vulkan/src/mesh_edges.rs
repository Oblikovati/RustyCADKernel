//! Native mesh edges mix with the surface in the same fragment, before alpha masking.
use crate::{PrimitiveDraw, PrimitiveTopology, RenderError, error::rejected};
/// Choose native mesh edge rasterization; `RenderMeshEdgePipeline::PolygonCompatibility`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderMeshEdgePipeline {
    /// Native programmable same-pass solid edges; patterned edges use polygon rasterization.
    #[default]
    Programmable,
    /// Native `OpenGl_Caps::usePolygonMode`, retaining actual triangle winding and raster edges.
    PolygonCompatibility,
}

/// Solid mesh edge parameters; `RenderMeshEdges::new([0., 1., 1., 1.], 1., false)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMeshEdges {
    pub(crate) color: [f32; 4],
    pub(crate) width: f32,
    pub(crate) feather: f32,
    pub(crate) skip_first: bool,
}
pub(crate) fn require_mesh_geometry(requested: bool, supported: bool) -> Result<(), RenderError> {
    if requested && !supported {
        return Err(rejected(
            (requested, supported),
            "geometry shader support for native same-pass mesh edges",
        ));
    }
    Ok(())
}
pub(crate) fn require_polygon_lines(requested: bool, supported: bool) -> Result<(), RenderError> {
    if requested && !supported {
        return Err(rejected(
            (requested, supported),
            "non-solid polygon rasterization support for native patterned mesh edges",
        ));
    }
    Ok(())
}
pub(crate) fn append_mesh_constants(
    bytes: &mut Vec<u8>,
    edges: Option<RenderMeshEdges>,
    extent: [u32; 2],
) {
    let Some(edges) = edges else {
        return;
    };
    bytes.extend(edges.color.iter().flat_map(|value| value.to_ne_bytes()));
    bytes.extend(
        [
            edges.width,
            edges.feather,
            extent[0] as f32,
            extent[1] as f32,
        ]
        .iter()
        .flat_map(|value| value.to_ne_bytes()),
    );
    bytes.extend(u32::from(edges.skip_first).to_ne_bytes());
    bytes.extend([0_u8; 12]);
}
fn validate_mesh_edge_parameters(color: [f32; 4], width: f32) -> Result<(), RenderError> {
    if color.iter().any(|v| !v.is_finite())
        || color[..3].iter().any(|v| !(0. ..=1.).contains(v))
        || !width.is_finite()
        || width <= 0.
    {
        return Err(rejected(
            (color, width),
            "finite alpha, RGB 0..=1 and positive finite mesh edge width",
        ));
    }
    Ok(())
}
impl RenderMeshEdges {
    /// Validate native same-pass edges; `RenderMeshEdges::new([0.,1.,1.,1.],2.,false)?`.
    pub fn new(color: [f32; 4], width: f32, skip_first: bool) -> Result<Self, RenderError> {
        validate_mesh_edge_parameters(color, width)?;
        Ok(Self {
            color,
            width,
            feather: 1.,
            skip_first,
        })
    }
    /// Preserve native rendering feather width; `edges.with_feather_width(2.)?`.
    pub fn with_feather_width(mut self, feather: f32) -> Result<Self, RenderError> {
        if !feather.is_finite() || feather < 0. {
            return Err(rejected(feather, "finite nonnegative mesh feather width"));
        }
        self.feather = feather;
        Ok(self)
    }
}
impl PrimitiveDraw {
    /// Mix edges with genuine triangle fragments; `draw.with_mesh_edges(edges)?`.
    pub fn with_mesh_edges(mut self, edges: RenderMeshEdges) -> Result<Self, RenderError> {
        if self.topology() != PrimitiveTopology::Triangles || self.textured_style.is_some() {
            return Err(rejected(
                self.topology(),
                "ordinary surface triangles for mesh edges",
            ));
        }
        self.mesh_edges = Some(edges);
        Ok(self)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_hollow_alpha_and_feather_are_preserved_without_normalizing() {
        let edges = RenderMeshEdges::new([0., 1., 1., -0.25], 2., false).unwrap();
        assert_eq!(edges.color[3], -0.25);
        assert_eq!(edges.with_feather_width(0.).unwrap().feather, 0.);
        assert_eq!(edges.with_feather_width(2.).unwrap().feather, 2.);
        for feather in [-1., f32::NAN, f32::INFINITY] {
            assert!(edges.with_feather_width(feather).is_err());
        }
        assert!(require_polygon_lines(false, false).is_ok());
        assert!(require_polygon_lines(true, true).is_ok());
        assert!(require_polygon_lines(true, false).is_err());
    }
    #[test]
    fn edge_state_validates_color_width_and_surface_topology() {
        for width in [0., -1., f32::INFINITY, f32::NAN] {
            assert!(RenderMeshEdges::new([1.; 4], width, false).is_err());
        }
        assert!(RenderMeshEdges::new([1.1, 1., 1., 1.], 1., false).is_err());
        let edges = RenderMeshEdges::new([0., 1., 1., 0.25], 3., true).unwrap();
        let draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        assert_eq!(draw.with_mesh_edges(edges).unwrap().mesh_edges, Some(edges));
        let points = PrimitiveDraw::points(
            vec![],
            vec![],
            crate::identity_clip(),
            crate::PointStyle::new(1., [1, 1], vec![[1.; 4]]).unwrap(),
        )
        .unwrap();
        assert!(points.with_mesh_edges(edges).is_err());
    }
    #[test]
    fn mesh_capability_and_push_bytes_preserve_existing_surface_prefix() {
        assert!(require_mesh_geometry(false, false).is_ok());
        assert!(require_mesh_geometry(true, true).is_ok());
        assert!(require_mesh_geometry(true, false).is_err());
        let mut bytes = vec![0x3c; 80];
        append_mesh_constants(&mut bytes, None, [64, 64]);
        assert_eq!(bytes.len(), 80);
        append_mesh_constants(
            &mut bytes,
            Some(RenderMeshEdges::new([1.; 4], 3., true).unwrap()),
            [64, 32],
        );
        assert_eq!(bytes.len(), 128);
        assert_eq!(&bytes[..80], &[0x3c; 80]);
        assert_eq!(f32::from_ne_bytes(bytes[96..100].try_into().unwrap()), 3.);
        assert_eq!(u32::from_ne_bytes(bytes[112..116].try_into().unwrap()), 1);
    }
}
