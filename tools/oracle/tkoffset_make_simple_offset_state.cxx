#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepOffset_MakeSimpleOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Failure.hxx>
#include <Standard_NoSuchObject.hxx>
#include <gp.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pln.hxx>

#include <iomanip>
#include <iostream>

namespace
{
void writeState(const char* label, const BRepOffset_MakeSimpleOffset& maker)
{
  const TopoDS_Shape& result = maker.GetResultShape();
  std::cout << "case=" << label << " done=" << maker.IsDone() << " error="
            << static_cast<int>(maker.GetError()) << " message=";
  const TCollection_AsciiString errorMessage = maker.GetErrorMessage();
  const char* message = errorMessage.ToCString();
  std::cout << (message[0] == '\0' ? "<empty>" : message);
  std::cout << " build_solid=" << maker.GetBuildSolidFlag() << " offset="
            << maker.GetOffsetValue() << " tolerance=" << maker.GetTolerance()
            << " result_null=" << result.IsNull();
  if (!result.IsNull()) std::cout << " result_type=" << static_cast<int>(result.ShapeType());
  std::cout << '\n';
}
} // namespace

int main()
{
  std::cout << std::setprecision(17);
  BRepOffset_MakeSimpleOffset maker;
  writeState("default", maker);
  try
  {
    maker.Generated(TopoDS_Shape());
  }
  catch (Standard_NoSuchObject const& failure)
  {
    std::cout << "case=generated_before_perform error=Standard_NoSuchObject message="
              << failure.what() << '\n';
  }
  catch (Standard_Failure const& failure)
  {
    std::cout << "case=generated_before_perform error=Standard_Failure message="
              << failure.what() << '\n';
  }
  maker.Perform();
  writeState("null_perform", maker);

  BRepBuilderAPI_MakeFace faceMaker(
    gp_Pln(gp_Ax3(gp::Origin(), gp::DZ(), gp::DX())), 0.0, 1.0, 0.0, 2.0);
  const TopoDS_Shape source = faceMaker.Face();
  maker.Initialize(source, 1.5);
  maker.SetTolerance(2.0e-6);
  maker.Perform();
  writeState("plane_perform", maker);
  const TopoDS_Shape generated = maker.Generated(source);
  const TopoDS_Shape modified = maker.Modified(source);
  std::cout << "case=history generated_null=" << generated.IsNull()
            << " generated_same_as_source=" << generated.IsSame(source)
            << " modified_null=" << modified.IsNull() << '\n';

  maker.Initialize(TopoDS_Shape(), 3.0);
  writeState("reinitialized_null", maker);
  std::cout << "case=history_after_initialize generated_same_as_previous="
            << maker.Generated(source).IsSame(generated) << '\n';
  maker.Perform();
  writeState("reinitialized_null_perform", maker);

  BRepOffset_MakeSimpleOffset solidMaker(source, 1.5);
  solidMaker.SetBuildSolidFlag(true);
  solidMaker.SetTolerance(2.0e-6);
  solidMaker.Perform();
  writeState("plane_solid_perform", solidMaker);
  GProp_GProps properties;
  BRepGProp::VolumeProperties(solidMaker.GetResultShape(), properties);
  std::cout << "case=plane_solid_metrics valid="
            << BRepCheck_Analyzer(solidMaker.GetResultShape()).IsValid()
            << " volume=" << properties.Mass() << '\n';

  const TopoDS_Shape box = BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape();
  BRepOffset_MakeSimpleOffset closedSolidMaker(box, 0.1);
  closedSolidMaker.SetBuildSolidFlag(true);
  closedSolidMaker.Perform();
  writeState("box_solid_perform", closedSolidMaker);
}
