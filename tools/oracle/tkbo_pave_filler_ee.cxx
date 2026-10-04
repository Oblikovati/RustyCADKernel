#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>
#include <iomanip>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void Run() { Init(Message_ProgressRange()); PerformEE(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
  void PrintVerticesToAvoidExtension() const {
    std::cout << "endpoint-avoid";
    for (auto it = myVertsToAvoidExtension.cbegin(); it != myVertsToAvoidExtension.cend(); ++it) {
      std::cout << ' ' << *it;
    }
    std::cout << '\n';
  }
};

static TopoDS_Compound edge_argument(const gp_Pnt& first, const gp_Pnt& last) {
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, BRepBuilderAPI_MakeEdge(first, last).Edge());
  return compound;
}

static TopoDS_Compound edge_argument(const TopoDS_Edge& edge) {
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, edge);
  return compound;
}

int main() {
  Probe filler;
  filler.AddArgument(edge_argument(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)));
  filler.AddArgument(edge_argument(gp_Pnt(0, -1, 0), gp_Pnt(0, 1, 0)));
  filler.SetFuzzyValue(1.0e-6);
  filler.Run();
  auto& ds = filler.DS();
  std::cout << "ee " << ds.NbShapes() << ' ' << ds.InterfEE().Length() << '\n' << std::fixed << std::setprecision(8);
  for (int i = 0; i < ds.InterfEE().Length(); ++i) {
    const auto& record = ds.InterfEE().Value(i);
    const auto& part = record.CommonPart();
    std::cout << "record " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << part.Type() << ' '
              << part.VertexParameter1() << ' ' << part.VertexParameter2() << '\n';
  }
  for (int edge : {1, 5}) {
    const auto& blocks = ds.PaveBlocks(edge);
    std::cout << "edge " << edge << ' ' << blocks.Extent() << '\n';
    for (auto it = blocks.cbegin(); it != blocks.cend(); ++it) {
      int v1, v2;
      double t1, t2;
      (*it)->Indices(v1, v2);
      (*it)->Range(t1, t2);
      std::cout << "pb " << v1 << ' ' << v2 << ' ' << t1 << ' ' << t2 << '\n';
    }
  }
  Probe multi;
  multi.AddArgument(edge_argument(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)));
  multi.AddArgument(edge_argument(gp_Pnt(0, -1, 0), gp_Pnt(0, 1, 0)));
  multi.AddArgument(edge_argument(gp_Pnt(-1, -1, 0), gp_Pnt(1, 1, 0)));
  multi.SetFuzzyValue(1.0e-6);
  multi.Run();
  auto& multiDS = multi.DS();
  std::cout << "multi " << multiDS.NbShapes() << ' ' << multiDS.InterfEE().Length() << '\n';
  for (int i = 0; i < multiDS.InterfEE().Length(); ++i) {
    const auto& record = multiDS.InterfEE().Value(i);
    std::cout << "multi-record " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << record.CommonPart().VertexParameter1() << ' '
              << record.CommonPart().VertexParameter2() << '\n';
  }
  for (int i = 0; i < multiDS.NbShapes(); ++i) {
    if (multiDS.ShapeInfo(i).ShapeType() != TopAbs_EDGE) continue;
    const auto& blocks = multiDS.PaveBlocks(i);
    std::cout << "multi-edge " << i << ' ' << blocks.Extent() << '\n';
    for (auto it = blocks.cbegin(); it != blocks.cend(); ++it) {
      int firstIndex, lastIndex;
      double first, last;
      (*it)->Indices(firstIndex, lastIndex);
      (*it)->Range(first, last);
      std::cout << "multi-pb " << firstIndex << ' ' << lastIndex << ' '
                << first << ' ' << last << '\n';
    }
  }
  Probe endpoint;
  endpoint.AddArgument(edge_argument(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0)));
  endpoint.AddArgument(edge_argument(gp_Pnt(0, -1, 0), gp_Pnt(0, 1, 0)));
  endpoint.SetFuzzyValue(1.0e-6);
  endpoint.Run();
  auto& endpointDS = endpoint.DS();
  std::cout << "endpoint " << endpointDS.NbShapes() << ' '
            << endpointDS.InterfEE().Length() << ' ' << endpointDS.InterfVE().Length() << '\n';
  for (int i = 0; i < endpointDS.InterfEE().Length(); ++i) {
    const auto& record = endpointDS.InterfEE().Value(i);
    std::cout << "endpoint-ee " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << record.CommonPart().VertexParameter1() << ' '
              << record.CommonPart().VertexParameter2() << '\n';
  }
  for (int i = 0; i < endpointDS.InterfVE().Length(); ++i) {
    const auto& record = endpointDS.InterfVE().Value(i);
    std::cout << "endpoint-ve " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << record.Parameter() << '\n';
  }
  for (int edge : {1, 5}) {
    const auto& blocks = endpointDS.PaveBlocks(edge);
    std::cout << "endpoint-edge " << edge << ' ' << blocks.Extent() << '\n';
    for (auto it = blocks.cbegin(); it != blocks.cend(); ++it) {
      int firstIndex, lastIndex;
      double first, last;
      (*it)->Indices(firstIndex, lastIndex);
      (*it)->Range(first, last);
      std::cout << "endpoint-pb " << firstIndex << ' ' << lastIndex << ' '
                << first << ' ' << last << '\n';
    }
  }
  endpoint.PrintVerticesToAvoidExtension();
  Probe overlap;
  BRep_Builder builder;
  TopoDS_Vertex start, end;
  builder.MakeVertex(start, gp_Pnt(0, 0, 0), 1.0e-7);
  builder.MakeVertex(end, gp_Pnt(2, 0, 0), 1.0e-7);
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(start, end).Edge();
  const TopoDS_Edge second = BRepBuilderAPI_MakeEdge(start, end).Edge();
  overlap.AddArgument(edge_argument(first));
  overlap.AddArgument(edge_argument(second));
  overlap.SetFuzzyValue(1.0e-6);
  overlap.Run();
  auto& overlapDS = overlap.DS();
  std::cout << "overlap " << overlapDS.NbShapes() << ' ' << overlapDS.InterfEE().Length() << '\n';
  for (int i = 0; i < overlapDS.InterfEE().Length(); ++i) {
    const auto& record = overlapDS.InterfEE().Value(i);
    std::cout << "overlap-record " << record.Index1() << ' ' << record.Index2() << ' '
              << record.CommonPart().Type() << '\n';
  }
  for (int edge : {1, 5}) {
    const auto& blocks = overlapDS.PaveBlocks(edge);
    std::cout << "overlap-edge " << edge << ' ' << blocks.Extent() << '\n';
    for (auto it = blocks.cbegin(); it != blocks.cend(); ++it) {
      int firstIndex, lastIndex;
      (*it)->Indices(firstIndex, lastIndex);
      const auto commonBlock = overlapDS.CommonBlock(*it);
      std::cout << "overlap-pb " << firstIndex << ' ' << lastIndex << ' '
                << (commonBlock.IsNull() ? 0 : commonBlock->PaveBlocks().Extent()) << '\n';
    }
  }
}
