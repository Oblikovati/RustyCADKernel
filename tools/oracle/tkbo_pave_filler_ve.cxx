#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <TopoDS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Circle.hxx>
#include <IntTools_Context.hxx>
#include <Bnd_Box.hxx>
#include <BRep_Builder.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <iostream>
#include <cmath>
#include <iomanip>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void PerformVertexEdge() { Init(Message_ProgressRange()); PerformVE(Message_ProgressRange()); }
  void InitializeOnly() { Init(Message_ProgressRange()); }
  void IntersectVertexEdge() { PerformVE(Message_ProgressRange()); }
  int ProjectPoint(const gp_Pnt& point, const TopoDS_Edge& edge, double& parameter, double& distance) {
    return myContext->ComputePE(point, 3.0e-6, edge, parameter, distance);
  }
  BOPDS_DS& DS() { return *myDS; }
};

static TopoDS_Compound vertex_argument(double x, double y = 0.0) {
  TopoDS_Compound compound;
  TopoDS_Vertex vertex;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.MakeVertex(vertex, gp_Pnt(x, y, 0), 1.0e-7);
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

static TopoDS_Edge shared_endpoint_edge(const TopoDS_Vertex& first, const TopoDS_Vertex& last) {
  return BRepBuilderAPI_MakeEdge(first, last).Edge();
}

static TopoDS_Compound edge_argument(const TopoDS_Edge& edge) {
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, edge);
  return compound;
}

