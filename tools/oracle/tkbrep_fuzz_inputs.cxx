// Capture the fixed-seed inputs from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
// Include the unchanged test to verify each captured draw against its actual mutation.
#include <BRepGraph_Fuzz_Test.cxx>
#include <array>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <stdexcept>

namespace
{
struct CapturedMutation
{
  int Kind;
  uint32_t Index;
  gp_Pnt Delta;
};
uint64_t bits(double value)
{
  uint64_t result = 0;
  std::memcpy(&result, &value, sizeof(result));
  return result;
}
void require(bool condition, const char* message)
{
  if (!condition)
    throw std::runtime_error(message);
}
CapturedMutation capture(BRepGraph& graph, std::mt19937& rng)
{
  const int kind = std::uniform_int_distribution<int>(0, 2)(rng);
  const std::array<uint32_t, 3> counts = {
    graph.Topo().Edges().Nb(), graph.Topo().Vertices().Nb(), graph.Topo().Faces().Nb()};
  require(counts[kind] > 0, "empty mutation domain: expected positive count");
  const uint32_t index = std::uniform_int_distribution<int>(0, counts[kind] - 1)(rng);
  gp_Pnt delta;
  if (kind == 1)
  {
    std::uniform_real_distribution<double> dist(-0.1, 0.1);
    // Preserve native argument evaluation; verify the resulting point below.
    delta = gp_Pnt(dist(rng), dist(rng), dist(rng));
  }
  return {kind, index, delta};
}
gp_Pnt expectedPoint(const BRepGraph& graph, const CapturedMutation& mutation)
{
  if (mutation.Kind != 1)
    return gp_Pnt();
  const BRepGraph_VertexId id(mutation.Index);
  require(!id.IsRemoved(graph), "removed vertex: expected active input domain");
  const gp_Pnt old = graph.Topo().Vertices().Definition(id).Point;
  return gp_Pnt(old.X() + mutation.Delta.X(), old.Y() + mutation.Delta.Y(),
                old.Z() + mutation.Delta.Z());
}
double mutationTolerance(const BRepGraph& graph, const CapturedMutation& mutation)
{
  if (mutation.Kind == 1)
    return 0.0;
  if (mutation.Kind == 0)
  {
    const BRepGraph_EdgeId id(mutation.Index);
    require(!id.IsRemoved(graph), "removed edge: expected active input domain");
    return graph.Topo().Edges().Definition(id).Tolerance;
  }
  const BRepGraph_FaceId id(mutation.Index);
  require(!id.IsRemoved(graph), "removed face: expected active input domain");
  return graph.Topo().Faces().Definition(id).Tolerance;
}
void checkPoint(const BRepGraph& graph, const CapturedMutation& mutation, const gp_Pnt& expected)
{
  if (mutation.Kind != 1)
    return;
  const gp_Pnt actual = graph.Topo().Vertices().Definition(BRepGraph_VertexId(mutation.Index)).Point;
  require(bits(actual.X()) == bits(expected.X()), "vertex X differs: expected identical native draw");
  require(bits(actual.Y()) == bits(expected.Y()), "vertex Y differs: expected identical native draw");
  require(bits(actual.Z()) == bits(expected.Z()), "vertex Z differs: expected identical native draw");
}
CapturedMutation checkedMutation(BRepGraph& graph, std::mt19937& rng)
{
  std::mt19937 prediction = rng;
  const CapturedMutation mutation = capture(graph, prediction);
  const gp_Pnt expected = expectedPoint(graph, mutation);
  const double tolerance = mutationTolerance(graph, mutation);
  require(applyOne(graph, rng), "skipped mutation: expected active original fixture");
  require(rng == prediction, "RNG state differs: expected original draw consumption");
  checkPoint(graph, mutation, expected);
  if (mutation.Kind != 1)
    require(bits(mutationTolerance(graph, mutation)) == bits(tolerance + 1.e-4),
            "tolerance differs: expected original 1e-4 increment");
  require(BRepGraph_Validate::Perform(graph, BRepGraph_Validate::Options::Audit()).IsValid(),
          "invalid graph: expected original audit to pass after mutation");
  return mutation;
}
void writeMutation(std::ostream& output, uint32_t seed, int iteration, const CapturedMutation& mutation)
{
  output << std::dec << seed << ' ' << iteration << ' ' << mutation.Kind << ' ' << mutation.Index
         << std::hex << ' ' << bits(mutation.Delta.X()) << ' ' << bits(mutation.Delta.Y())
         << ' ' << bits(mutation.Delta.Z()) << '\n';
}
void captureSeed(std::ostream& output, uint32_t seed, bool cylinder)
{
  BRepGraph graph;
  graph.Clear();
  const TopoDS_Shape shape = cylinder ? BRepPrimAPI_MakeCylinder(5.0, 15.0).Shape()
                                      : BRepPrimAPI_MakeBox(10.0, 20.0, 30.0).Shape();
  const auto build = graph.Shapes().Add(shape);
  require(build.IsOk() && !graph.IsEmpty(), "empty seed graph: expected successful native builder");
  require(BRepGraph_Validate::Perform(graph, BRepGraph_Validate::Options::Audit()).IsValid(),
          "invalid seed graph: expected original initial audit to pass");
  std::mt19937 rng(seed);
  for (int iteration = 0; iteration < (cylinder ? 40 : 50); ++iteration)
    writeMutation(output, seed, iteration, checkedMutation(graph, rng));
}
void captureFixture(const char* path, bool cylinder)
{
  std::ofstream output(path);
  require(output.good(), "output open failed: expected writable fixture path");
  output << "# seed iteration kind index dx_bits dy_bits dz_bits\n";
  for (uint32_t seed : {1u, 7u, 42u, 137u, 2026u, 0xC0FFEEu, 0xDEADBEEFu})
    captureSeed(output, seed, cylinder);
  require(output.good(), "output write failed: expected complete fixture");
}
} // namespace
int main(int argc, char** argv)
{
  require(argc == 3, "wrong argument count: expected box-output cylinder-output");
  captureFixture(argv[1], false);
  captureFixture(argv[2], true);
}
