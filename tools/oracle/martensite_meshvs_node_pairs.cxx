// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0: real signed node fields, native hashes, equality and NCollection maps.
#include <MeshVS_TwoNodes.hxx>
#include <MeshVS_SymmetricPairHasher.hxx>
#include <NCollection_Map.hxx>
#include <array>
#include <iostream>
#include <limits>
#include <vector>

static std::vector<MeshVS_TwoNodes> Cases() {
  const std::array<int, 11> ids{std::numeric_limits<int>::min(),
    std::numeric_limits<int>::min()+1,-65536,-1,0,1,2,255,65535,
    std::numeric_limits<int>::max()-1,std::numeric_limits<int>::max()};
  std::vector<MeshVS_TwoNodes> result;
  for (int first : ids) for (int second : ids) result.emplace_back(first,second);
  return result;
}
static void HashRows(const std::vector<MeshVS_TwoNodes>& cases) {
  const MeshVS_SymmetricPairHasher policy;
  for (const auto& key : cases) {
    std::cout << "key " << key.First << " " << key.Second << " "
      << std::hash<MeshVS_TwoNodes>{}(key) << " " << policy({key.First,key.Second}) << "\n";
  }
}
static void EqualityRows(const std::vector<MeshVS_TwoNodes>& cases) {
  const MeshVS_SymmetricPairHasher policy;
  for (std::size_t index=0;index<cases.size();++index) {
    const auto& first=cases[index];
    std::cout << "equal " << index << " ";
    for (const auto& second:cases) std::cout << (first==second);
    std::cout << " ";
    for (const auto& second:cases) std::cout << policy({first.First,first.Second},{second.First,second.Second});
    std::cout << "\n";
  }
}
static void MapRows(const std::vector<MeshVS_TwoNodes>& cases) {
  NCollection_Map<MeshVS_TwoNodes> nodes;
  NCollection_Map<MeshVS_NodePair,MeshVS_SymmetricPairHasher> pairs;
  for (const auto& key:cases) {
    std::cout << "map " << nodes.Add(key) << " " << pairs.Add({key.First,key.Second}) << " "
      << nodes.Extent() << " " << pairs.Extent() << "\n";
  }
  const auto& stored=nodes.Added(MeshVS_TwoNodes(1,-1));
  const auto& pair=pairs.Added({1,-1});
  std::cout << "orientation " << stored.First << " " << stored.Second << " " << pair.first << " " << pair.second << "\n";
  std::cout << "remove " << nodes.Remove({1,-1}) << " " << pairs.Remove({1,-1}) << " "
    << nodes.Contains({-1,1}) << " " << pairs.Contains({-1,1}) << "\n";
}
int main() {
  const auto cases=Cases();
  const MeshVS_TwoNodes zero, first(-7);
  std::cout << "defaults " << zero.First << " " << zero.Second << " " << first.First << " " << first.Second << "\n";
  HashRows(cases);
  EqualityRows(cases);
  MapRows(cases);
}
