// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeFix_SplitTool.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge, new Geom_Line(gp_Ax1(gp_Pnt(), gp_Dir(1., 0., 0.))), 1.e-7);
  builder.UpdateEdge(edge,
                     new Geom2d_TrimmedCurve(
                       new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))),
                       0.,
                       1.e9),
                     face,
                     1.e-7);
  builder.Range(edge, face, 0., 1.e9);
  builder.Range(edge, 0., 1., true);
  builder.SameParameter(edge, false);
  bool cutLine = true;
  const double cut = 1.e9 - 1.e-4;
  const bool changed = ShapeFix_SplitTool().CutEdge(edge, 0., cut, face, cutLine);
  double faceFirst, faceLast, edgeFirst, edgeLast;
  BRep_Tool::Range(edge, face, faceFirst, faceLast);
  BRep_Tool::Range(edge, edgeFirst, edgeLast);
  std::cout << std::setprecision(17) << "tiny_trimmed_line_cut changed " << changed
            << " cut_line " << cutLine << " face_first " << faceFirst << " face_last "
            << faceLast << " edge_first " << edgeFirst << " edge_last " << edgeLast << '\n';
}
