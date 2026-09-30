#include <Geom_Parabola.hxx>
occ::handle<Geom_Curve> bo_tools_geometry(int kind,double first,double last) {
 occ::handle<Geom_Curve> curve;
 if(kind==0)curve=new Geom_Line(gp_Ax1());
 else if(kind==1||kind==2)curve=new Geom_Circle(gp_Ax2(),2.);
 else if(kind==3){NCollection_Array1<gp_Pnt> p(1,4);p(1)=gp_Pnt();p(2)=gp_Pnt(1,2,0);p(3)=gp_Pnt(-1,2,0);p(4)=gp_Pnt();curve=new Geom_BezierCurve(p);}
 else if(kind==4||kind==5)curve=new Geom_Parabola(gp_Ax2(),kind==4?0.5:0.);
 else curve=new Geom_Line(gp_Ax1(gp_Pnt(),gp_Dir(1,0,0)));
 if(kind==0||kind==2||kind==4||kind==5||kind==6)return new Geom_TrimmedCurve(curve,first,last);
 return curve;
}
void bo_tools_descriptor(const IntTools_Curve& c) {
 bo_int(c.HasBounds());bo_real(c.Curve()->FirstParameter());bo_real(c.Curve()->LastParameter());bo_real(c.Tolerance());bo_real(c.TangentialTolerance());
 for(auto pc:{c.FirstCurve2d(),c.SecondCurve2d()}){bo_int(pc.IsNull());if(!pc.IsNull()){bo_real(pc->FirstParameter());bo_real(pc->LastParameter());}}
}
void bo_tools_curves_case(const std::vector<double>& v) {
 try{auto geom=bo_tools_geometry(int(v[0]),v[1],v[2]);bo_int(IntTools_Tools::IsClosed(geom));bo_real(IntTools_Tools::CurveTolerance(geom,v[3]));
 occ::handle<Geom2d_Curve> pc=new Geom2d_Line(gp_Ax2d()),null;IntTools_Curve curve(geom,v[4]==0.?null:pc,pc,v[3],v[3]*2.);
 NCollection_Sequence<IntTools_Curve> out;out.Append(curve);bo_int(IntTools_Tools::SplitCurve(curve,out));bo_int(out.Length());for(auto& item:out)bo_tools_descriptor(item);
 Bnd_Box box;bo_int(IntTools_Tools::CheckCurve(curve,box));double a,b,c,d,e,f;box.Get(a,b,c,d,e,f);for(double x:{a,b,c,d,e,f})bo_real(x);
 auto other=bo_tools_geometry(int(v[5]),v[1],v[2]);NCollection_Sequence<IntTools_Curve> input;input.Append(curve);input.Append(IntTools_Curve(other,null,null,0,0));input.Append(curve);
 IntTools_Tools::RejectLines(input,out);bo_int(out.Length());for(auto& item:input)bo_tools_descriptor(item);for(auto& item:out)bo_tools_descriptor(item);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
