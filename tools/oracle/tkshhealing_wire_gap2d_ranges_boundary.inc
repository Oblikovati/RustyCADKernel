void run_trimmed_surface_clip() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1.1, 0.2, 0), gp_Pnt(2.1, 0.2, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  Handle(Geom_Surface) plane = new Geom_Plane(gp_Pln());
  Handle(Geom_Surface) surface = new Geom_RectangularTrimmedSurface(plane, 0., 1., 0., 1., true, true);
  builder.MakeFace(face, surface, 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("trimmed_surface_clip", tool, tool.FixGap2d(2, false));
}

void run_surface_boundary_crossing() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0.8, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(0.8, 0.4, 0), gp_Pnt(1.8, -0.6, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  Handle(Geom_Surface) plane = new Geom_Plane(gp_Pln());
  Handle(Geom_Surface) surface = new Geom_RectangularTrimmedSurface(plane, 0., 1., 0., 1., true, true);
  builder.MakeFace(face, surface, 1e-7);
  builder.Add(face, wire);
  Handle(Geom2d_Curve) firstCurve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)));
  Handle(Geom2d_Curve) secondCurve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0.8, 0.4), gp_Dir2d(1, -1)));
  builder.UpdateEdge(first, firstCurve, face, 1e-7);
  builder.Range(first, face, 0., 0.8);
  builder.UpdateEdge(second, secondCurve, face, 1e-7);
  builder.Range(second, face, 0., 1.4142135623730951);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("surface_boundary_crossing", tool, tool.FixGap2d(2, false));
}
