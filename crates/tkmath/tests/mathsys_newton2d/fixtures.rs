struct QuadraticFunc;
impl SmallSystem<2> for QuadraticFunc {
    fn evaluate(&mut self, x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]) -> bool {
        let [theU, theV] = *x;
        theF[0] = 2.0 * theU;
        theF[1] = 2.0 * theV;
        theJ[0][0] = 2.0;
        theJ[0][1] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 2.0;
        true
    }
}
struct GenericLinearExactStep;
impl SmallSystem<2> for GenericLinearExactStep {
    fn evaluate(&mut self, x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]) -> bool {
        let [theU, theV] = *x;
        theF[0] = theU - 1.0;
        theF[1] = theV - 2.0;
        theJ[0][0] = 1.0;
        theJ[0][1] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0;
        true
    }
}
struct GenericHugeJacobianConstantResidual;
impl SmallSystem<2> for GenericHugeJacobianConstantResidual {
    fn evaluate(&mut self, x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]) -> bool {
        let [theU, theV] = *x;
        theF[0] = 1.0;
        theF[1] = 1.0;
        theJ[0][0] = 1.0e20;
        theJ[0][1] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0e20;
        true
    }
}
struct SymmetricDistFunc {
    myTargetU: f64,
    myTargetV: f64,
}
impl SymmetricFunction2D for SymmetricDistFunc {
    fn value(&mut self, x: &[f64; 2], f: &mut [f64; 2]) -> bool {
        let [theU, theV] = *x;
        f[0] = 2.0 * (theU - self.myTargetU);
        f[1] = 2.0 * (theV - self.myTargetV);
        true
    }
    fn value_and_jacobian(&mut self, x: &[f64; 2], f: &mut [f64; 2], j: &mut [f64; 3]) -> bool {
        let [theU, theV] = *x;
        f[0] = 2.0 * (theU - self.myTargetU);
        f[1] = 2.0 * (theV - self.myTargetV);
        j[0] = 2.0;
        j[1] = 0.0;
        j[2] = 2.0;
        true
    }
}
struct SymmetricHeterogeneousFunc;
impl SymmetricFunction2D for SymmetricHeterogeneousFunc {
    fn value(&mut self, x: &[f64; 2], f: &mut [f64; 2]) -> bool {
        let [theU, theV] = *x;
        f[0] = 1.0e20 * (theU - 100.0);
        f[1] = theV + 200.0;
        true
    }
    fn value_and_jacobian(&mut self, x: &[f64; 2], f: &mut [f64; 2], j: &mut [f64; 3]) -> bool {
        let [theU, theV] = *x;
        self.value(x, f);
        j[0] = 1.0e20;
        j[1] = 0.0;
        j[2] = 1.0;
        true
    }
}
struct HugeJacobianConstantResidual;
impl SymmetricFunction2D for HugeJacobianConstantResidual {
    fn value(&mut self, x: &[f64; 2], f: &mut [f64; 2]) -> bool {
        let [theU, theV] = *x;
        f[0] = 1.0;
        f[1] = 1.0;
        true
    }
    fn value_and_jacobian(&mut self, x: &[f64; 2], f: &mut [f64; 2], j: &mut [f64; 3]) -> bool {
        let [theU, theV] = *x;
        f[0] = 1.0;
        f[1] = 1.0;
        j[0] = 1.0e20;
        j[1] = 0.0;
        j[2] = 1.0e20;
        true
    }
}
struct ArmijoRegressionFunc;
impl SymmetricFunction2D for ArmijoRegressionFunc {
    fn value(&mut self, x: &[f64; 2], f: &mut [f64; 2]) -> bool {
        let [theU, theV] = *x;
        f[0] = 1.0;
        f[1] = 2.0;
        true
    }
    fn value_and_jacobian(&mut self, x: &[f64; 2], f: &mut [f64; 2], j: &mut [f64; 3]) -> bool {
        let [theU, theV] = *x;
        f[0] = 1.0;
        f[1] = 2.0;
        j[0] = 1.0;
        j[1] = 0.0;
        j[2] = -1.0;
        true
    }
}
