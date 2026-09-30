// FaceFace state behavior from the unchanged OCCT 3d097a0 shared library.
#include <IntTools_FaceFace.hxx>
#include <limits>
void bo_face_state(const IntTools_FaceFace& ff) {
 bo_int(ff.IsDone());bo_int(ff.TangentFaces());bo_int(ff.Face1().IsNull());
 bo_int(ff.Face2().IsNull());bo_int(ff.Context().IsNull());
 bo_int(ff.Points().Length());bo_real(ff.FuzzyValue());
 try {bo_int(ff.Lines().Length());}
 catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
}
void bo_face_state_case(const std::vector<double>& v) {
 IntTools_FaceFace ff;bo_face_state(ff);ff.SetParameters(v[0]!=0,v[1]!=0,v[2]!=0,v[3]);
 ff.SetFuzzyValue(v[4]==99?std::numeric_limits<double>::quiet_NaN():v[4]);
 occ::handle<IntTools_Context> context=new IntTools_Context;ff.SetContext(context);bo_int(ff.Context()==context);
 NCollection_List<IntSurf_PntOn2S> points;
 for(int i=0;i<int(v[5]);++i)points.Append(IntSurf_PntOn2S());
 ff.SetList(points);bo_face_state(ff);
 for(bool split:{false,true}) {
  try {ff.PrepareLines3D(split);bo_int(true);}
  catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 }
 ff.SetContext(occ::handle<IntTools_Context>());bo_face_state(ff);
}

#ifdef TKBO_FACEFACE_PRIVATE_PROBE
extern bool ClassifyLin2d(const occ::handle<GeomAdaptor_Surface>&, const gp_Lin2d&, const double, double&, double&);
void bo_face_clip_case(const std::vector<double>& v) {
 occ::handle<Geom_Surface> plane=new Geom_Plane(gp::XOY());
 occ::handle<GeomAdaptor_Surface> surface=new GeomAdaptor_Surface(plane,v[0],v[1],v[2],v[3]);
 gp_Lin2d line(gp_Pnt2d(v[4],v[5]),gp_Dir2d(v[6],v[7]));
 double first=9,last=10;bo_int(ClassifyLin2d(surface,line,v[8],first,last));
 bo_real(first);bo_real(last);
}

void bo_face_plane_curve(const IntTools_Curve& curve) {
 bo_real(curve.Tolerance());bo_real(curve.TangentialTolerance());
 for(double t:{curve.Curve()->FirstParameter(),curve.Curve()->LastParameter()}) {
  bo_real(t);gp_Pnt p=curve.Curve()->Value(t);for(double x:{p.X(),p.Y(),p.Z()})bo_real(x);
 }
 for(const auto& pc:{curve.FirstCurve2d(),curve.SecondCurve2d()}) {
  bo_int(pc.IsNull());if(!pc.IsNull()){for(double t:{pc->FirstParameter(),pc->LastParameter()}){bo_real(t);gp_Pnt2d p=pc->Value(t);bo_real(p.X());bo_real(p.Y());}}
 }
}
void bo_face_planes_case(const std::vector<double>& v) {
 try {
  occ::handle<Geom_Surface> p1=new Geom_Plane(gp_Pnt(0,0,0),gp_Dir(0,0,1));
  occ::handle<Geom_Surface> p2=new Geom_Plane(gp_Pnt(0,0,v[0]),gp_Dir(v[1],v[2],v[3]));
  occ::handle<GeomAdaptor_Surface> s1=new GeomAdaptor_Surface(p1,-v[4],v[4],-v[4],v[4]);
  occ::handle<GeomAdaptor_Surface> s2=new GeomAdaptor_Surface(p2,-v[4],v[4],-v[4],v[4]);
  NCollection_Sequence<IntTools_Curve> curves;curves.Append(IntTools_Curve());bool tangent=true;
  PerformPlanes(s1,s2,v[5],v[6],1e-8,v[5]+v[6],v[7]!=0,v[8]!=0,curves,tangent);
  bo_int(tangent);bo_int(curves.Length());for(int i=2;i<=curves.Length();++i)bo_face_plane_curve(curves(i));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
}
void bo_face_policy_case(const std::vector<double>& v) {
 try {
  auto first=occ::down_cast<GeomAdaptor_Surface>(bo_topol_surface({v[0],v[2],0,1,0,1}));
  auto second=occ::down_cast<GeomAdaptor_Surface>(bo_topol_surface({v[1],v[3],0,1,0,1}));
  bo_int(SortTypes(first->GetType(),second->GetType()));
  int low,high,iterations;ApproxParameters(first,second,low,high,iterations);bo_int(low);bo_int(high);bo_int(iterations);
  double tolerance=v[4];Tolerances(first,second,tolerance);bo_real(tolerance);
  double u1,v1,u2,v2;Parameters(first,second,gp_Pnt(v[5],v[6],v[7]),u1,v1,u2,v2);
  for(double t:{u1,v1,u2,v2})bo_real(t);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
}
void bo_face_pcurves_case(const std::vector<double>& v) {
 gp_Cylinder cylinder(gp_Ax3(),v[0]);
 gp_Dir direction=v[4]==0?gp_Dir(0,0,1):gp_Dir(1,0,0);
 gp_Dir x=v[4]==0?gp_Dir(1,0,0):gp_Dir(0,1,0);
 gp_Ax3 axes(gp_Pnt(v[2],v[3],0),direction,x);if(v[5]!=0)axes.XReverse();
 gp_Sphere sphere(axes,v[1]);bo_int(ApproxWithPCurves(cylinder,sphere));
}

TopoDS_Face bo_face_special_face(int kind,double radius) {
 if(kind==23)return BRepBuilderAPI_MakeFace(gp_Torus(gp_Ax3(),radius*2.,radius));
 if(kind==24)return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(),radius));
 occ::handle<Geom_Surface> surface=new Geom_CylindricalSurface(gp_Ax3(),radius);
 auto trim=[](const occ::handle<Geom_Surface>& s)->occ::handle<Geom_Surface>{return new Geom_RectangularTrimmedSurface(s,0.2,5.5,-1.,1.);};
 auto offset=[](const occ::handle<Geom_Surface>& s)->occ::handle<Geom_Surface>{return new Geom_OffsetSurface(s,0.1,true);};
 if(kind==25)surface=offset(trim(surface));else if(kind==26)surface=trim(offset(surface));
 else if(kind==27)surface=offset(offset(surface));else if(kind==28)surface=trim(trim(surface));else surface=trim(surface);
 TopoDS_Face face;BRep_Builder().MakeFace(face,surface,1e-7);return face;
}
void bo_face_bounds_case(const std::vector<double>& v) {
 try {
  TopoDS_Face face;
  if(v[0]>=23)face=bo_face_special_face(int(v[0]),v[1]);
  else if(v[0]<14)face=bo_fclass_face(int(v[0]),v[1]);
  else {auto surface=occ::down_cast<GeomAdaptor_Surface>(bo_topol_surface({v[0]-14,v[1],0,1,0,1}));BRep_Builder().MakeFace(face,surface->Surface(),1e-7);}
  double u1,u2,v1,v2;IntTools_Context().UVBounds(face,u1,u2,v1,v2);
  if(v[3]!=0){u1=v[4];u2=v[5];v1=v[6];v2=v[7];}
  CorrectSurfaceBoundaries(face,v[2],u1,u2,v1,v2);
  for(double t:{u1,u2,v1,v2})bo_real(t);
  CorrectPlaneBoundaries(u1,u2,v1,v2);for(double t:{u1,u2,v1,v2})bo_real(t);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
}
#endif
