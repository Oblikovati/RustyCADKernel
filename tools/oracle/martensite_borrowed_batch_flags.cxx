// SPDX-License-Identifier: GPL-2.0-only
// OCCT3d097a0 mapped Add freezes components before copy; ADR0014 avoids that input mutation.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Trsf.hxx>
#include <iostream>

static void CaptureMatrix(const TopoDS_Shape& shape, bool free, const char* label)
{
  const gp_Trsf transform = shape.Location().Transformation();
  std::cout << "matrix " << free << ' ' << label;
  for (int row = 1; row <= 3; ++row)
    for (int column = 1; column <= 4; ++column)
      std::cout << ' ' << transform.Value(row, column);
  std::cout << '\n';
}
static void CaptureWorld(const TopoDS_Shape& located, const TopoDS_Shape& copied, bool free)
{
  const gp_XYZ originalShift = located.Location().Transformation().TranslationPart();
  const gp_XYZ copiedShift = copied.Location().Transformation().TranslationPart();
  const gp_Pnt originalPoint = BRep_Tool::Pnt(TopoDS::Vertex(located));
  const gp_Pnt copiedPoint = BRep_Tool::Pnt(TopoDS::Vertex(copied));
  std::cout << "world " << free << ' ' << originalShift.X() << ' ' << copiedShift.X()
    << ' ' << originalPoint.X() << ' ' << copiedPoint.X() << ' '
    << originalPoint.Distance(copiedPoint) << '\n';
  CaptureMatrix(located, free, "source");
  CaptureMatrix(copied, free, "copy");
}
static void CaptureCopied(const TopoDS_Compound& batch, const TopoDS_Shape& located,
  const TopoDS_Shape& reversed, bool free)
{
  BRepBuilderAPI_Copy copier(batch, true, true);
  TopoDS_Iterator copied(copier.Shape());
  const TopoDS_Shape first = copied.Value();
  copied.Next();
  const TopoDS_Shape second = copied.Value();
  std::cout << "copy " << free << ' ' << first.IsSame(second) << ' ' << first.IsSame(located)
    << ' ' << (first.Orientation() == located.Orientation()) << ' '
    << (second.Orientation() == reversed.Orientation()) << ' '
    << (first.Location() == located.Location()) << '\n';
  CaptureWorld(located, first, free);
}
static void CaptureFlags(bool free)
{
  BRep_Builder builder;
  TopoDS_Vertex source;
  builder.MakeVertex(source, gp_Pnt(1.,2.,3.), 1e-7);
  source.Free(free);
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(3.,4.,5.));
  const TopoDS_Shape located = source.Located(TopLoc_Location(translation));
  const TopoDS_Shape reversed = located.Reversed();
  TopoDS_Compound batch;
  builder.MakeCompound(batch);
  builder.Add(batch, located);
  builder.Add(batch, reversed);
  std::cout << "add " << free << ' ' << source.Free() << ' ' << located.IsSame(reversed) << '\n';
  CaptureCopied(batch, located, reversed, free);
}
int main()
{
  CaptureFlags(false);
  CaptureFlags(true);
  return 0;
}
