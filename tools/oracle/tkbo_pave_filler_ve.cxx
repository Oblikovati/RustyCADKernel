#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Bnd_Box.hxx>
#include <BRep_Builder.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>
#include <iostream>
#include <iomanip>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void PerformVertexEdge() { Init(Message_ProgressRange()); PerformVE(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

static TopoDS_Compound vertex_argument(double x) {
  TopoDS_Compound compound;
  TopoDS_Vertex vertex;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.MakeVertex(vertex, gp_Pnt(x, 0, 0), 1.0e-7);
  builder.Add(compound, vertex);
  return compound;
}

static TopoDS_Compound edge_argument(double length = 1.0) {
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(length, 0, 0)).Edge());
  return compound;
}

int main() {
  Probe filler;
  filler.AddArgument(vertex_argument(0.5));
  filler.AddArgument(edge_argument());
  filler.SetFuzzyValue(1.0e-6);
  filler.PerformVertexEdge();
  auto& ds = filler.DS();
  std::cout << ds.NbShapes() << ' ' << ds.InterfVE().Length() << ' '
            << ds.PaveBlocks(3).Extent() << '\n' << std::fixed << std::setprecision(8);
  for (int i = 0; i < ds.InterfVE().Length(); ++i) {
    const auto& interf = ds.InterfVE().Value(i);
    std::cout << "ve " << interf.Index1() << ' ' << interf.Index2() << ' '
              << interf.Parameter() << ' ' << interf.IndexNew() << '\n';
  }
  const auto& blocks = ds.PaveBlocks(3);
  for (auto it = blocks.cbegin(); it != blocks.cend(); ++it) {
    double first, last, shrunkFirst, shrunkLast;
    bool splittable;
    Bnd_Box bounds;
    (*it)->Range(first, last);
    (*it)->ShrunkData(shrunkFirst, shrunkLast, bounds, splittable);
    int firstIndex, lastIndex;
    (*it)->Indices(firstIndex, lastIndex);
    std::cout << "pb " << firstIndex << ' ' << lastIndex << ' '
              << first << ' ' << last << ' ' << shrunkFirst << ' ' << shrunkLast << ' '
              << splittable << ' ' << (*it)->ExtPaves().Extent() << '\n';
  }
  Probe nonDestructive;
  nonDestructive.AddArgument(vertex_argument(0.5));
  nonDestructive.AddArgument(edge_argument());
  nonDestructive.SetFuzzyValue(1.0e-6);
  nonDestructive.SetNonDestructive(true);
  nonDestructive.PerformVertexEdge();
  auto& dsND = nonDestructive.DS();
  std::cout << "non-destructive " << dsND.NbShapes() << ' '
            << dsND.InterfVE().Length() << ' ' << dsND.PaveBlocks(3).Extent() << '\n';
  const auto& interfND = dsND.InterfVE().Value(0);
  std::cout << "ve-nd " << interfND.Index1() << ' ' << interfND.Index2() << ' '
            << interfND.Parameter() << ' ' << interfND.IndexNew() << '\n';
  double xmin, ymin, zmin, xmax, ymax, zmax;
  Bnd_Box vertexBounds = dsND.ShapeInfo(interfND.IndexNew()).Box();
  vertexBounds.Get(xmin, ymin, zmin, xmax, ymax, zmax);
  std::cout << std::fixed << std::setprecision(8)
            << "box-nd " << vertexBounds.GetGap() << ' '
            << xmin << ' ' << ymin << ' ' << zmin << ' '
            << xmax << ' ' << ymax << ' ' << zmax << '\n';
  const auto& blocksND = dsND.PaveBlocks(3);
  for (auto it = blocksND.cbegin(); it != blocksND.cend(); ++it) {
    int firstIndex, lastIndex;
    double first, last;
    (*it)->Indices(firstIndex, lastIndex);
    (*it)->Range(first, last);
    std::cout << "pb-nd " << firstIndex << ' ' << lastIndex << ' '
              << first << ' ' << last << '\n';
  }
  Probe micro;
  micro.AddArgument(vertex_argument(3.5e-7));
  micro.AddArgument(edge_argument(7.0e-7));
  micro.SetFuzzyValue(1.0e-8);
  micro.PerformVertexEdge();
  auto& microDS = micro.DS();
  const auto& microBlock = microDS.PaveBlocks(3).First();
  std::cout << "micro " << microDS.InterfVE().Length() << ' ';
  double microFirst, microLast;
  bool microSplittable;
  Bnd_Box microBounds;
  microBlock->ShrunkData(microFirst, microLast, microBounds, microSplittable);
  std::cout << microSplittable << ' ' << microFirst << ' ' << microLast << '\n';
}
