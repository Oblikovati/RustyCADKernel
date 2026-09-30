// Complete FaceFace behavior from immutable OCCT 3d097a0; supplementary to all ten originals.
void bo_face_perform_curve(const IntTools_Curve& curve) {
 bo_real(curve.Tolerance());bo_real(curve.TangentialTolerance());
 bo_int(curve.FirstCurve2d().IsNull());bo_int(curve.SecondCurve2d().IsNull());
 for(double t:{curve.Curve()->FirstParameter(),curve.Curve()->LastParameter()}) {
  bo_real(t);gp_Pnt p=curve.Curve()->Value(t);
  for(double coordinate:{p.X(),p.Y(),p.Z()})bo_real(coordinate);
 }
}
void bo_face_perform_case(const std::vector<double>& v) {
 try {
  auto first=bo_fclass_face(int(v[0]),3.);auto second=bo_fclass_face(int(v[1]),2.);
  IntTools_FaceFace ff;ff.SetParameters(v[2]!=0,v[3]!=0,v[4]!=0,1e-7);
  ff.Perform(first,second,v[5]!=0);bo_int(ff.IsDone());bo_int(ff.TangentFaces());
  bo_int(ff.Points().Length());bo_int(ff.Lines().Length());
  for(const auto& curve:ff.Lines())bo_face_perform_curve(curve);
  ff.PrepareLines3D(v[6]!=0);bo_int(ff.Lines().Length());
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
}
