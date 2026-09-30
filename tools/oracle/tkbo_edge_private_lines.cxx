struct EELineProbe : IntTools_EdgeEdge {
 void configure(const std::vector<double>& v){
  double dx=1,dy=0;switch(int(v[0])){case 1:dx=-1;break;case 2:dx=0;dy=1;break;case 3:dy=1;break;case 4:dy=1e-8;break;case 5:dy=1e-14;break;}
  myEdge1=BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(0,0,0),gp_Dir(1,0,0)),0,1);
  myEdge2=BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(v[1],v[2],v[3]),gp_Dir(dx,dy,0)),0,1);
  if(v[8]!=0){TopoDS_Iterator child(myEdge1);BRep_Builder builder;builder.Add(myEdge2,child.Value());}
  myCurve1.Initialize(myEdge1);myCurve2.Initialize(myEdge2);myRange1=IntTools_Range(v[4],v[5]);myRange2=IntTools_Range(-0.5,0.75);myTol1=v[6];myTol2=v[7];myTol=myTol1+myTol2;
 }
 using IntTools_EdgeEdge::ComputeLineLine;
 void output(){bo_int(myCommonParts.Length());for(const auto& part:myCommonParts){bo_int(part.Type());bo_int(part.AllNullFlag());bo_int(part.Edge1().IsSame(myEdge1));bo_int(part.Edge2().IsSame(myEdge2));bo_range(part.Range1());bo_real(part.VertexParameter1());bo_real(part.VertexParameter2());bo_int(part.Ranges2().Length());for(const auto& range:part.Ranges2())bo_range(range);}}
};
void ee_lines(const std::vector<double>& v){EELineProbe probe;probe.configure(v);probe.ComputeLineLine();probe.output();}
