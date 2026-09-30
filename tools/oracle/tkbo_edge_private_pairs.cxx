struct EEPairProbe : IntTools_EdgeEdge {
 void configure(const std::vector<double>& v){
  myGeom1=ee_geometry_fixture(int(v[0]),0.5);myGeom2=ee_geometry_fixture(int(v[1]),0.5);
  myEdge1=BRepBuilderAPI_MakeEdge(myGeom1,0,1);myCurve1.Initialize(myEdge1);
  myRange1=IntTools_Range(v[2],v[3]);myRange2=IntTools_Range(v[4],v[5]);myTol=v[6];myRes1=myRes2=v[7];myPTol1=1e-7;myResCoeff1=ResolutionCoeff(myCurve1,myRange1);
 }
 using IntTools_EdgeEdge::IsIntersection; using IntTools_EdgeEdge::IsCoincident; using IntTools_EdgeEdge::CheckCoincidence;using IntTools_EdgeEdge::FindBestSolution;
};
void ee_pairs(const std::vector<double>& v){
 EEPairProbe probe;probe.configure(v);
 try{bo_int(probe.IsCoincident());}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try{bo_int(probe.CheckCoincidence(v[2],v[3],v[4],v[5],v[6],v[7]));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try{bo_int(probe.IsIntersection(v[2],v[3],v[4],v[5]));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 double first=17,last=19;try{probe.FindBestSolution(v[2],v[3],v[4],v[5],first,last);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_real(first);bo_real(last);
}