static TopoDS_Compound closed_edge_argument(double radius) {
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), radius);
  const occ::handle<Geom_Circle> geometry = new Geom_Circle(circle);
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, BRepBuilderAPI_MakeEdge(geometry, 0.0, 6.283185307179586).Edge());
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
  Probe unify;
  unify.AddArgument(vertex_argument(5.0e-7, 2.0e-7));
  unify.AddArgument(edge_argument(10.0));
  unify.SetFuzzyValue(1.0e-6);
  unify.PerformVertexEdge();
  auto& unifyDS = unify.DS();
  int representative = -1;
  bool hasRepresentative = unifyDS.HasShapeSD(1, representative);
  std::cout << "unify " << unifyDS.NbShapes() << ' '
            << unifyDS.InterfVE().Length() << ' ' << unifyDS.InterfVV().Length() << ' '
            << unifyDS.PaveBlocks(3).Extent() << ' ' << hasRepresentative << ' '
            << representative << '\n';
  const auto& vv = unifyDS.InterfVV().Value(0);
  std::cout << "unify-vv " << vv.Index1() << ' ' << vv.Index2() << ' '
            << vv.IndexNew() << '\n';
  const auto& unifiedBlocks = unifyDS.PaveBlocks(3);
  for (auto it = unifiedBlocks.cbegin(); it != unifiedBlocks.cend(); ++it) {
    int firstIndex, lastIndex;
    double first, last;
    (*it)->Indices(firstIndex, lastIndex);
    (*it)->Range(first, last);
    std::cout << "unify-pb " << firstIndex << ' ' << lastIndex << ' '
              << first << ' ' << last << '\n';
  }
  Probe common;
  common.AddArgument(vertex_argument(5.0, 2.0e-7));
  TopoDS_Vertex start, end;
  BRep_Builder builder;
  builder.MakeVertex(start, gp_Pnt(0, 0, 0), 1.0e-7);
  builder.MakeVertex(end, gp_Pnt(10, 0, 0), 1.0e-7);
  common.AddArgument(edge_argument(shared_endpoint_edge(start, end)));
  common.AddArgument(edge_argument(shared_endpoint_edge(start, end)));
  common.SetFuzzyValue(1.0e-6);
  common.InitializeOnly();
  auto& commonDS = common.DS();
  NCollection_List<occ::handle<BOPDS_PaveBlock>> originalCommonBlocks;
  originalCommonBlocks.Append(commonDS.ChangePaveBlocks(3).First());
  originalCommonBlocks.Append(commonDS.ChangePaveBlocks(7).First());
  occ::handle<BOPDS_CommonBlock> oldCommon = new BOPDS_CommonBlock;
  oldCommon->SetPaveBlocks(originalCommonBlocks);
  commonDS.SetCommonBlock(originalCommonBlocks.First(), oldCommon);
  commonDS.SetCommonBlock(originalCommonBlocks.Last(), oldCommon);
  common.IntersectVertexEdge();
  std::cout << "common " << commonDS.NbShapes() << ' ' << commonDS.InterfVE().Length() << '\n';
  for (int edgeIndex : {3, 7}) {
    const auto& edgeBlocks = commonDS.PaveBlocks(edgeIndex);
    std::cout << "common-edge " << edgeIndex << ' ' << edgeBlocks.Extent() << '\n';
    for (auto it = edgeBlocks.cbegin(); it != edgeBlocks.cend(); ++it) {
      int firstIndex, lastIndex;
      double first, last;
      (*it)->Indices(firstIndex, lastIndex);
      (*it)->Range(first, last);
      const auto commonBlock = commonDS.CommonBlock(*it);
      std::cout << "common-pb " << firstIndex << ' ' << lastIndex << ' '
                << first << ' ' << last << ' '
                << (commonBlock.IsNull() ? 0 : commonBlock->PaveBlocks().Extent()) << '\n';
    }
  }
  Probe closed;
  closed.AddArgument(vertex_argument(0.0, 10.0 + 2.0e-7));
  closed.AddArgument(closed_edge_argument(10.0));
  closed.AddArgument(closed_edge_argument(10.0));
  closed.SetFuzzyValue(1.0e-6);
  closed.InitializeOnly();
  auto& closedDS = closed.DS();
  NCollection_List<occ::handle<BOPDS_PaveBlock>> closedOriginals;
  closedOriginals.Append(closedDS.ChangePaveBlocks(3).First());
  closedOriginals.Append(closedDS.ChangePaveBlocks(6).First());
  occ::handle<BOPDS_CommonBlock> oldClosed = new BOPDS_CommonBlock;
  oldClosed->SetPaveBlocks(closedOriginals);
  closedDS.SetCommonBlock(closedOriginals.First(), oldClosed);
  closedDS.SetCommonBlock(closedOriginals.Last(), oldClosed);
  for (auto it = closedOriginals.cbegin(); it != closedOriginals.cend(); ++it) {
    int firstIndex, lastIndex;
    (*it)->Indices(firstIndex, lastIndex);
    std::cout << "closed-source " << firstIndex << ' ' << lastIndex << '\n';
  }
  double projectedParameter = -1.0, projectedDistance = -1.0;
  TopoDS_Edge secondClosedEdge = TopoDS::Edge(closedDS.Shape(6));
  const int projectedStatus = closed.ProjectPoint(
    gp_Pnt(10.0 / std::sqrt(2.0), 10.0 / std::sqrt(2.0), 0.0), secondClosedEdge,
    projectedParameter, projectedDistance);
  std::cout << "closed-projection " << projectedStatus << ' ' << projectedParameter << ' '
            << projectedDistance << '\n';
  closed.IntersectVertexEdge();
  std::cout << "closed " << closedDS.NbShapes() << ' ' << closedDS.InterfVE().Length() << '\n';
  for (int edgeIndex : {3, 6}) {
    const auto& edgeBlocks = closedDS.PaveBlocks(edgeIndex);
    std::cout << "closed-edge " << edgeIndex << ' ' << edgeBlocks.Extent() << '\n';
    for (auto it = edgeBlocks.cbegin(); it != edgeBlocks.cend(); ++it) {
      int firstIndex, lastIndex;
      double first, last;
      (*it)->Indices(firstIndex, lastIndex);
      (*it)->Range(first, last);
      const auto commonBlock = closedDS.CommonBlock(*it);
      std::cout << "closed-pb " << firstIndex << ' ' << lastIndex << ' '
                << first << ' ' << last << ' '
                << (commonBlock.IsNull() ? 0 : commonBlock->PaveBlocks().Extent()) << '\n';
    }
  }
}
