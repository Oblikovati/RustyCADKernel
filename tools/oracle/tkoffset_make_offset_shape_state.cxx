#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <Standard_Failure.hxx>
#include <Standard_NoSuchObject.hxx>
#include <gp.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pln.hxx>

#include <iostream>

void writeGenerated(const char* label, BRepOffsetAPI_MakeOffsetShape& maker,
                    const TopoDS_Shape& input)
{
  try
  {
    const int count = maker.Generated(input).Size();
    std::cout << label << "_generated=" << count << ' ';
  }
  catch (const Standard_NoSuchObject&)
  {
    std::cout << label << "_generated_error=Standard_NoSuchObject ";
  }
  catch (const Standard_Failure& failure)
  {
    std::cout << label << "_generated_error=" << failure.what() << ' ';
  }
}

void writeModified(const char* label, BRepOffsetAPI_MakeOffsetShape& maker,
                   const TopoDS_Shape& input)
{
  try
  {
    const int count = maker.Modified(input).Size();
    std::cout << label << "_modified=" << count << ' ';
  }
  catch (const Standard_NoSuchObject&)
  {
    std::cout << label << "_modified_error=Standard_NoSuchObject ";
  }
  catch (const Standard_Failure& failure)
  {
    std::cout << label << "_modified_error=" << failure.what() << ' ';
  }
}

void writeHistory(const char* label, BRepOffsetAPI_MakeOffsetShape& maker,
                  const TopoDS_Shape& input)
{
  writeGenerated(label, maker, input);
  writeModified(label, maker, input);
  std::cout << label << "_deleted=" << maker.IsDeleted(input) << '\n';
}

void writeDefaultAndNull(BRepOffsetAPI_MakeOffsetShape& maker)
{
  std::cout << "default_done=" << maker.IsDone() << ' ';
  writeHistory("default", maker, TopoDS_Shape());
  maker.Build();
  std::cout << "default_build_done=" << maker.IsDone() << '\n';
  maker.PerformBySimple(TopoDS_Shape(), 1.0);
  std::cout << "null_done=" << maker.IsDone() << '\n';
  writeHistory("null", maker, TopoDS_Shape());
}

TopoDS_Shape makePlaneFace()
{
  BRepBuilderAPI_MakeFace faceMaker(
    gp_Pln(gp_Ax3(gp::Origin(), gp::DZ(), gp::DX())), 0.0, 1.0, 0.0, 2.0);
  return faceMaker.Face();
}

void writePlaneRun(BRepOffsetAPI_MakeOffsetShape& maker, const TopoDS_Shape& face)
{
  maker.PerformBySimple(face, 1.5);
  std::cout << "plane_done=" << maker.IsDone() << " plane_result_null=" << maker.Shape().IsNull()
            << '\n';
  writeHistory("plane", maker, face);
  maker.Build();
  std::cout << "plane_build_done=" << maker.IsDone() << " plane_build_result_null="
            << maker.Shape().IsNull() << '\n';
}

void writeFailedRedo(BRepOffsetAPI_MakeOffsetShape& maker)
{
  maker.PerformBySimple(TopoDS_Shape(), 3.0);
  std::cout << "failed_redo_done=" << maker.IsDone() << '\n';
  try
  {
    (void)maker.Shape();
    std::cout << "failed_redo_shape=returned\n";
  }
  catch (const Standard_Failure& failure)
  {
    std::cout << "failed_redo_shape_error=" << failure.what() << '\n';
  }
}

int main()
{
  BRepOffsetAPI_MakeOffsetShape maker;
  writeDefaultAndNull(maker);
  const TopoDS_Shape face = makePlaneFace();
  writePlaneRun(maker, face);
  writeFailedRedo(maker);
}
