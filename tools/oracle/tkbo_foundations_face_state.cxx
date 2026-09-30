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

#endif
