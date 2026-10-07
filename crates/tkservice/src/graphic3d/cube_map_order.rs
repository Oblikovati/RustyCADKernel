//! `Graphic3d_CubeMapOrder` and `Graphic3d_ValidatedCubeMapOrder`: the permutation that maps
//! cube map sides to tile (or file) indices, packed as six 3-bit digits.
use super::Graphic3dCubeMapSide;
use rustycad_tkernel::standard::Failure;
use std::sync::OnceLock;

/// A side-to-index order; `Graphic3dCubeMapOrder::new(0, 1, 2, 3, 4, 5).is_valid()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Graphic3dCubeMapOrder {
    /// `myConvolution`: the six indices as base-8 digits.
    convolution: u32,
    /// `myHasOverflows`: whether an index above 5 was assigned.
    has_overflows: bool,
}

/// An order checked to be a permutation (`Graphic3d_ValidatedCubeMapOrder`); made only by
/// [`Graphic3dCubeMapOrder::validated`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graphic3dValidatedCubeMapOrder {
    order: Graphic3dCubeMapOrder,
}
impl Graphic3dValidatedCubeMapOrder {
    /// The valid order (`Order`, `operator->`); `validated.order().get(side)`.
    pub fn order(&self) -> &Graphic3dCubeMapOrder {
        &self.order
    }
}

const SIDES: [Graphic3dCubeMapSide; 6] = {
    use Graphic3dCubeMapSide as S;
    [S::PosX, S::NegX, S::PosY, S::NegY, S::PosZ, S::NegZ]
};

