// BRepPrim one-axis probe operations (the records are documented in
// tkprim_probe_brepprim.cxx); the Rust twin is
// crates/tkprim/examples/pr_probe/brep_prim_one_axis.rs.
// The shapes of a one-axis primitive: the members 0..29 of the call record.
TopoDS_Shape pr_oneaxis_member(BRepPrim_OneAxis& p,int op){
 switch(op){
  case 0:return p.Shell();case 1:return p.LateralFace();case 2:return p.TopFace();
  case 3:return p.BottomFace();case 4:return p.StartFace();case 5:return p.EndFace();
  case 6:return p.LateralWire();case 7:return p.LateralStartWire();case 8:return p.LateralEndWire();
  case 9:return p.TopWire();case 10:return p.BottomWire();case 11:return p.StartWire();
  case 12:return p.AxisStartWire();case 13:return p.EndWire();case 14:return p.AxisEndWire();
  case 15:return p.AxisEdge();case 16:return p.StartEdge();case 17:return p.EndEdge();
  case 18:return p.StartTopEdge();case 19:return p.StartBottomEdge();case 20:return p.EndTopEdge();
  case 21:return p.EndBottomEdge();case 22:return p.TopEdge();case 23:return p.BottomEdge();
  case 24:return p.AxisTopVertex();case 25:return p.AxisBottomVertex();case 26:return p.TopStartVertex();
  case 27:return p.TopEndVertex();case 28:return p.BottomStartVertex();default:return p.BottomEndVertex();}}
// The setters 40..44 of the call record.
void pr_oneaxis_setter(BRepPrim_OneAxis& p,int op,double arg){
 if(op==40)p.Angle(arg);
 else if(op==41)p.VMin(arg);
 else if(op==42)p.VMax(arg);
 else if(op==43)p.SetMeridianOffset(arg);
 else p.Axes(p.Axes().Rotated(gp::OX(),arg));}
// The queries 50..61 of the call record.
void pr_oneaxis_query(BRepPrim_OneAxis& p,int op,double arg){
 switch(op){
  case 50:ga_pnt2d(p.MeridianValue(arg));break;case 51:ga_flag(p.MeridianOnAxis(arg));break;
  case 52:ga_flag(p.MeridianClosed());break;case 53:ga_flag(p.VMaxInfinite());break;
  case 54:ga_flag(p.VMinInfinite());break;case 55:ga_flag(p.HasTop());break;
  case 56:ga_flag(p.HasBottom());break;case 57:ga_flag(p.HasSides());break;
  case 58:ga_real(p.Angle());break;case 59:ga_real(p.VMin());break;
  case 60:ga_real(p.VMax());break;default:pr_print_ax2(p.Axes());break;}}
// One call of the record.
void pr_oneaxis_call(BRepPrim_OneAxis& p,int op,double arg){
 if(op<30){pr_shape(pr_oneaxis_member(p,op));return;}
 if(op==30){pr_shape(p.MakeEmptyLateralFace());return;}
 if(op==31){pr_shape(p.MakeEmptyMeridianEdge(arg));return;}
 if(op==32){TopoDS_Edge e=p.MakeEmptyMeridianEdge(arg);TopoDS_Face f=p.MakeEmptyLateralFace();
  p.SetMeridianPCurve(e,f);pr_shape(e);return;}
 if(op<50){pr_oneaxis_setter(p,op,arg);return;}
 pr_oneaxis_query(p,op,arg);}
// The primitive of `kind ctor fields...`: kind 0 cylinder, 1 cone, 2 sphere, 3 torus,
// 4 revolution; each constructor overload in header order (the cone of (Angle, Axes),
// whose radius is uninitialized natively, is left out).
BRepPrim_OneAxis* pr_cylinder(GaReader& r,int ctor){
 if(ctor==0){gp_Ax2 a=pr_ax2(r);double radius=r.real();return new BRepPrim_Cylinder(a,radius,r.real());}
 if(ctor==1)return new BRepPrim_Cylinder(r.real());
 if(ctor==2){gp_Pnt c=r.pnt();return new BRepPrim_Cylinder(c,r.real());}
 if(ctor==3){gp_Ax2 a=pr_ax2(r);return new BRepPrim_Cylinder(a,r.real());}
 if(ctor==4){double radius=r.real();return new BRepPrim_Cylinder(radius,r.real());}
 gp_Pnt c=r.pnt();double radius=r.real();return new BRepPrim_Cylinder(c,radius,r.real());}
BRepPrim_OneAxis* pr_cone(GaReader& r,int ctor){
 if(ctor==0){double angle=r.real();gp_Ax2 a=pr_ax2(r);double h=r.real();return new BRepPrim_Cone(angle,a,h,r.real());}
 if(ctor==1)return new BRepPrim_Cone(r.real());
 if(ctor==2){double angle=r.real();return new BRepPrim_Cone(angle,r.pnt());}
 if(ctor==4){double r1=r.real(),r2=r.real();return new BRepPrim_Cone(r1,r2,r.real());}
 if(ctor==5){gp_Pnt c=r.pnt();double r1=r.real(),r2=r.real();return new BRepPrim_Cone(c,r1,r2,r.real());}
 gp_Ax2 a=pr_ax2(r);double r1=r.real(),r2=r.real();return new BRepPrim_Cone(a,r1,r2,r.real());}
BRepPrim_OneAxis* pr_sphere(GaReader& r,int ctor){
 if(ctor==0)return new BRepPrim_Sphere(r.real());
 if(ctor==1){gp_Pnt c=r.pnt();return new BRepPrim_Sphere(c,r.real());}
 gp_Ax2 a=pr_ax2(r);return new BRepPrim_Sphere(a,r.real());}
BRepPrim_OneAxis* pr_torus(GaReader& r,int ctor){
 if(ctor==0){gp_Ax2 a=pr_ax2(r);double major=r.real();return new BRepPrim_Torus(a,major,r.real());}
 if(ctor==1){double major=r.real();return new BRepPrim_Torus(major,r.real());}
 gp_Pnt c=r.pnt();double major=r.real();return new BRepPrim_Torus(c,major,r.real());}
BRepPrim_OneAxis* pr_revolution(GaReader& r){
 gp_Ax2 a=pr_ax2(r);double vmin=r.real(),vmax=r.real();
 occ::handle<Geom_Curve> m=tp_geom_curve(r);occ::handle<Geom2d_Curve> pm=tp_geom2d_curve(r);
 return new BRepPrim_Revolution(a,vmin,vmax,m,pm);}
BRepPrim_OneAxis* pr_one_axis(GaReader& r,int kind,int ctor){
 if(kind==0)return pr_cylinder(r,ctor);
 if(kind==1)return pr_cone(r,ctor);
 if(kind==2)return pr_sphere(r,ctor);
 if(kind==3)return pr_torus(r,ctor);
 return pr_revolution(r);}
// `kind ctor fields... ncalls (op arg)...`: the primitive (or the constructor's exception
// and the end of the line), each call printed or raising, then the shell.
void pr_bp_oneaxis_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),ctor=r.integer();std::unique_ptr<BRepPrim_OneAxis> p;
 try{p.reset(pr_one_axis(r,kind,ctor));}
 catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';ga_end();return;}
 int n=r.integer();
 for(int i=0;i<n;++i){int op=r.integer();double arg=r.real();if(op==99){ga_end();return;}GA_TRY(pr_oneaxis_call(*p,op,arg))}
 GA_TRY(pr_shape(p->Shell()))
 ga_end();}
