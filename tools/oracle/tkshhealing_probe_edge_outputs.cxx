// ShapeAnalysis_Edge output preservation with nonzero caller-owned sentinels.
void sh_output_tangent(GaReader& r, const TpShapes& t, ShapeAnalysis_Edge& tool, int op) {
 TopoDS_Edge edge=sh_edge(t,r.integer()); TopoDS_Face face;
 occ::handle<Geom_Surface> surface; TopLoc_Location location;
 if(op==0)face=sh_face(t,r.integer());
 else {surface=t.surfaces.at(r.integer());location=t.locations.at(r.integer());}
 bool atEnd=r.flag();double fraction=r.real(),x=r.real(),y=r.real(),vx=r.real(),vy=r.real();
 gp_Pnt2d point(x,y);gp_Vec2d tangent(vx,vy);
 bool result=op==0?tool.GetEndTangent2d(edge,face,atEnd,point,tangent,fraction):tool.GetEndTangent2d(edge,surface,location,atEnd,point,tangent,fraction);
 ga_flag(result);ga_pnt2d(point);sh_vec2d(tangent);
}
void sh_output_tolerance(GaReader& r, const TpShapes& t, ShapeAnalysis_Edge& tool, int op) {
 TopoDS_Edge edge=sh_edge(t,r.integer());TopoDS_Face face;
 if(op==2)face=sh_face(t,r.integer());
 double first=r.real(),last=r.real();
 ga_flag(op==2?tool.CheckVertexTolerance(edge,face,first,last):tool.CheckVertexTolerance(edge,first,last));
 ga_real(first);ga_real(last);sh_flags(tool);
}
void sh_output_deviation(GaReader& r, const TpShapes& t, ShapeAnalysis_Edge& tool, int op) {
 TopoDS_Edge edge=sh_edge(t,r.integer());TopoDS_Face face;
 if(op==4)face=sh_face(t,r.integer());
 int samples=r.integer();double deviation=r.real();
 ga_flag(op==4?tool.CheckSameParameter(edge,face,deviation,samples):tool.CheckSameParameter(edge,deviation,samples));
 ga_real(deviation);sh_flags(tool);
}
void sh_edgeoutputs_case(std::istream& input) {
 GaReader r{input};TpShapes t=tp_shapes(r);int op=r.integer();ShapeAnalysis_Edge tool;
 if(op<2)GA_TRY(sh_output_tangent(r,t,tool,op))
 else if(op<4)GA_TRY(sh_output_tolerance(r,t,tool,op))
 else GA_TRY(sh_output_deviation(r,t,tool,op))
 ga_end();
}
