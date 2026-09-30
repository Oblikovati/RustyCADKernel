// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_PlateG0Criterion.hxx/.cxx
// and GeomPlate_PlateG1Criterion.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The AdvApp2Var criteria of `GeomPlate_MakeApprox`: the G0 and G1 gaps at the plate
//! constraint points.
use super::criterion_patch::PatchPolynomial;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::adv_app2var::{
    AdvApp2VarContext, AdvApp2VarCriterion, AdvApp2VarCriterionRepartition,
    AdvApp2VarCriterionType, AdvApp2VarPatch,
};
use rustycad_tkmath::gp::{Point3, Vector3, Xy, Xyz};
use std::f64::consts::PI;

/// The fields of `AdvApp2Var_Criterion` and the constraint data of a plate criterion.
#[derive(Clone)]
struct PlateCriterionData {
    data: Sequence<Xy>,
    xyz: Sequence<Xyz>,
    max_value: f64,
    kind: AdvApp2VarCriterionType,
    repartition: AdvApp2VarCriterionRepartition,
}

/// The G0 criterion of a plate approximation: the greatest distance between the
/// patch and the target points `G0Data` at the parameters `Data` inside the patch
/// (`GeomPlate_PlateG0Criterion`); satisfied below `Maximum`.
/// `GeomPlatePlateG0Criterion::new(&uv, &points, 1e-3, AdvApp2VarCriterionType::Absolute, AdvApp2VarCriterionRepartition::Regular)`.
#[derive(Clone)]
pub struct GeomPlatePlateG0Criterion {
    fields: PlateCriterionData,
}

/// The G1 criterion of a plate approximation: the greatest angle between the patch
/// normal and the target normals `G1Data` at the parameters `Data` inside the patch
/// (`GeomPlate_PlateG1Criterion`); satisfied below `Maximum`.
/// `GeomPlatePlateG1Criterion::new(&uv, &normals, 1e-2, AdvApp2VarCriterionType::Absolute, AdvApp2VarCriterionRepartition::Regular)`.
#[derive(Clone)]
pub struct GeomPlatePlateG1Criterion {
    fields: PlateCriterionData,
}

impl PlateCriterionData {
    fn new(
        data: &Sequence<Xy>,
        values: &Sequence<Xyz>,
        maximum: f64,
        kind: AdvApp2VarCriterionType,
        repartition: AdvApp2VarCriterionRepartition,
    ) -> Self {
        Self {
            data: data.clone(),
            xyz: values.clone(),
            max_value: maximum,
            kind,
            repartition,
        }
    }

    /// The constraints inside the patch: their reduced parameters and target values.
    fn inside(&self, patch: &PatchPolynomial) -> Vec<((f64, f64), Xyz)> {
        let pairs = self.data.iter().zip(self.xyz.iter());
        pairs
            .filter(|(p2d, _)| patch.contains(**p2d))
            .map(|(p2d, value)| (patch.reduced(*p2d), *value))
            .collect()
    }
}

impl GeomPlatePlateG0Criterion {
    /// The criterion on the parameters `data`, the points `g0_data` and the bound
    /// `maximum` (`GeomPlate_PlateG0Criterion(Data, G0Data, Maximum, Type, Repart)`;
    /// the native defaults are Absolute and Regular).
    /// `GeomPlatePlateG0Criterion::new(&uv, &points, 1e-3, kind, repartition)`.
    pub fn new(
        data: &Sequence<Xy>,
        g0_data: &Sequence<Xyz>,
        maximum: f64,
        kind: AdvApp2VarCriterionType,
        repartition: AdvApp2VarCriterionRepartition,
    ) -> Self {
        Self {
            fields: PlateCriterionData::new(data, g0_data, maximum, kind, repartition),
        }
    }
}