impl Graphic3dCubeMapOrder {
    /// The order of the six sides' locations; values above 5 mark an overflow.
    /// Example: `Graphic3dCubeMapOrder::new(1, 0, 2, 3, 4, 5)`.
    pub fn new(pos_x: u8, neg_x: u8, pos_y: u8, neg_y: u8, pos_z: u8, neg_z: u8) -> Self {
        let mut order = Self::default();
        for (side, value) in SIDES
            .into_iter()
            .zip([pos_x, neg_x, pos_y, neg_y, pos_z, neg_z])
        {
            order.set(side, value);
        }
        order
    }
    /// `Graphic3d_CubeMapOrder(const Graphic3d_ValidatedCubeMapOrder&)`;
    /// `Graphic3dCubeMapOrder::from_validated(&Graphic3dCubeMapOrder::default_order())`.
    pub fn from_validated(order: &Graphic3dValidatedCubeMapOrder) -> Self {
        order.order
    }
    /// `Set(const Graphic3d_CubeMapOrder&)`: copies another order; `order.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// `Validated`: the checked order, or `Standard_Failure` for repetitions or overflows.
    /// Example: `order.validated()?`.
    pub fn validated(&self) -> Result<Graphic3dValidatedCubeMapOrder, Failure> {
        if !self.is_valid() {
            let message = format!(
                "Try of Graphic3d_ValidatedCubeMapOrder creation using invalid Graphic3d_CubeMapOrder; got {self:?}"
            );
            return Err(Failure::with_type("Standard_Failure", &message, ""));
        }
        Ok(Graphic3dValidatedCubeMapOrder { order: *self })
    }
    /// `Set(side, value)`: assigns a location; a value above 5 only marks an overflow.
    /// Example: `order.set(Graphic3dCubeMapSide::PosX, 3)`.
    pub fn set(&mut self, side: Graphic3dCubeMapSide, value: u8) -> &mut Self {
        if value > 5 {
            self.has_overflows = true;
            return self;
        }
        self.set_digit(side as u8, value);
        self
    }
    /// The identity order; `order.set_default()`.
    pub fn set_default(&mut self) -> &mut Self {
        for index in 0..6u8 {
            self.set_digit(index, index);
        }
        self
    }
    /// Composes with a permutation: each location `l` becomes `permutation[l]`.
    /// Example: `order.permute(&permutation)`.
    pub fn permute(&mut self, permutation: &Graphic3dValidatedCubeMapOrder) -> &mut Self {
        for index in 0..6u8 {
            let value = permutation.order.digit(self.digit(index));
            self.set_digit(index, value);
        }
        self
    }
    /// A permuted copy; `order.permuted(&permutation)`.
    pub fn permuted(&self, permutation: &Graphic3dValidatedCubeMapOrder) -> Self {
        let mut order = *self;
        order.permute(permutation);
        order
    }
    /// Swaps the locations of two sides; `order.swap(Graphic3dCubeMapSide::PosY, Graphic3dCubeMapSide::NegY)`.
    pub fn swap(&mut self, first: Graphic3dCubeMapSide, second: Graphic3dCubeMapSide) -> &mut Self {
        let saved = self.get(first);
        self.set_digit(first as u8, self.get(second));
        self.set_digit(second as u8, saved);
        self
    }
    /// A swapped copy; `order.swapped(Graphic3dCubeMapSide::PosY, Graphic3dCubeMapSide::NegY)`.
    pub fn swapped(&self, first: Graphic3dCubeMapSide, second: Graphic3dCubeMapSide) -> Self {
        let mut order = *self;
        order.swap(first, second);
        order
    }
    /// The location of a side (`Get`, `operator[]`); `order.get(Graphic3dCubeMapSide::NegZ)`.
    pub fn get(&self, side: Graphic3dCubeMapSide) -> u8 {
        self.digit(side as u8)
    }
    /// Resets to all zeros without overflow; `order.clear()`.
    pub fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
    /// Whether every location is 0; `order.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.convolution == 0
    }
    /// Whether two sides share a location; `order.has_repetitions()`.
    pub fn has_repetitions(&self) -> bool {
        let mut seen = [false; 8];
        (0..6u8).any(|index| std::mem::replace(&mut seen[self.digit(index) as usize], true))
    }
    /// Whether a location above 5 was assigned; `order.has_overflows()`.
    pub fn has_overflows(&self) -> bool {
        self.has_overflows
    }
    /// A permutation without overflow; `order.is_valid()`.
    pub fn is_valid(&self) -> bool {
        !self.has_repetitions() && !self.has_overflows()
    }
    /// The identity order, validated once; `Graphic3dCubeMapOrder::default_order()`.
    pub fn default_order() -> Graphic3dValidatedCubeMapOrder {
        static DEFAULT: OnceLock<Graphic3dValidatedCubeMapOrder> = OnceLock::new();
        *DEFAULT.get_or_init(|| Graphic3dValidatedCubeMapOrder {
            order: *Self::default().set_default(),
        })
    }
    /// `operator[]` of a cube map's raw side value, which goes past `NEG_Z` after the end of
    /// an iteration: the digit of `(unsigned char)side`; a shift beyond 31 bits (undefined
    /// natively) reads 0. Example: `order.location_of_side_index(6) == 0`.
    pub(crate) fn location_of_side_index(&self, side: i32) -> u8 {
        let shift = u32::from(side as u8) * 3;
        self.convolution
            .checked_shr(shift)
            .map_or(0, |digits| (digits % 8) as u8)
    }
    /// The private `get`: the base-8 digit of an index.
    fn digit(&self, index: u8) -> u8 {
        ((self.convolution / (1u32 << (u32::from(index) * 3))) % 8) as u8
    }
    /// The private `set`: replaces the base-8 digit of an index (wrapping as natively).
    fn set_digit(&mut self, index: u8, value: u8) {
        let place = 1u32 << (u32::from(index) * 3);
        self.convolution = self
            .convolution
            .wrapping_sub(place.wrapping_mul(u32::from(self.digit(index))));
        self.convolution = self
            .convolution
            .wrapping_add(place.wrapping_mul(u32::from(value)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graphic3dCubeMapSide as S;

    #[test]
    fn orders_validate_permutations_only() {
        assert!(
            Graphic3dCubeMapOrder::new(5, 4, 3, 2, 1, 0)
                .validated()
                .is_ok()
        );
        assert!(Graphic3dCubeMapOrder::new(0, 0, 2, 3, 4, 5).has_repetitions());
        let overflow = Graphic3dCubeMapOrder::new(6, 1, 2, 3, 4, 5);
        assert!(overflow.has_overflows());
        assert_eq!(
            overflow.validated().unwrap_err().exception_type(),
            "Standard_Failure"
        );
        assert!(Graphic3dCubeMapOrder::default().is_empty());
    }

    #[test]
    fn swap_and_permute_compose_locations() {
        let order = Graphic3dCubeMapOrder::default_order();
        let swapped = order.order().swapped(S::PosY, S::NegY);
        assert_eq!((swapped.get(S::PosY), swapped.get(S::NegY)), (3, 2));
        let reversed = Graphic3dCubeMapOrder::new(5, 4, 3, 2, 1, 0)
            .validated()
            .unwrap();
        let permuted = swapped.permuted(&reversed);
        assert_eq!(permuted.get(S::PosX), 5);
        assert_eq!(permuted.get(S::PosY), 2);
        assert_eq!(
            Graphic3dCubeMapOrder::from_validated(&reversed).get(S::NegZ),
            0
        );
    }
}
