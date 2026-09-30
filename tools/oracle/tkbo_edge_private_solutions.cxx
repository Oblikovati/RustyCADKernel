struct EESolutionProbe : IntTools_EdgeEdge {
 void configure(const std::vector<double>& v){
  myGeom1=ee_geometry_fixture(int(v[0]),0.5);myGeom2=ee_geometry_fixture(int(v[1]),0.5);
  myEdge1=BRepBuilderAPI_MakeEdge(myGeom1,0,1);myEdge2=BRepBuilderAPI_MakeEdge(myGeom2,0,1);myCurve1.Initialize(myEdge1);myCurve2.Initialize(myEdge2);
  myRange1=IntTools_Range(v[2],v[3]);myRange2=IntTools_Range(v[4],v[5]);myTol1=v[6];myTol2=2*v[6];myTol=myTol1+myTol2;myPTol1=myPTol2=1e-7;
  myResCoeff1=ResolutionCoeff(myCurve1,myRange1);myResCoeff2=ResolutionCoeff(myCurve2,myRange2);
  myRes1=Resolution(myCurve1.Curve().Curve(),myCurve1.GetType(),myResCoeff1,myTol1);myRes2=Resolution(myCurve2.Curve().Curve(),myCurve2.GetType(),myResCoeff2,myTol2);
 }
 using IntTools_EdgeEdge::FindSolutions;
};
void ee_solutions(const std::vector<double>& v){
 EESolutionProbe probe;probe.configure(v);NCollection_Sequence<IntTools_Range> first,second;first.Append(IntTools_Range(17,19));second.Append(IntTools_Range(23,29));bool split=false;
 try{probe.FindSolutions(first,second,split);bo_int(split);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 bo_int(first.Length());for(int i=1;i<=first.Length();++i){bo_range(first(i));bo_range(second(i));}
}
