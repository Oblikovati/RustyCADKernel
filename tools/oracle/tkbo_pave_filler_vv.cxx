#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_ShapeInfo.hxx>
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
  void InitDS() { Init(Message_ProgressRange()); }
  void RunVV() { PerformVV(Message_ProgressRange()); }
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
  filler.InitDS();
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
  separate.InitDS();
  separate.RunVV();
  std::cout << "separate " << separate.DS().NbShapes() << ' '
            << separate.DS().InterfVV().Length() << ' '
            << separate.DS().ShapesSD().Extent() << '\n';

  Probe chain;
  chain.AddArgument(argument(0));
  chain.AddArgument(argument(5.0e-7));
  chain.AddArgument(argument(1.0e-6));
  chain.SetFuzzyValue(1.0e-6);
  chain.InitDS();
  chain.RunVV();
  auto& chainDS = chain.DS();
  std::cout << "chain " << chainDS.NbShapes() << ' '
            << chainDS.InterfVV().Length() << ' '
            << chainDS.ShapesSD().Extent() << '\n';
  for (int index = 0; index < chainDS.InterfVV().Length(); ++index) {
    const auto& interf = chainDS.InterfVV().Value(index);
    std::cout << interf.Index1() << ' ' << interf.Index2() << ' '
              << interf.IndexNew() << '\n';
  }
  chainDS.ChangeShapeInfo(6).ChangeBox().Add(gp_Pnt(10, 0, 0));
  chain.RunVV();
  const gp_Pnt& chainPoint = BRep_Tool::Pnt(TopoDS::Vertex(chainDS.Shape(6)));
  std::cout << "chain-repeat " << chainDS.NbShapes() << ' '
            << chainDS.InterfVV().Length() << ' ' << chainDS.ShapesSD().Extent()
            << ' ' << std::fixed << std::setprecision(8) << chainPoint.X() << ' '
            << BRep_Tool::Tolerance(TopoDS::Vertex(chainDS.Shape(6))) << '\n';
  double xmin, ymin, zmin, xmax, ymax, zmax;
  chainDS.ShapeInfo(6).Box().Get(xmin, ymin, zmin, xmax, ymax, zmax);
  std::cout << "chain-box " << std::fixed << std::setprecision(8)
            << xmin << ' ' << ymin << ' ' << zmin << ' '
            << xmax << ' ' << ymax << ' ' << zmax << '\n';
}
