// DsgPrs dimension and relation presentations through the recording display driver, the twin
// of crates/tkv3d/examples/v3_probe/dsg_prs.rs. Each request is
// `Kind ArrowLength Side P1..P5 D1..D3 V1..V4 F1 F2`.
#include <DsgPrs.hxx>
#include <DsgPrs_AnglePresentation.hxx>
#include <DsgPrs_Chamf2dPresentation.hxx>
#include <DsgPrs_ConcentricPresentation.hxx>
#include <DsgPrs_DatumPrs.hxx>
#include <DsgPrs_DiameterPresentation.hxx>
#include <DsgPrs_EllipseRadiusPresentation.hxx>
#include <DsgPrs_EqualDistancePresentation.hxx>
#include <DsgPrs_EqualRadiusPresentation.hxx>
#include <DsgPrs_FilletRadiusPresentation.hxx>
#include <DsgPrs_FixPresentation.hxx>
#include <DsgPrs_IdenticPresentation.hxx>
#include <DsgPrs_LengthPresentation.hxx>
#include <DsgPrs_MidPointPresentation.hxx>
#include <DsgPrs_OffsetPresentation.hxx>
#include <DsgPrs_ParalPresentation.hxx>
#include <DsgPrs_PerpenPresentation.hxx>
#include <DsgPrs_RadiusPresentation.hxx>
#include <DsgPrs_ShapeDirPresentation.hxx>
#include <DsgPrs_SymbPresentation.hxx>
#include <DsgPrs_SymmetricPresentation.hxx>
#include <DsgPrs_TangentPresentation.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRep_Builder.hxx>
#include <ElCLib.hxx>
#include <TopoDS_Compound.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Prs3d_ArrowAspect.hxx>
#include <Prs3d_DimensionAspect.hxx>
#include <gp_Elips.hxx>
struct V3DsgInput {
  int kind;double arrow;DsgPrs_ArrowSide side;gp_Pnt p[5];gp_Dir d[3];double v[4];bool f[2];
};
static V3DsgInput v3_dsg_input(GaReader& r){
  V3DsgInput in;in.kind=r.integer();in.arrow=r.real();in.side=DsgPrs_ArrowSide(r.integer());
  for(gp_Pnt& p:in.p)p=r.pnt();for(gp_Dir& d:in.d)d=r.dir();for(double& v:in.v)v=r.real();for(bool& f:in.f)f=r.flag();return in;}
static void v3_dsg_texts(const V3DisplayTrace& trace){
  ga_int(int(trace.texts.size()));
  for(const occ::handle<Graphic3d_Text>& t:trace.texts){ga_text(t->Text().ToCString());ga_pnt(t->Position());ga_real(t->Height());}}
