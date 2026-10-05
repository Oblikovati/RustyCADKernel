"""Bind existing MathOpt Rust solvers to compiler-captured declarations."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]
SOLVER_BINDINGS = (
    (
        "MathOpt::BFGS(Function &,const math_Vector &,const MathUtils::Config &)",
        "43efa44d5a54bfe10bbbeb985e3a6f3d8e2d300ac94cceb851e2405620a96349",
        "rustycad_tkmath::math_opt::bfgs",
    ),
    (
        "MathOpt::BFGSNumerical(Function &,const math_Vector &,double,const MathUtils::Config &)",
        "924e4c828e72cfa9cd11ca1081b367e196fc7e4feab56b2b1e10130be9fe0bce",
        "rustycad_tkmath::math_opt::bfgs_numerical",
    ),
    (
        "MathOpt::LBFGS(Function &,const math_Vector &,size_t,const MathUtils::Config &)",
        "4c78f1812fe561b8b3fc83315712bcb821c3d260ab7a60ab9039755ba71b37ec",
        "rustycad_tkmath::math_opt::lbfgs",
    ),
    (
        "MathOpt::BFGSBounded(Function &,const math_Vector &,const math_Vector &,const math_Vector &,const MathUtils::Config &)",
        "cb9c0f772ff31247902c900746f729fc627ddb5145407a9f0c26f5e361522905",
        "rustycad_tkmath::math_opt::bfgs_bounded",
    ),
    (
        "MathOpt::Brent(Function &,double,double,const MathUtils::Config &)",
        "6fc87c53e119ebcc6506b0877bb0c06fc21d20bbca3344af24ec9d36c510aae5",
        "rustycad_tkmath::math_opt::brent",
    ),
    (
        "MathOpt::Golden(Function &,double,double,const MathUtils::Config &)",
        "1b8a89a08e9bc51f8deb3480ee7a4100e9d1fadb333664ddfe1c47f7cfb635b0",
        "rustycad_tkmath::math_opt::golden",
    ),
    (
        "MathOpt::BrentWithBracket(Function &,double,double,const MathUtils::Config &)",
        "7d27ceb36a3438d14a41d4e6f02fd874e5d42427f318b1798f18d264e4e61f2d",
        "rustycad_tkmath::math_opt::brent_with_bracket",
    ),
    (
        "MathOpt::FRPR(Function &,const math_Vector &,const MathOpt::FRPRConfig &)",
        "e5bb13b1985edd44379acae5199d837e009b8f138f27519caebd308937364bf8",
        "rustycad_tkmath::math_opt::frpr",
    ),
    (
        "MathOpt::FRPRNumerical(Function &,const math_Vector &,double,const MathOpt::FRPRConfig &)",
        "e06b0d0176c20cf7b0f123c70eba0d1b8c523b9ce9be733cb240b9c40f3493ff",
        "rustycad_tkmath::math_opt::frpr_numerical",
    ),
    (
        "MathOpt::GlobalConfig(MathOpt::GlobalStrategy,uint32_t)",
        "db011ffe13ca1f1f7eb022484bc748a2e48b514445e00c9ab8c3ac7dec6b1a41",
        "rustycad_tkmath::math_opt::GlobalConfig::new",
    ),
    (
        "MathOpt::DifferentialEvolution(Function &,const math_Vector &,const math_Vector &,const MathOpt::GlobalConfig &)",
        "26160703b287caf463f29413b8684f43c990dbd810feec532a00ce4d91e5d14d",
        "rustycad_tkmath::math_opt::differential_evolution",
    ),
    (
        "MathOpt::MultiStart(Function &,const math_Vector &,const math_Vector &,const MathOpt::GlobalConfig &)",
        "956e9c6352a8bc25f6988fb6abf310a639ed277b90908c11fa0fe6bb93a266ad",
        "rustycad_tkmath::math_opt::multi_start",
    ),
    (
        "MathOpt::GlobalMinimum(Function &,const math_Vector &,const math_Vector &,const MathOpt::GlobalConfig &)",
        "fac239c25904041e3c65528d640eb5fa4bc76da109ca3da221f2d1f61fe0e7db",
        "rustycad_tkmath::math_opt::global_minimum",
    ),
    (
        "MathOpt::GlobalMinimum(Function &,const math_Vector &,const math_Vector &,const MathOpt::GlobalConfig &,const MathOpt::PSOConfig *,const NCollection_DynamicArray<MathOpt::PSOSeedParticle> *,MathOpt::PSOStats *)",
        "be917a7f864c7f55366ce8c63bd17e5e0a8c4b60732bed5f874bcfcef222413c",
        "rustycad_tkmath::math_opt::global_minimum_with_pso",
    ),
    (
        "MathOpt::Newton(Function &,const math_Vector &,const MathOpt::NewtonConfig &)",
        "8b950ec52edaa01bb65ffdae42874224b2cdf3db21115f638887fc6fef7dc4a8",
        "rustycad_tkmath::math_opt::newton",
    ),
    (
        "MathOpt::NewtonModified(Function &,const math_Vector &,const MathOpt::NewtonConfig &)",
        "366495975e2499161c0873dd9522c91c654ef63de73dd7ee158152e42020ad44",
        "rustycad_tkmath::math_opt::newton_modified",
    ),
    (
        "MathOpt::NewtonNumericalHessian(Function &,const math_Vector &,double,const MathOpt::NewtonConfig &)",
        "f3011a16e89f5bcc35b3e03e523bfe5a4b2d63c607c8edaf78bc7030a2d57eb6",
        "rustycad_tkmath::math_opt::newton_numerical_hessian",
    ),
    (
        "MathOpt::NewtonNumerical(Function &,const math_Vector &,double,double,const MathOpt::NewtonConfig &)",
        "6e2e3e1c7c85eb780c7499e73bc9ead3726a3e863779416ae4431edea976ab3c",
        "rustycad_tkmath::math_opt::newton_numerical",
    ),
    (
        "MathOpt::NewtonBounded(Function &,const math_Vector &,const math_Vector &,const math_Vector &,const MathOpt::NewtonConfig &)",
        "fcd481bdacbfa11c615578635365bd85bb9034a917c60d2ab28b7776139c09f6",
        "rustycad_tkmath::math_opt::newton_bounded",
    ),
    (
        "MathOpt::PSO(Function &,const math_Vector &,const math_Vector &,const MathOpt::PSOConfig &,const NCollection_DynamicArray<MathOpt::PSOSeedParticle> *,MathOpt::PSOStats *)",
        "a4b6af1862585f925750e764734ea9f56170e5b77e08cd2b2f39aa75e70d2aee",
        "rustycad_tkmath::math_opt::pso_seeded",
    ),
    (
        "MathOpt::PSO(Function &,const math_Vector &,const math_Vector &,const MathOpt::PSOConfig &)",
        "c14bbfc1fadcdd8e7830d2e23575ceb63eb1db5f689db4fa466764837727f459",
        "rustycad_tkmath::math_opt::pso",
    ),
    (
        "MathOpt::Powell(Function &,const math_Vector &,const MathUtils::Config &)",
        "ba74fe43b5dbc2f602e4f6fe8a49610f94b1170bb08a90d2f0dbe5264aa07e7a",
        "rustycad_tkmath::math_opt::powell",
    ),
    (
        "MathOpt::PowellWithDirections(Function &,const math_Vector &,const math_Matrix &,const MathUtils::Config &)",
        "dda554dd0d53a233171f222ea09ac2a66db614d6c49f195bd4a7e6f3001831f2",
        "rustycad_tkmath::math_opt::powell_with_directions",
    ),
    (
        "MathOpt::Uzawa(const math_Matrix &,const math_Vector &,const math_Vector &,size_t,size_t,const MathOpt::UzawaConfig &)",
        "5c7cd6fcf96a57af8986d8006b81199269c17c17ccd91727335a964fc5aa4fc1",
        "rustycad_tkmath::math_opt::uzawa",
    ),
    (
        "MathOpt::UzawaEquality(const math_Matrix &,const math_Vector &,const math_Vector &,const MathOpt::UzawaConfig &)",
        "b19af190f816f945203e0ea9120920319f6cf71947c8dda925bd2074ca420aeb",
        "rustycad_tkmath::math_opt::uzawa_equality",
    ),
)


class MathOptSolverBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["MathOptSolverBindingsTest"]) -> None:
        path = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        with gzip.open(path, "rt") as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        document = json.loads((ROOT / "porting/parity.json").read_text())
        cls.by_selector = {row["upstream_id"]: row for row in document["api"]}

    def test_all_solver_selectors_are_unique_and_rust_bound(
        self: "MathOptSolverBindingsTest",
    ) -> None:
        self.assertEqual(len(SOLVER_BINDINGS), 25)
        for selector, compiler_id, rust_path in SOLVER_BINDINGS:
            status, candidates = cpp_candidates(selector, "TKMath", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            row = self.by_selector[selector]
            self.assertEqual(row["rust_item"], rust_path)
            self.assertEqual(row["status"], "implemented")


if __name__ == "__main__":
    unittest.main()
