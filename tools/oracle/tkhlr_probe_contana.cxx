// All Contap_ContAna overloads at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Contap_ContAna.hxx>
#include <gp_Sphere.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Cone.hxx>
void hl_contana_snapshot(const Contap_ContAna& c) {
 ga_flag(c.IsDone());GA_TRY(ga_int(c.NbContours()));GA_TRY(ga_int(c.TypeContour()));
 GA_TRY(gp_Circ a=c.Circle();ga_pnt(a.Location());ga_dir(a.Position().Direction());ga_dir(a.Position().XDirection());ga_real(a.Radius()));
 for(int i:{-1,0,1,2,3,4,5}){GA_TRY(gp_Lin a=c.Line(i);ga_pnt(a.Location());ga_dir(a.Direction()));}
}
struct HlAnalyticInput{gp_Ax3 frame;double radius,semi;gp_XYZ view;double draft;};
HlAnalyticInput hl_contana_read(GaReader& r) {
 gp_Pnt p=r.pnt();gp_Dir n=r.dir(),x=r.dir();gp_Ax3 f(gp_Ax2(p,n,x));if(r.flag())f.YReverse();
 double radius=r.real(),semi=r.real();gp_XYZ view=r.xyz();return {f,radius,semi,view,r.real()};
}
void hl_contana_perform(Contap_ContAna& c,int mode,const HlAnalyticInput& a) {
 if(mode%3==2){gp_Pnt eye(a.view);switch(mode/3){
 case 0:c.Perform(gp_Sphere(a.frame,a.radius),eye);break;
 case 1:c.Perform(gp_Cylinder(a.frame,a.radius),eye);break;
 default:c.Perform(gp_Cone(a.frame,a.semi,a.radius),eye);break;}return;}
 gp_Dir d(a.view);switch(mode){
 case 0:c.Perform(gp_Sphere(a.frame,a.radius),d);break;
 case 1:c.Perform(gp_Sphere(a.frame,a.radius),d,a.draft);break;
 case 3:c.Perform(gp_Cylinder(a.frame,a.radius),d);break;
 case 4:c.Perform(gp_Cylinder(a.frame,a.radius),d,a.draft);break;
 case 6:c.Perform(gp_Cone(a.frame,a.semi,a.radius),d);break;
 default:c.Perform(gp_Cone(a.frame,a.semi,a.radius),d,a.draft);break;}
}
void hl_contana_case(std::istream& in) {
 GaReader r{in};int mode=r.integer(),warm=r.integer();Contap_ContAna c;
 GA_TRY(HlAnalyticInput a=hl_contana_read(r);if(warm)c.Perform(gp_Sphere(gp_Ax3(),2),gp_Dir(0,0,1));hl_contana_snapshot(c);hl_contana_perform(c,mode,a));
 hl_contana_snapshot(c);ga_end();
}
