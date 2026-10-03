#include <BRepPrimAPI_MakeBox.hxx>
#include <ShapeFix_FixSmallFace.hxx>

#include <iomanip>
#include <iostream>

static void probe(const char* name, const TopoDS_Shape& input) {
  ShapeFix_FixSmallFace fixer;
  fixer.Init(input);
  const TopoDS_Shape result = fixer.FixSpotFace();
  std::cout << name << ' ' << result.IsSame(input) << ' ' << fixer.Shape().IsSame(input) << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  probe("regular_solid", BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape());
}
