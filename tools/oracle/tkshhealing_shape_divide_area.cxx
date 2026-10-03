#include <ShapeUpgrade_FaceDivideArea.hxx>
#include <ShapeUpgrade_ShapeDivideArea.hxx>
#include <iomanip>
#include <iostream>

class ShapeDivideAreaProbe : public ShapeUpgrade_ShapeDivideArea
{
public:
  using ShapeUpgrade_ShapeDivideArea::GetSplitFaceTool;
};

static void observe(const char* theName, ShapeDivideAreaProbe& theDivider)
{
  const occ::handle<ShapeUpgrade_FaceDivide> aFirst = theDivider.GetSplitFaceTool();
  const occ::handle<ShapeUpgrade_FaceDivide> aSecond = theDivider.GetSplitFaceTool();
  const occ::handle<ShapeUpgrade_FaceDivideArea> anAreaTool =
      occ::down_cast<ShapeUpgrade_FaceDivideArea>(aFirst);
  std::cout << theName << ' ' << std::setprecision(17) << theDivider.MaxArea() << ' '
            << theDivider.NbParts() << ' ' << anAreaTool->MaxArea() << ' '
            << anAreaTool->NbParts() << ' ' << (aFirst == aSecond) << '\n';
}

int main()
{
  ShapeDivideAreaProbe aDefault;
  observe("default", aDefault);
  aDefault.MaxArea() = 12.5;
  aDefault.NbParts() = 7;
  aDefault.SetSplittingByNumber(true);
  aDefault.SetSplittingByNumber(false);
  observe("mode_toggle", aDefault);
  aDefault.SetNumbersUVSplits(2, 3);
  observe("uv_positive", aDefault);
  aDefault.SetNumbersUVSplits(0, 4);
  observe("uv_nonpositive", aDefault);
}
