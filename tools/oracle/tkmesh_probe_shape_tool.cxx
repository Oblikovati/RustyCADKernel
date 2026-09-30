// ShapeTool uses actual BRep geometry and native polygon representations.
#include <BRepMesh_ShapeTool.hxx>
#include <BRepMeshData_Edge.hxx>
#include <Geom_Circle.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
std::pair<TopoDS_Edge,TopoDS_Face> tool_topology(int kind,double tolerance,double span) {
  BRep_Builder builder;TopoDS_Edge edge;builder.MakeEdge(edge);
  if(kind!=0) {
    occ::handle<Geom_Curve> curve=kind==3?occ::handle<Geom_Curve>(new Geom_Circle(gp_Ax2(),1.)):occ::handle<Geom_Curve>(new Geom_Line(gp_Ax1(gp_Pnt(),gp_Dir(1,0,0))));
    builder.UpdateEdge(edge,curve,2.e-7);builder.Range(edge,0.,span,false);
  }
  if(kind>=2) {
    TopoDS_Vertex first;builder.MakeVertex(first,gp_Pnt(),tolerance);builder.Add(edge,first.Oriented(TopAbs_FORWARD));
    TopoDS_Vertex last=first;if(kind==2)builder.MakeVertex(last,gp_Pnt(1,0,0),tolerance);builder.Add(edge,last.Oriented(TopAbs_REVERSED));
  }
  TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,edge);
  TopoDS_Face face;builder.MakeFace(face,new Geom_Plane(gp_Ax3()),3.e-7);builder.Add(face,wire);return {edge,face};
}
TopLoc_Location tool_location(bool located) {
  if(!located)return TopLoc_Location();gp_Trsf transform;transform.SetTranslation(gp_Vec(2,3,4));return TopLoc_Location(transform);
}
void tool_ranges(const TopoDS_Edge& edge,const TopoDS_Face& face) {
  for(bool orient:{false,true}) {
    occ::handle<Geom_Curve> curve;double first=0,last=0;ga_flag(BRepMesh_ShapeTool::Range(edge,curve,first,last,orient));if(!curve.IsNull()){ga_real(first);ga_real(last);}
    occ::handle<Geom2d_Curve> pcurve;ga_flag(BRepMesh_ShapeTool::Range(edge,face,pcurve,first,last,orient));if(!pcurve.IsNull()){ga_real(first);ga_real(last);}
    gp_Pnt2d start(19,23),end(29,31);ga_flag(BRepMesh_ShapeTool::UVPoints(edge,face,start,end,orient));ga_pnt2d(start);ga_pnt2d(end);
  }
}
void mesh_shape_tool(std::istream& input) {
  int kind,toleranceIndex,spanIndex,mask,orient,located;input>>kind>>toleranceIndex>>spanIndex>>mask>>orient>>located;
  const double tolerance=std::array<double,3>{1.e-7,1.,100.}[toleranceIndex],span=std::array<double,3>{0.001,1.,6.25}[spanIndex];
  auto [edge,face]=tool_topology(kind,tolerance,span);edge.Orientation(TopAbs_Orientation(orient));
  occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;
  occ::handle<IMeshData_Edge> discrete=new (allocator) BRepMeshData_Edge(edge,allocator);occ::handle<IMeshData_Face> discreteFace=new (allocator) BRepMeshData_Face(face,allocator);
  auto pcurve=discrete->AddPCurve(discreteFace.get(),TopAbs_Orientation(orient));
  discrete->SetSameParam(mask&1);discrete->SetSameRange(mask&2);discrete->SetDegenerated(mask&4);BRepMesh_ShapeTool::CheckAndUpdateFlags(discrete,pcurve);
  ga_flag(discrete->GetSameParam());ga_flag(discrete->GetSameRange());ga_flag(discrete->GetDegenerated());ga_real(BRepMesh_ShapeTool::MaxFaceTolerance(face));tool_ranges(edge,face);
  ga_pnt(BRepMesh_ShapeTool::UseLocation(gp_Pnt(2,3,4),tool_location(located)));
  double size=17;BRepMesh_ShapeTool::BoxMaxDimension(Bnd_Box(),size);ga_real(size);Bnd_Box bounds;bounds.Update(-2,-3,-4,span,tolerance,2);BRepMesh_ShapeTool::BoxMaxDimension(bounds,size);ga_real(size);ga_end();
}
void mesh_shape_polygons(std::istream& input) {
  int located,orient;input>>located>>orient;auto [edge,face]=tool_topology(2,1.e-7,1.);edge.Orientation(TopAbs_Orientation(orient));auto location=tool_location(located);face.Location(location);
  occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(3,1,false);
  for(int i=1;i<=3;++i)mesh->SetNode(i,gp_Pnt(i,5,6));BRepMesh_ShapeTool::AddInFace(face,mesh);for(int i=1;i<=3;++i)ga_pnt(mesh->Node(i));
  TopLoc_Location found;ga_flag(!BRep_Tool::Triangulation(face,found).IsNull());
  occ::handle<Poly_PolygonOnTriangulation> first=new Poly_PolygonOnTriangulation(2,false),last=new Poly_PolygonOnTriangulation(2,false);
  BRepMesh_ShapeTool::UpdateEdge(edge,first,mesh,location);ga_flag(!BRep_Tool::PolygonOnTriangulation(edge,mesh,location).IsNull());
  BRepMesh_ShapeTool::UpdateEdge(edge,first,last,mesh,location);ga_flag(BRep_Tool::IsClosed(edge,mesh,location));
  BRepMesh_ShapeTool::NullifyEdge(edge,mesh,location);ga_flag(BRep_Tool::PolygonOnTriangulation(edge,mesh,location).IsNull());
  occ::handle<Poly_Polygon3D> polygon=new Poly_Polygon3D(2,false);BRepMesh_ShapeTool::UpdateEdge(edge,polygon);ga_flag(!BRep_Tool::Polygon3D(edge,found).IsNull());
  BRepMesh_ShapeTool::NullifyEdge(edge,TopLoc_Location());ga_flag(BRep_Tool::Polygon3D(edge,found).IsNull());
  BRepMesh_ShapeTool::NullifyFace(face);ga_flag(BRep_Tool::Triangulation(face,found).IsNull());ga_end();
}
