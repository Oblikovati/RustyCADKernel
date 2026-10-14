// SPDX-License-Identifier: GPL-2.0-only
// Capture actual OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 static factories.
#include <MeshVS_DataSource3D.hxx>
#include <iostream>
#include <limits>

using Topology = occ::handle<NCollection_HArray1<NCollection_Sequence<int>>>;
using Factory = Topology (*)(int);

static void EmitTopology(const char* kind, int count, Factory factory) {
  const Topology topology = factory(count);
  std::cout << kind << ' ' << count << ' ' << topology.IsNull();
  if (topology.IsNull()) {
    std::cout << '\n';
    return;
  }
  std::cout << ' ' << topology->Lower() << ' ' << topology->Upper();
  for (int face = topology->Lower(); face <= topology->Upper(); ++face) {
    const auto& nodes = topology->Value(face);
    std::cout << ' ' << nodes.Length();
    for (int node = 1; node <= nodes.Length(); ++node) std::cout << ' ' << nodes.Value(node);
  }
  std::cout << '\n';
}

static void EmitOwnership(const char* kind, Factory factory) {
  Topology first = factory(3);
  Topology second = factory(3);
  Topology alias = first;
  alias->ChangeValue(1).SetValue(1, 99);
  std::cout << kind << "-ownership " << (first == second) << ' ' << (first == alias)
            << ' ' << first->Value(1).Value(1) << ' ' << second->Value(1).Value(1);
  first.Nullify();
  std::cout << ' ' << alias->Value(1).Value(1) << '\n';
}

int main() {
  for (int count : {std::numeric_limits<int>::min(), -7, -1, 0, 1, 2, 3, 4, 5, 8, 16, 64}) {
    EmitTopology("prism", count, MeshVS_DataSource3D::CreatePrismTopology);
    EmitTopology("pyramid", count, MeshVS_DataSource3D::CreatePyramidTopology);
  }
  EmitOwnership("prism", MeshVS_DataSource3D::CreatePrismTopology);
  EmitOwnership("pyramid", MeshVS_DataSource3D::CreatePyramidTopology);
}
