// Native oracle for the BRepAlgoAPI Boolean classes: Cut, Fuse, Common, Section (shapes and
// plane), the General Fuse API, a reused pave filler and the section edges.
#include <BOPAlgo_PaveFiller.hxx>
#include <BRepAlgoAPI_BuilderAlgo.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Section.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <gp_Ax2.hxx>
#include <gp_Pln.hxx>
#include "tkbo_shape_dump.hxx"

static void report(const char* label, BRepAlgoAPI_BuilderAlgo& api)
{
  std::cout << "case " << label << " done " << api.IsDone() << " errors " << api.HasErrors()
            << " warnings " << api.HasWarnings() << " modified " << api.HasModified()
            << " generated " << api.HasGenerated() << " deleted " << api.HasDeleted() << '\n';
  ShapeIndex all;
  dumpResult(api.Shape(), all);
  std::cout << "section";
  for (const TopoDS_Shape& edge : api.SectionEdges())
    std::cout << ' ' << all.FindIndex(edge);
  std::cout << '\n';
}

int main()
{
  TopoDS_Shape box = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  TopoDS_Shape other = BRepPrimAPI_MakeBox(gp_Pnt(5, 5, 5), 10., 10., 10.).Shape();
  TopoDS_Shape cylinder =
    BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 5, -5), gp_Dir(0, 0, 1)), 2., 20.).Shape();
  {
    BRepAlgoAPI_Cut cut(box, other);
    report("cut", cut);
    BRepAlgoAPI_Fuse fuse(box, cylinder);
    report("fuse", fuse);
    BRepAlgoAPI_Common common(box, cylinder);
    report("common", common);
    BRepAlgoAPI_Section section(box, cylinder);
    report("section", section);
    BRepAlgoAPI_Section plane(box, gp_Pln(gp_Pnt(0, 0, 4), gp_Dir(0, 0, 1)));
    report("section-plane", plane);
  }
  {
    NCollection_List<TopoDS_Shape> arguments;
    arguments.Append(box);
    arguments.Append(cylinder);
    BRepAlgoAPI_BuilderAlgo general;
    general.SetArguments(arguments);
    general.Build();
    report("general-fuse", general);
  }
  {
    NCollection_List<TopoDS_Shape> arguments;
    arguments.Append(box);
    arguments.Append(other);
    BOPAlgo_PaveFiller filler;
    filler.SetArguments(arguments);
    filler.Perform();
    BRepAlgoAPI_Cut forward(box, other, filler, true);
    report("filler-cut", forward);
    BRepAlgoAPI_Cut backward(box, other, filler, false);
    report("filler-cut21", backward);
    BRepAlgoAPI_Common common(box, other, filler);
    report("filler-common", common);
  }
  {
    BRepAlgoAPI_Cut empty;
    empty.Build();
    std::cout << "case empty done " << empty.IsDone() << " errors " << empty.HasErrors() << '\n';
  }
  return 0;
}
