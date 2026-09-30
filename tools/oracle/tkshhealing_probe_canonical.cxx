// State and in/out geometry of ShapeAnalysis_CanonicalRecognition at pinned OCCT.
#include <ShapeAnalysis_CanonicalRecognition.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>
#include <gp_Elips.hxx>
void sh_can_frame(const gp_Ax3& p) {
 ga_pnt(p.Location());ga_xyz(p.Direction().XYZ());ga_xyz(p.XDirection().XYZ());ga_xyz(p.YDirection().XYZ());
}
void sh_can_recognize(GaReader& r,ShapeAnalysis_CanonicalRecognition& tool,int op) {
 double tol=r.real(),radius=r.real(),angle=r.real();
 if(op==2){gp_Pln p;ga_flag(tool.IsPlane(tol,p));sh_can_frame(p.Position());}
 else if(op==3){gp_Cylinder p(gp_Ax3(),radius);ga_flag(tool.IsCylinder(tol,p));sh_can_frame(p.Position());ga_real(p.Radius());}
 else if(op==4){gp_Cone p(gp_Ax3(),angle,radius);ga_flag(tool.IsCone(tol,p));sh_can_frame(p.Position());ga_real(p.SemiAngle());ga_real(p.RefRadius());}
 else if(op==5){gp_Sphere p(gp_Ax3(),radius);ga_flag(tool.IsSphere(tol,p));sh_can_frame(p.Position());ga_real(p.Radius());}
 else if(op==6){gp_Lin p;ga_flag(tool.IsLine(tol,p));ga_pnt(p.Location());ga_xyz(p.Direction().XYZ());}
 else if(op==7){gp_Circ p(gp_Ax2(),radius);ga_flag(tool.IsCircle(tol,p));sh_can_frame(gp_Ax3(p.Position()));ga_real(p.Radius());}
 else {gp_Elips p(gp_Ax2(),radius,radius/2);ga_flag(tool.IsEllipse(tol,p));sh_can_frame(gp_Ax3(p.Position()));ga_real(p.MajorRadius());ga_real(p.MinorRadius());}
}
void sh_can_state(const ShapeAnalysis_CanonicalRecognition& tool,const TpShapes& t) {
 ga_int(tool.GetStatus());ga_real(tool.GetGap());ga_flag(tool.GetShape().IsNull());
 for(const auto& shape:t.shapes)ga_flag(tool.GetShape().IsSame(shape));
}
void sh_canonical_case(std::istream& input) {
 GaReader r{input};TpShapes t=tp_shapes(r);int initial=r.integer();
 ShapeAnalysis_CanonicalRecognition tool=initial<0?ShapeAnalysis_CanonicalRecognition():ShapeAnalysis_CanonicalRecognition(t.shapes.at(initial));
 sh_can_state(tool,t);int count=r.integer();
 for(int i=0;i<count;++i){int op=r.integer();
  if(op==0){int index=r.integer();GA_TRY(tool.SetShape(t.shapes.at(index)))}
  else if(op==1)tool.ClearStatus();
  else GA_TRY(sh_can_recognize(r,tool,op))
  sh_can_state(tool,t);
 }ga_end();
}