static void v3_dsg_circle_out(const gp_Circ& c){ga_pnt(c.Location());ga_dir(c.Axis().Direction());ga_dir(c.XAxis().Direction());ga_real(c.Radius());}
// The package computations with their outputs.
static void v3_dsg_package(const V3DsgInput& in){
  switch(in.kind){
    case 100:{gp_Pnt e1(9,9,9),e2(8,8,8);gp_Dir d1(0,1,0);GA_TRY(DsgPrs::ComputePlanarFacesLengthPresentation(in.v[0],in.v[1],in.p[0],in.p[1],in.d[0],in.p[2],gp_Pln(in.p[3],in.d[1]),e1,e2,d1))
      ga_pnt(e1);ga_pnt(e2);ga_dir(d1);break;}
    case 101:{gp_Circ ac,tc;double f1=7,l1=7,f2=6,l2=6;gp_Pnt e1(1,1,1),e2(2,2,2),pp(3,3,3);gp_Dir d1(1,0,0),d2(0,1,0);
      GA_TRY(DsgPrs::ComputeFacesAnglePresentation(in.arrow,in.v[0],in.p[0],in.p[1],in.p[2],in.d[0],in.d[1],in.d[2],in.f[0],gp_Ax1(in.p[4],in.d[2]),in.p[3],ac,f1,l1,e1,e2,d1,d2,pp,tc,f2,l2))
      v3_dsg_circle_out(ac);ga_real(f1);ga_real(l1);ga_pnt(e1);ga_pnt(e2);ga_dir(d1);ga_dir(d2);ga_pnt(pp);v3_dsg_circle_out(tc);ga_real(f2);ga_real(l2);break;}
    case 102:{bool spec=false;gp_Circ fc;double f=5,l=5;gp_Pnt e(4,4,4),dp(5,5,5);gp_Dir da(0,0,1);
      GA_TRY(DsgPrs::ComputeFilletRadiusPresentation(in.arrow,in.v[0],in.p[0],in.d[0],in.p[1],in.p[2],in.p[3],in.p[4],in.f[0],spec,fc,f,l,e,da,dp))
      ga_flag(spec);v3_dsg_circle_out(fc);ga_real(f);ga_real(l);ga_pnt(e);ga_dir(da);ga_pnt(dp);break;}
    case 103:{gp_Pnt o(1,2,3),e(4,5,6);GA_TRY(DsgPrs::ComputeRadiusLine(in.p[0],in.p[1],in.p[2],in.f[0],o,e))ga_pnt(o);ga_pnt(e);break;}
    case 104:{GA_TRY(gp_Elips el(gp_Ax2(in.p[0],in.d[0],in.d[1]),in.v[0],in.v[1]);ga_real(DsgPrs::DistanceFromApex(el,in.p[1],in.v[2])))break;}
    default:{occ::handle<Geom_Surface> s=new Geom_SphericalSurface(gp_Ax3(in.p[4],in.d[2]),in.v[0]);gp_Pnt e2(7,7,7);gp_Dir d1(1,0,0);
      occ::handle<Geom_Curve> vc,uc;double fu=1,du=2,fv=3,dv=4;
      GA_TRY(DsgPrs::ComputeCurvilinearFacesLengthPresentation(in.v[1],in.v[2],s,in.p[0],in.p[1],in.d[0],e2,d1,vc,uc,fu,du,fv,dv))
      ga_pnt(e2);ga_dir(d1);ga_flag(vc.IsNull());ga_flag(uc.IsNull());ga_real(fu);ga_real(du);ga_real(fv);ga_real(dv);}
  }}
