#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Map.hxx>
#include <algorithm>
#include <iostream>
#include <vector>

int main() {
  TopoDS_Shape box = BRepPrimAPI_MakeBox(1.0, 2.0, 3.0).Shape();
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(box);
  BOPDS_DS ds;
  ds.SetArguments(arguments);
  ds.Init(1.0e-7);
  int face = -1;
  std::vector<int> vertices;
  std::vector<int> edges;
  for (int index = 0; index < ds.NbSourceShapes(); ++index) {
    const BOPDS_ShapeInfo& info = ds.ShapeInfo(index);
    if (face < 0 && info.ShapeType() == TopAbs_FACE) face = index;
    if (info.ShapeType() == TopAbs_VERTEX) vertices.push_back(index);
  }
  for (int index : ds.ShapeInfo(face).SubShapes()) {
    if (ds.ShapeInfo(index).ShapeType() != TopAbs_EDGE) continue;
    edges.push_back(index);
    ds.ChangePaveBlocks(index);
  }
  const int firstVertex = vertices[0];
  const int wrongVertex = vertices[1];
  const int generatedVertex = vertices[2];
  auto& vfWrong = ds.InterfVF().Appended();
  vfWrong.SetIndices(face, wrongVertex);
  auto& vfRight = ds.InterfVF().Appended();
  vfRight.SetIndices(firstVertex, face);
  auto& efWrong = ds.InterfEF().Appended();
  efWrong.SetIndices(face, edges[0]);
  efWrong.SetIndexNew(wrongVertex);
  auto& efRight = ds.InterfEF().Appended();
  efRight.SetIndices(edges[0], face);
  efRight.SetIndexNew(generatedVertex);
  NCollection_Map<int> faces;
  faces.Add(face);
  ds.ChangePaveBlocks(edges[0]).First()->SetEdge(edges[0]);
  ds.UpdateFaceInfoOn(faces);
  const BOPDS_FaceInfo& on = ds.FaceInfo(face);
  std::cout << "on " << on.PaveBlocksOn().Extent() << ' ' << on.VerticesOn().Extent()
            << ' ' << on.VerticesIn().Extent() << '\n';
  auto& efCommon = ds.InterfEF().Appended();
  efCommon.SetIndices(edges[0], face);
  const occ::handle<BOPDS_PaveBlock> block = ds.PaveBlocks(edges[0]).First();
  occ::handle<BOPDS_CommonBlock> common = new BOPDS_CommonBlock;
  NCollection_List<occ::handle<BOPDS_PaveBlock>> commonBlocks;
  commonBlocks.Append(block);
  common->SetPaveBlocks(commonBlocks);
  NCollection_List<int> commonFaces;
  commonFaces.Append(face);
  common->SetFaces(commonFaces);
  ds.SetCommonBlock(block, common);
  ds.UpdateFaceInfoIn(faces);
  const BOPDS_FaceInfo& in = ds.FaceInfo(face);
  std::cout << "in " << in.PaveBlocksIn().Extent() << ' ' << in.VerticesIn().Extent()
            << ' ' << in.VerticesIn().Contains(firstVertex) << ' '
            << in.VerticesIn().Contains(generatedVertex) << ' '
            << in.VerticesIn().Contains(wrongVertex) << '\n';
  ds.RefineFaceInfoOn();
  const BOPDS_FaceInfo& refined = ds.FaceInfo(face);
  std::cout << "refined " << refined.PaveBlocksOn().Extent() << ' '
            << refined.VerticesOn().Extent() << '\n';
}
