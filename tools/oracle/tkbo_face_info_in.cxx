#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <NCollection_IndexedMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Map.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <algorithm>
#include <vector>

int main() {
  BRep_Builder builder;
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), -2, 2, -2, 2).Face();
  TopoDS_Vertex internal, alias1, alias2, vfVertex, efVertex;
  builder.MakeVertex(internal, gp_Pnt(0, 0, 0), 1.0e-7);
  builder.MakeVertex(alias1, gp_Pnt(0, 0, 0), 1.0e-7);
  builder.MakeVertex(alias2, gp_Pnt(0, 0, 0), 1.0e-7);
  builder.MakeVertex(vfVertex, gp_Pnt(0.5, 0, 0), 1.0e-7);
  builder.MakeVertex(efVertex, gp_Pnt(-0.5, 0, 0), 1.0e-7);
  builder.Add(face, internal);
  TopoDS_Compound argument;
  builder.MakeCompound(argument);
  builder.Add(argument, face);
  builder.Add(argument, alias1);
  builder.Add(argument, alias2);
  builder.Add(argument, vfVertex);
  builder.Add(argument, efVertex);
  TopoDS_Edge probeEdge = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)).Edge();
  builder.Add(argument, probeEdge);

  NCollection_List<TopoDS_Shape> args;
  args.Append(argument);
  BOPDS_DS ds;
  ds.SetArguments(args);
  ds.Init(1.0e-7);
  const int faceIndex = ds.Index(face);
  const int internalIndex = ds.Index(internal);
  const int alias1Index = ds.Index(alias1);
  const int alias2Index = ds.Index(alias2);
  const int vfIndex = ds.Index(vfVertex);
  const int efIndex = ds.Index(efVertex);
  const int edgeIndex = ds.Index(probeEdge);
  ds.AddShapeSD(internalIndex, alias1Index);
  ds.AddShapeSD(alias1Index, alias2Index);
  auto& vf = ds.InterfVF().Appended();
  vf.SetIndices(vfIndex, faceIndex);
  auto& efNew = ds.InterfEF().Appended();
  efNew.SetIndices(edgeIndex, faceIndex);
  efNew.SetIndexNew(efIndex);
  auto& paveBlocks = ds.ChangePaveBlocks(edgeIndex);
  occ::handle<BOPDS_PaveBlock> block = paveBlocks.First();
  occ::handle<BOPDS_CommonBlock> common = new BOPDS_CommonBlock;
  NCollection_List<occ::handle<BOPDS_PaveBlock>> commonBlocks;
  commonBlocks.Append(block);
  common->SetPaveBlocks(commonBlocks);
  NCollection_List<int> commonFaces;
  commonFaces.Append(faceIndex);
  common->SetFaces(commonFaces);
  ds.SetCommonBlock(block, common);
  auto& efCommon = ds.InterfEF().Appended();
  efCommon.SetIndices(edgeIndex, faceIndex);

  NCollection_IndexedMap<occ::handle<BOPDS_PaveBlock>> directBlocks;
  NCollection_Map<int> directVertices;
  ds.FaceInfoIn(faceIndex, directBlocks, directVertices);
  ds.ChangeFaceInfo(faceIndex);
  ds.UpdateFaceInfoIn(faceIndex);
  const BOPDS_FaceInfo& info = ds.FaceInfo(faceIndex);
  std::vector<int> vertices(directVertices.cbegin(), directVertices.cend());
  std::sort(vertices.begin(), vertices.end());
  std::cout << "in " << faceIndex << ' ' << edgeIndex << ' ' << ds.HasFaceInfo(faceIndex)
            << ' ' << directBlocks.Extent() << ' ' << directVertices.Extent();
  for (int vertex : vertices) std::cout << ' ' << vertex;
  std::cout << '\n';
  std::vector<int> cached(info.VerticesIn().cbegin(), info.VerticesIn().cend());
  std::sort(cached.begin(), cached.end());
  std::cout << "cached " << info.PaveBlocksIn().Extent() << ' ' << info.VerticesIn().Extent();
  for (int vertex : cached) std::cout << ' ' << vertex;
  std::cout << '\n';
  for (auto it = info.PaveBlocksIn().cbegin(); it != info.PaveBlocksIn().cend(); ++it)
    std::cout << "inside-pb " << (*it)->OriginalEdge() << '\n';
  ds.AddShapeSD(alias2Index, vfIndex);
  ds.UpdateFaceInfoIn(faceIndex);
  std::vector<int> refreshed(info.VerticesIn().cbegin(), info.VerticesIn().cend());
  std::sort(refreshed.begin(), refreshed.end());
  std::cout << "refreshed " << info.PaveBlocksIn().Extent() << ' ' << info.VerticesIn().Extent();
  for (int vertex : refreshed) std::cout << ' ' << vertex;
  std::cout << '\n';
}