static void v3_dsg_call(const occ::handle<Graphic3d_Structure>& prs,const occ::handle<Prs3d_Drawer>& dr,const V3DsgInput& in){
  const TCollection_ExtendedString t("12.5");const gp_Pnt* p=in.p;const gp_Dir* d=in.d;const double* v=in.v;const bool* f=in.f;const DsgPrs_ArrowSide s=in.side;
  auto circ=[&](int c,int n,double radius){return gp_Circ(gp_Ax2(p[c],d[n]),radius);};
  auto elips=[&](){return gp_Elips(gp_Ax2(p[2],d[0],d[1]),v[1],v[2]);};
  auto plane=[&](int c,int n){return occ::handle<Geom_Plane>(new Geom_Plane(p[c],d[n]));};
  switch(in.kind){
    case 0:DsgPrs_Chamf2dPresentation::Add(prs,dr,p[0],p[1],t);break;
    case 1:DsgPrs_Chamf2dPresentation::Add(prs,dr,p[0],p[1],t,s);break;
    case 2:DsgPrs_ConcentricPresentation::Add(prs,dr,p[0],v[0],d[0],p[1]);break;
    case 3:DsgPrs_DatumPrs::Add(prs,gp_Ax2(p[0],d[0],d[1]),dr);break;
    case 4:DsgPrs_DiameterPresentation::Add(prs,dr,t,p[0],circ(1,0,v[0]),s,f[0]);break;
    case 5:DsgPrs_DiameterPresentation::Add(prs,dr,t,p[0],circ(1,0,v[0]),v[1],v[2],s,f[0]);break;
    case 6:DsgPrs_EllipseRadiusPresentation::Add(prs,dr,v[0],t,p[0],p[1],p[2],f[0],s);break;
    case 7:DsgPrs_EllipseRadiusPresentation::Add(prs,dr,v[0],t,elips(),p[0],p[1],p[2],v[3],f[1],f[0],s);break;
    case 8:{occ::handle<Geom_OffsetCurve> oc=new Geom_OffsetCurve(new Geom_Ellipse(elips()),0.5,d[0]);
      DsgPrs_EllipseRadiusPresentation::Add(prs,dr,v[0],t,oc,p[0],p[1],p[2],v[3],f[1],f[0],s);break;}
    case 9:DsgPrs_EqualDistancePresentation::Add(prs,dr,p[0],p[1],p[2],p[3],plane(4,0));break;
    case 10:{gp_Pnt a(9,9,9),b(8,8,8);DsgPrs_EqualDistancePresentation::AddInterval(prs,dr,p[0],p[1],d[0],p[2],s,a,b);ga_pnt(a);ga_pnt(b);break;}
    case 11:{gp_Circ c1=circ(0,0,v[0]),c2=circ(1,0,v[1]);
      DsgPrs_EqualDistancePresentation::AddIntervalBetweenTwoArcs(prs,dr,c1,c2,ElCLib::Value(v[2],c1),ElCLib::Value(v[3],c1),ElCLib::Value(v[2],c2),ElCLib::Value(v[3],c2),s);break;}
    case 12:DsgPrs_EqualRadiusPresentation::Add(prs,dr,p[0],p[1],p[2],p[3],plane(4,0));break;
    case 13:{gp_Pnt dp(1,1,1),e(2,2,2);occ::handle<Geom_TrimmedCurve> tc;bool hc=true;
      DsgPrs_FilletRadiusPresentation::Add(prs,dr,v[0],t,p[0],d[0],p[1],p[2],p[3],p[4],s,f[0],dp,e,tc,hc);
      ga_pnt(dp);ga_pnt(e);ga_flag(hc);ga_flag(tc.IsNull());if(!tc.IsNull()){ga_real(tc->FirstParameter());ga_real(tc->LastParameter());}break;}
    case 14:DsgPrs_FixPresentation::Add(prs,dr,p[0],p[1],d[0],v[0]);break;
    case 15:DsgPrs_IdenticPresentation::Add(prs,dr,t,p[0],p[1]);break;
    case 16:DsgPrs_IdenticPresentation::Add(prs,dr,t,p[0],p[1],p[2]);break;
    case 17:DsgPrs_IdenticPresentation::Add(prs,dr,t,gp_Ax2(p[4],d[0]),p[0],p[1],p[2],p[3]);break;
    case 18:DsgPrs_IdenticPresentation::Add(prs,dr,t,gp_Ax2(p[4],d[0]),p[0],p[1],p[2],p[3],p[4]);break;
    case 19:DsgPrs_IdenticPresentation::Add(prs,dr,t,elips(),p[0],p[1],p[3],p[4]);break;
    case 20:DsgPrs_LengthPresentation::Add(prs,dr,t,p[0],p[1],d[0],p[2]);break;
    case 21:DsgPrs_LengthPresentation::Add(prs,dr,t,p[0],p[1],d[0],p[2],s);break;
    case 22:DsgPrs_LengthPresentation::Add(prs,dr,t,p[0],p[1],gp_Pln(p[3],d[1]),d[0],p[2],s);break;
    case 23:DsgPrs_LengthPresentation::Add(prs,dr,t,new Geom_SphericalSurface(gp_Ax3(p[4],d[2]),v[0]),p[0],p[1],d[0],p[2],s);break;
    case 24:DsgPrs_LengthPresentation::Add(prs,dr,p[0],p[1],s);break;
    case 25:DsgPrs_MidPointPresentation::Add(prs,dr,gp_Ax2(p[0],d[0]),p[1],p[2],p[3],f[0]);break;
    case 26:DsgPrs_MidPointPresentation::Add(prs,dr,gp_Ax2(p[0],d[0]),p[1],p[2],p[3],p[4],p[0],f[0]);break;
    case 27:DsgPrs_MidPointPresentation::Add(prs,dr,circ(0,0,v[0]),p[1],p[2],p[3],p[4],p[0],f[0]);break;
    case 28:DsgPrs_MidPointPresentation::Add(prs,dr,elips(),p[1],p[0],p[3],p[4],p[1],f[0]);break;
    case 29:DsgPrs_OffsetPresentation::Add(prs,dr,t,p[0],p[1],d[0],d[1],p[2]);break;
    case 30:DsgPrs_OffsetPresentation::AddAxes(prs,dr,t,p[0],p[1],d[0],d[1],p[2]);break;
    case 31:DsgPrs_ParalPresentation::Add(prs,dr,t,p[0],p[1],d[0],p[2]);break;
    case 32:DsgPrs_ParalPresentation::Add(prs,dr,t,p[0],p[1],d[0],p[2],s);break;
    case 33:DsgPrs_PerpenPresentation::Add(prs,dr,p[0],p[1],p[2],p[3],p[4],f[0],f[1]);break;
    case 34:DsgPrs_RadiusPresentation::Add(prs,dr,t,p[0],circ(1,0,v[0]),v[1],v[2],f[0],f[1]);break;
    case 35:DsgPrs_RadiusPresentation::Add(prs,dr,t,p[0],p[1],p[2],s,f[0],f[1]);break;
    case 36:DsgPrs_SymmetricPresentation::Add(prs,dr,p[0],p[1],d[0],gp_Lin(p[2],d[1]),p[3]);break;
    case 37:DsgPrs_SymmetricPresentation::Add(prs,dr,p[0],p[1],circ(4,0,v[0]),gp_Lin(p[2],d[1]),p[3]);break;
    case 38:DsgPrs_SymmetricPresentation::Add(prs,dr,p[0],p[1],gp_Lin(p[2],d[1]),p[3]);break;
    case 39:DsgPrs_TangentPresentation::Add(prs,dr,p[0],d[0],v[0]);break;
    case 40:DsgPrs_SymbPresentation::Add(prs,dr,t,p[0]);break;
    case 41:DsgPrs_AnglePresentation::Add(prs,dr,v[0],p[0],p[1],p[2],d[0],d[1],p[3]);break;
    case 42:DsgPrs_AnglePresentation::Add(prs,dr,v[0],t,p[0],p[1],p[2],d[0],d[1],p[3]);break;
    case 43:DsgPrs_AnglePresentation::Add(prs,dr,v[0],t,p[0],p[1],p[2],d[0],d[1],d[2],p[3]);break;
    case 44:DsgPrs_AnglePresentation::Add(prs,dr,v[0],t,p[0],p[1],p[2],d[0],d[1],p[3],s);break;
    case 45:DsgPrs_AnglePresentation::Add(prs,dr,v[0],t,p[0],p[1],p[2],d[0],d[1],d[2],f[0],gp_Ax1(p[4],d[2]),p[3],s);break;
    case 46:DsgPrs_AnglePresentation::Add(prs,dr,v[0],p[0],p[1],gp_Ax1(p[0],d[0]),s);break;
    case 47:{gp_Ax2 ax(p[0],d[0]);gp_Circ c(ax,v[0]);gp_Circ cmin(gp_Ax2(p[1],d[0]),v[1]);gp_Circ cmax(gp_Ax2(p[2],d[0]),v[2]);
      DsgPrs_AnglePresentation::Add(prs,dr,0.0,f[0]?TCollection_ExtendedString():t,c,p[3],p[4],cmin,cmax,v[3]);break;}
    case 49:DsgPrs_ShapeDirPresentation::Add(prs,dr,BRepBuilderAPI_MakeEdge(p[0],p[1]).Edge(),int(v[3]));break;
    case 50:case 51:case 52:{BRepBuilderAPI_MakePolygon poly(p[0],p[1],p[2],true);TopoDS_Shape sh=poly.Wire();
      if(in.kind>50)sh=BRepBuilderAPI_MakeFace(poly.Wire(),true).Face();
      if(in.kind>51){BRep_Builder b;TopoDS_Compound c;b.MakeCompound(c);b.Add(c,sh);b.Add(c,BRepBuilderAPI_MakeFace(BRepBuilderAPI_MakePolygon(p[2],p[3],p[4],true).Wire(),true).Face());sh=c;}
      if(f[1])sh.Reverse();
      DsgPrs_ShapeDirPresentation::Add(prs,dr,sh,int(v[3]));break;}
    default:DsgPrs::ComputeSymbol(prs,dr->DimensionAspect(),p[0],p[1],d[0],d[1],s,f[0]);
  }}
void v3_dsgprs(std::istream& input){
  GaReader r{input};V3DsgInput in=v3_dsg_input(r);
  if(in.kind>=100){v3_dsg_package(in);ga_end();return;}
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  occ::handle<Prs3d_Drawer> dr=new Prs3d_Drawer();dr->SetupOwnDefaults();dr->DimensionAspect()->ArrowAspect()->SetLength(in.arrow);
  GA_TRY(v3_dsg_call(prs,dr,in))
  v3_display_trace(trace);v3_dsg_texts(trace);ga_int(prs->Groups().Size());
  ga_real(dr->DimensionAspect()->ArrowAspect()->Length());ga_int(int(dr->DimensionAspect()->LineAspect()->Aspect()->Type()));
  ga_end();}
