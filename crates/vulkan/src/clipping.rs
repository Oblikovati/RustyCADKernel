//! Immutable world-space halfspaces follow OCCT 3d097a0's fragment clip-chain rules.
use crate::{RenderError, error::rejected};

/// Intersect chains, each of which retains the union of its positive halfspaces.
/// Example: `RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], identity_clip())?`.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderClipping {
    world_from_object: [f32; 16],
    chains: Vec<Vec<[f32; 4]>>,
}
impl RenderClipping {
    /// Snapshot plane chains and the affine model transform; `RenderClipping::new(chains, model)?`.
    pub fn new(
        chains: Vec<Vec<[f32; 4]>>,
        world_from_object: [f32; 16],
    ) -> Result<Self, RenderError> {
        validate_model(&world_from_object)?;
        for chain in &chains {
            validate_chain(chain)?;
        }
        let count: usize = chains.iter().map(Vec::len).sum();
        u32::try_from(count).map_err(|_| rejected(count, "a 32-bit clipping plane count"))?;
        Ok(Self {
            world_from_object,
            chains,
        })
    }
    /// Read the immutable world-space chains; `clipping.chains()`.
    pub fn chains(&self) -> &[Vec<[f32; 4]>] {
        &self.chains
    }
    /// Read the model matrix used for plane evaluation; `clipping.world_from_object()`.
    pub fn world_from_object(&self) -> &[f32; 16] {
        &self.world_from_object
    }
    pub(crate) fn encode(&self) -> Vec<u8> {
        let count = self.chains.iter().map(Vec::len).sum::<usize>() as u32;
        let mut bytes: Vec<u8> = self
            .world_from_object
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect();
        bytes.extend([count, 0, 0, 0].into_iter().flat_map(u32::to_ne_bytes));
        for chain in &self.chains {
            for (index, equation) in chain.iter().enumerate() {
                bytes.extend(equation.iter().flat_map(|v| v.to_ne_bytes()));
                let remaining = (chain.len() - index) as u32;
                bytes.extend([remaining, 0, 0, 0].into_iter().flat_map(u32::to_ne_bytes));
            }
        }
        bytes
    }
}
fn validate_model(model: &[f32; 16]) -> Result<(), RenderError> {
    if !model.iter().all(|value| value.is_finite())
        || [model[3], model[7], model[11], model[15]] != [0., 0., 0., 1.]
    {
        return Err(rejected(model, "a finite affine column-major model matrix"));
    }
    Ok(())
}
fn validate_chain(chain: &[[f32; 4]]) -> Result<(), RenderError> {
    if chain.is_empty()
        || chain.iter().any(|plane| {
            !plane.iter().all(|v| v.is_finite()) || plane[..3].iter().all(|v| *v == 0.)
        })
    {
        return Err(rejected(
            chain,
            "a nonempty chain of finite equations with nonzero normals",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity_clip;
    #[test]
    fn storage_layout_preserves_chains_equations_and_model() {
        let chains = vec![
            vec![[1., 0., 0., -2.], [0., 1., 0., 3.]],
            vec![[0., 0., 1., 0.]],
        ];
        let mut model = identity_clip();
        model[12] = 4.;
        let clipping = RenderClipping::new(chains.clone(), model).unwrap();
        assert_eq!(clipping.chains(), chains);
        assert_eq!(*clipping.world_from_object(), model);
        let bytes = clipping.encode();
        assert_eq!(bytes.len(), 80 + 3 * 32);
        assert_eq!(&bytes[48..52], &4f32.to_ne_bytes());
        assert_eq!(&bytes[64..68], &3u32.to_ne_bytes());
        for (index, remaining) in [2u32, 1, 1].into_iter().enumerate() {
            let offset = 80 + index * 32;
            assert_eq!(&bytes[offset + 16..offset + 20], &remaining.to_ne_bytes());
        }
        assert_eq!(&bytes[92..96], &(-2f32).to_ne_bytes());
    }
    #[test]
    fn empty_volume_is_valid_but_empty_or_invalid_chains_are_rejected() {
        assert_eq!(
            RenderClipping::new(vec![], identity_clip())
                .unwrap()
                .encode()
                .len(),
            80
        );
        for chain in [
            vec![],
            vec![[0.; 4]],
            vec![[f32::NAN, 1., 0., 0.]],
            vec![[1., 0., 0., f32::INFINITY]],
        ] {
            assert!(RenderClipping::new(vec![chain], identity_clip()).is_err());
        }
    }
    #[test]
    fn affine_transform_validation_does_not_reject_mirroring_or_singular_scale() {
        for value in [0., -2.] {
            let mut model = identity_clip();
            model[0] = value;
            assert!(RenderClipping::new(vec![], model).is_ok());
        }
        for index in [0, 3, 7, 11, 15] {
            let mut model = identity_clip();
            model[index] = f32::INFINITY;
            assert!(RenderClipping::new(vec![], model).is_err());
        }
    }
}
