#include <BRepPrimAPI_MakeCylinder.hxx>
#include <ShapeUpgrade_ClosedFaceDivide.hxx>
#include <ShapeUpgrade_ShapeDivideClosed.hxx>
#include <ShapeUpgrade_WireDivide.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Shape.hxx>

#include <iostream>

class ShapeDivideClosedProbe : public ShapeUpgrade_ShapeDivideClosed
{
public:
  explicit ShapeDivideClosedProbe(const TopoDS_Shape& shape) : ShapeUpgrade_ShapeDivideClosed(shape) {}

  occ::handle<ShapeUpgrade_FaceDivide> splitFaceTool() const
  {
    return GetSplitFaceTool();
  }
};

static int count(const TopoDS_Shape& shape, const TopAbs_ShapeEnum kind)
{
  int result = 0;
  for (TopExp_Explorer explorer(shape, kind); explorer.More(); explorer.Next())
  {
    ++result;
  }
  return result;
}

static void observe(const int cuts)
{
  const TopoDS_Shape source = BRepPrimAPI_MakeCylinder(2.0, 5.0).Face();
  ShapeDivideClosedProbe divider(source);
  if (cuts != 1)
  {
    divider.SetNbSplitPoints(cuts);
  }
  const bool changed = divider.Perform();
  const TopoDS_Shape result = divider.Result();
  const occ::handle<ShapeUpgrade_ClosedFaceDivide> faceTool =
    occ::handle<ShapeUpgrade_ClosedFaceDivide>::DownCast(divider.splitFaceTool());
  std::cout << cuts << ' ' << changed << ' ' << count(result, TopAbs_FACE) << ' '
            << count(result, TopAbs_EDGE) << ' ' << divider.Status(ShapeExtend_DONE1) << ' '
            << divider.Status(ShapeExtend_DONE2) << ' ' << divider.Status(ShapeExtend_FAIL1) << ' '
            << divider.Status(ShapeExtend_FAIL2) << ' ' << divider.Status(ShapeExtend_FAIL3) << ' '
            << faceTool->GetNbSplitPoints() << ' ' << faceTool->GetWireDivideTool().IsNull() << '\n';
}

int main()
{
  observe(0);
  observe(1);
  observe(2);
  observe(3);
}
