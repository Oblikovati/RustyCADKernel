#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>
#include <iostream>
#include <iomanip>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void FillRanges() { Init(Message_ProgressRange()); FillShrunkData(TopAbs_VERTEX, TopAbs_EDGE); }
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
static TopoDS_Compound edge_argument() {
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)).Edge());
  return compound;
}
int main() {
  Probe filler;
  filler.AddArgument(vertex_argument(0.5));
  filler.AddArgument(edge_argument());
  filler.SetFuzzyValue(1.0e-6);
  filler.FillRanges();
  const auto& blocks = filler.DS().PaveBlocks(3);
  std::cout << filler.DS().NbShapes() << ' ' << blocks.Extent() << '\n';
  const auto& block = blocks.First();
  double first, last, xmin, ymin, zmin, xmax, ymax, zmax;
  Bnd_Box bounds;
  bool splittable;
  block->ShrunkData(first, last, bounds, splittable);
  bounds.Get(xmin, ymin, zmin, xmax, ymax, zmax);
  std::cout << std::fixed << std::setprecision(8)
            << first << ' ' << last << ' ' << splittable << ' ' << bounds.GetGap() << '\n'
            << xmin << ' ' << ymin << ' ' << zmin << ' '
            << xmax << ' ' << ymax << ' ' << zmax << '\n';
}
