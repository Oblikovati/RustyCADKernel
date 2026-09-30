#include <GeomInt_LineConstructor.hxx>
std::vector<double> bo_wline_parameters(int pattern) {
 const double tau=2*M_PI;
 switch(pattern){case 0:return {};case 1:return {0,0,0};case 2:return {0.1,0.2,0.3};case 3:return {0,0.1,0.2,tau};
 case 4:return {0.1,0.2,0,tau-0.2,tau-0.1};case 5:return {0,0.1,tau};case 6:return {tau,tau-0.1,tau-0.2,0};
 case 7:return {0,1e-10,0.1,0.2,tau-1e-10,tau};case 8:return {0.1,0.1,0,0.2,0.2};case 9:return {0,0.1,0.2,0.3};case 10:return {0.1,0.2,0.3,0};case 11:return {-0.1,-0.2,0,0.1,0.2};case 12:return {1e-6,0.1,0.2,tau+1e-6};case 13:return {-1e-6,0.1,0.2,-1e-6};default:return {0.1,0.2,1e-6,0.3,0.4};}
}
occ::handle<IntPatch_WLine> bo_wline_walking(const occ::handle<Geom_Surface>& surface,const std::vector<double>& v) {
 occ::handle<IntSurf_LineOn2S> points=new IntSurf_LineOn2S;
 for(double parameter:bo_wline_parameters(int(v[2]))){bool vertical=v.size()>6&&v[6]!=0.;double u=vertical?v[3]:parameter,w=vertical?parameter:v[3];gp_Pnt p=surface->Value(u,w);IntSurf_PntOn2S point;
 double x=v[1]==0.?p.X():(vertical?u:u+v[4]),y=v[1]==0.?p.Y():(vertical?w+v[4]:w);point.SetValue(p,u,w,x,y);points->Add(point);}
 occ::handle<IntPatch_WLine> line=new IntPatch_WLine(points,false);line->SetCreatingWayInfo(IntPatch_WLine::IntPatch_WLImpImp);return line;
}
void bo_wline_emit(const occ::handle<IntPatch_WLine>& line) {
 bo_int(line->NbPnts());bo_int(line->GetCreatingWay());for(int i=1;i<=line->NbPnts();++i){const auto& p=line->Point(i);double u,v,x,y;p.Parameters(u,v,x,y);for(double value:{p.Value().X(),p.Value().Y(),p.Value().Z(),u,v,x,y})bo_real(value);}
}
#include <Adaptor3d_TopolTool.hxx>
GeomInt_LineConstructor bo_wline_constructor(const occ::handle<IntPatch_WLine>& line,const occ::handle<GeomAdaptor_Surface>& a,const occ::handle<GeomAdaptor_Surface>& b,int mode) {
 GeomInt_LineConstructor constructor;if(mode<2)return constructor;int count=line->NbPnts();std::vector<int> indices;
 if(mode==2)indices={1,count};else if(mode==3){for(int i=1;i<=count;++i)indices.push_back(i);}else indices={2,count-1};
 for(int index:indices){if(index<1||index>count)continue;IntPatch_Point point;point.SetValue(line->Point(index).Value(),0.,false);double u,v,x,y;line->Point(index).Parameters(u,v,x,y);point.SetParameters(u,v,x,y);point.SetParameter(index);line->AddVertex(point);}
 constructor.Load(new Adaptor3d_TopolTool(a),new Adaptor3d_TopolTool(b),a,b);constructor.Perform(line);return constructor;
}
void bo_wline_decompose_case(const std::vector<double>& v) {
 try{auto f1=bo_fclass_face(int(v[0]),2.),f2=bo_fclass_face(int(v[1]),2.);auto s1=BRep_Tool::Surface(f1),s2=BRep_Tool::Surface(f2);
 occ::handle<GeomAdaptor_Surface> a=new GeomAdaptor_Surface(s1),b=new GeomAdaptor_Surface(s2);auto line=bo_wline_walking(s1,v);auto constructor=bo_wline_constructor(line,a,b,int(v[5]));NCollection_Sequence<occ::handle<IntPatch_Line>> out;out.Append(line);
 bo_int(IntTools_WLineTool::DecompositionOfWLine(line,a,b,f1,f2,constructor,v[5]==1.,1e-6,out,new IntTools_Context));bo_int(out.Length());for(auto& item:out)bo_wline_emit(occ::down_cast<IntPatch_WLine>(item));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