impl GeomPlatePlateG1Criterion {
    /// The criterion on the parameters `data`, the normals `g1_data` and the bound
    /// `maximum` (`GeomPlate_PlateG1Criterion(Data, G1Data, Maximum, Type, Repart)`;
    /// the native defaults are Absolute and Regular).
    /// `GeomPlatePlateG1Criterion::new(&uv, &normals, 1e-2, kind, repartition)`.
    pub fn new(
        data: &Sequence<Xy>,
        g1_data: &Sequence<Xyz>,
        maximum: f64,
        kind: AdvApp2VarCriterionType,
        repartition: AdvApp2VarCriterionRepartition,
    ) -> Self {
        Self {
            fields: PlateCriterionData::new(data, g1_data, maximum, kind, repartition),
        }
    }
}

impl AdvApp2VarCriterion for GeomPlatePlateG0Criterion {
    /// The square root of the greatest squared distance between the patch point and
    /// the target point, over the constraints strictly inside the patch (`Value`).
    fn value(&self, p: &AdvApp2VarPatch, c: &AdvApp2VarContext) -> Result<(), Failure> {
        let patch = PatchPolynomial::read(p, c)?;
        let mut dist: f64 = 0.0;
        for (reduced, target) in self.fields.inside(&patch) {
            let digit: [f64; 3] = patch.eval(reduced, 0, 0)?;
            let p3d = Point3::new(digit[0], digit[1], digit[2]);
            let x: f64 = p3d.x() - target.x();
            let y: f64 = p3d.y() - target.y();
            let z: f64 = p3d.z() - target.z();
            let dist_tmp: f64 = x * x + y * y + z * z;
            if dist_tmp > dist {
                dist = dist_tmp;
            }
        }
        p.set_crit_value(dist.sqrt());
        Ok(())
    }
    fn is_satisfied(&self, p: &AdvApp2VarPatch) -> bool {
        p.crit_value() < self.fields.max_value
    }
    fn max_value(&self) -> f64 {
        self.fields.max_value
    }
    fn criterion_type(&self) -> AdvApp2VarCriterionType {
        self.fields.kind
    }
    fn repartition(&self) -> AdvApp2VarCriterionRepartition {
        self.fields.repartition
    }
}

/// The angle between the lines of `a` and `b` in `[0, PI/2]`, as `Value` folds it.
fn line_angle(a: Vector3, b: Vector3) -> Result<f64, Failure> {
    let angle: f64 = a.angle(b)?;
    if angle > PI / 2.0 {
        return Ok(PI - angle);
    }
    Ok(angle)
}

impl AdvApp2VarCriterion for GeomPlatePlateG1Criterion {
    /// The greatest angle between the normal lines of the patch and the targets, over
    /// the constraints strictly inside the patch (`Value`); Standard_ConstructionError
    /// (gp_VectorWithNullMagnitude) for a null normal, as natively.
    fn value(&self, p: &AdvApp2VarPatch, c: &AdvApp2VarContext) -> Result<(), Failure> {
        let patch = PatchPolynomial::read(p, c)?;
        let mut ang: f64 = 0.0;
        for (reduced, target) in self.fields.inside(&patch) {
            let v3h = Vector3::new(target.x(), target.y(), target.z());
            let d1u: [f64; 3] = patch.eval(reduced, 1, 0)?;
            let d1v: [f64; 3] = patch.eval(reduced, 0, 1)?;
            let v1s = Vector3::new(d1u[0], d1u[1], d1u[2]);
            let v2s = Vector3::new(d1v[0], d1v[1], d1v[2]);
            let angle: f64 = line_angle(v1s ^ v2s, v3h)?;
            if angle > ang {
                ang = angle;
            }
        }
        p.set_crit_value(ang);
        Ok(())
    }
    fn is_satisfied(&self, p: &AdvApp2VarPatch) -> bool {
        p.crit_value() < self.fields.max_value
    }
    fn max_value(&self) -> f64 {
        self.fields.max_value
    }
    fn criterion_type(&self) -> AdvApp2VarCriterionType {
        self.fields.kind
    }
    fn repartition(&self) -> AdvApp2VarCriterionRepartition {
        self.fields.repartition
    }
}
