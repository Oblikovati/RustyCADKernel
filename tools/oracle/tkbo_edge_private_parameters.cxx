// A derived probe exposes the protected static method without modifying OCCT.
struct EEParameterProbe : IntTools_EdgeEdge { using IntTools_EdgeEdge::FindParameters; };
void ee_parameters(const std::vector<double>& v){
 auto source=ee_geometry_fixture(int(v[0]),0.5);TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(source,0,1);BRepAdaptor_Curve curve(edge);IntTools_Range range(v[1],v[2]);
 Bnd_Box box;box.Add(gp_Pnt(v[3]-v[4],-v[4],-v[4]));box.Add(gp_Pnt(v[3]+v[4],v[4],v[4]));box.SetGap(v[5]);
 double coefficient=ResolutionCoeff(curve,range),first=17,last=19;
 try{bo_int(EEParameterProbe::FindParameters(curve,v[1],v[2],v[6],v[7],v[8],coefficient,box,first,last));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 bo_real(first);bo_real(last);
}
