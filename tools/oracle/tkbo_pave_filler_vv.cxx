#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS.hxx>
#include <gp_Pnt.hxx>
#include <iostream>
#include <iomanip>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void RunVV() { Init(Message_ProgressRange()); PerformVV(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};
static TopoDS_Compound argument(double x) {
  TopoDS_Compound result;
  TopoDS_Vertex vertex;
  BRep_Builder builder;
  builder.MakeCompound(result);
  builder.MakeVertex(vertex, gp_Pnt(x, 0, 0), 1.0e-7);
  builder.Add(result, vertex);
  return result;
}
int main() {
  Probe filler;
  filler.AddArgument(argument(0));
  filler.AddArgument(argument(5.0e-7));
  filler.SetFuzzyValue(1.0e-6);
  filler.RunVV();
  auto& ds = filler.DS();
  std::cout << ds.NbShapes() << ' ' << ds.InterfVV().Length() << ' '
            << ds.ShapesSD().Extent() << '\n';
  const gp_Pnt& point = BRep_Tool::Pnt(TopoDS::Vertex(ds.Shape(4)));
  std::cout << std::fixed << std::setprecision(8) << point.X() << ' '
            << BRep_Tool::Tolerance(TopoDS::Vertex(ds.Shape(4))) << '\n';
  for (int i = 0; i < ds.InterfVV().Length(); ++i) {
    const auto& interf = ds.InterfVV().Value(i);
    std::cout << interf.Index1() << ' ' << interf.Index2() << ' '
              << interf.IndexNew() << '\n';
  }
  NCollection_DataMap<int, int>::Iterator it(ds.ShapesSD());
  for (; it.More(); it.Next())
    std::cout << it.Key() << ' ' << it.Value() << '\n';

  Probe separate;
  separate.AddArgument(argument(0));
  separate.AddArgument(argument(1.0e-2));
  separate.RunVV();
  std::cout << "separate " << separate.DS().NbShapes() << ' '
            << separate.DS().InterfVV().Length() << ' '
            << separate.DS().ShapesSD().Extent() << '\n';
}
