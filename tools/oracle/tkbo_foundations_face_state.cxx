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
