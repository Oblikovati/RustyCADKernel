// The ChFiKPart helpers that run without the TKBool data structure.
#include <ChFiKPart_ComputeData_CS.hxx>
#include <ChFiKPart_ComputeData_Fcts.hxx>
#include <gp_Circ.hxx>
#include <gp_Cylinder.hxx>

// `fl_kpart_fcts u f l eps UV1 UV2 pardeb parfin GUIDE gf gl SURFACE`: ChFiKPart_InPeriod,
// ChFiKPart_PCurve and ChFiKPart_ProjPC of the guide on [gf, gl] on the surface.
void fl_kpart_fcts_case(std::istream& in){
 GaReader r{in};double u=r.real(),f=r.real(),l=r.real(),eps=r.real();gp_Pnt2d uv1=r.pnt2d(),uv2=r.pnt2d();double pd=r.real(),pf=r.real();
 occ::handle<Geom_Curve> c=fl_geom_guide(r);double gf=r.real(),gl=r.real();occ::handle<Geom_Surface> s=fl_geom_surface(r);
 ga_real(ChFiKPart_InPeriod(u,f,l,eps));
 GA_TRY(fl_curve2d(ChFiKPart_PCurve(uv1,uv2,pd,pf)));
 GA_TRY(occ::handle<Geom2d_Curve> pc;ChFiKPart_ProjPC(GeomAdaptor_Curve(c,gf,gl),GeomAdaptor_Surface(s),pc);fl_curve2d(pc));
 ga_end();}

// `fl_corner_spine PLANE SURFACE P1S1 P1S2 P2S2 R`: ChFiKPart_CornerSpine.
void fl_corner_spine_case(std::istream& in){
 GaReader r{in};occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);gp_Pnt2d p1s1=r.pnt2d(),p1s2=r.pnt2d(),p2s2=r.pnt2d();double rad=r.real();
 GA_TRY(gp_Cylinder cyl;gp_Circ circ;double first=-1,last=-1;ChFiKPart_CornerSpine(s1,s2,p1s1,p1s1,p1s2,p2s2,rad,cyl,circ,first,last);
  ga_pnt(cyl.Location());ga_dir(cyl.Position().Direction());ga_dir(cyl.Position().XDirection());ga_dir(cyl.Position().YDirection());ga_real(cyl.Radius());
  ga_pnt(circ.Location());ga_dir(circ.Axis().Direction());ga_dir(circ.XAxis().Direction());ga_real(circ.Radius());ga_real(first);ga_real(last));
 ga_end();}

#include <ChFiKPart_ComputeData.hxx>
#include <ChFiKPart_ComputeData_ChAsymPlnCon.hxx>
#include <ChFiKPart_ComputeData_ChAsymPlnCyl.hxx>
#include <ChFiKPart_ComputeData_ChAsymPlnPln.hxx>
#include <ChFiKPart_ComputeData_ChPlnCon.hxx>
#include <ChFiKPart_ComputeData_ChPlnCyl.hxx>
#include <ChFiKPart_ComputeData_ChPlnPln.hxx>
#include <ChFiKPart_ComputeData_FilPlnCon.hxx>
#include <ChFiKPart_ComputeData_FilPlnCyl.hxx>
#include <ChFiKPart_ComputeData_FilPlnPln.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <TopOpeBRepDS_DataStructure.hxx>

// A surface of the data structure: its kind and three points.
void fl_kpart_surface(const occ::handle<Geom_Surface>& s){
 ga_flag(s.IsNull());if(s.IsNull())return;GeomAdaptor_Surface a(s);ga_int(int(a.GetType()));
 GA_TRY(ga_pnt(s->Value(0.2,0.3));ga_pnt(s->Value(1.1,0.7));ga_pnt(s->Value(2.0,-0.4)));}
// The surface data written by a ChFiKPart function and the geometry it put in the data
// structure.
void fl_kpart_data(const TopOpeBRepDS_DataStructure& ds,const occ::handle<ChFiDS_SurfData>& d){
 ga_int(ds.NbSurfaces());ga_int(ds.NbCurves());ga_int(d->Surf());ga_int(int(d->Orientation()));
 if(d->Surf()>0){fl_kpart_surface(ds.Surface(d->Surf()).Surface());ga_real(ds.Surface(d->Surf()).Tolerance());}
 for(int i=1;i<=2;++i){const ChFiDS_FaceInterference& fi=d->Interference(i);ga_int(fi.LineIndex());
  if(fi.LineIndex()>0){fl_curve3d(ds.Curve(fi.LineIndex()).Curve());ga_real(ds.Curve(fi.LineIndex()).Tolerance());}
  ga_int(int(fi.Transition()));ga_real(fi.FirstParameter());ga_real(fi.LastParameter());
  fl_curve2d(fi.PCurveOnFace());fl_curve2d(fi.PCurveOnSurf());}
 for(int f=1;f>=0;--f)for(int i=1;i<=2;++i){fl_common_point(d->Vertex(f==1,i));ga_pnt2d(d->Get2dPoints(f==1,i));}}

// `fl_kpart_make op PLANE SURFACE or1 or2 ofpl GUIDE first a b fu lu plandab dison`: the
// fillet (op 0), the chamfer of mode op-1 (op 1..3, distances a b) or the asymmetric
// chamfer (op 4, distance a and angle b) between a plane and a plane, a cylinder (guide
// line or circle) or a cone (guide circle).
void fl_kpart_make_case(std::istream& in){
 GaReader r{in};int op=r.integer();occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);
 TopAbs_Orientation or1=TopAbs_Orientation(r.integer()),or2=TopAbs_Orientation(r.integer()),ofpl=TopAbs_Orientation(r.integer());
 occ::handle<Geom_Curve> g=fl_geom_guide(r);double first=r.real(),a=r.real(),b=r.real(),fu=r.real(),lu=r.real();bool plandab=r.flag(),dison=r.flag();
 TopOpeBRepDS_DataStructure ds;occ::handle<ChFiDS_SurfData> d=new ChFiDS_SurfData();
 occ::handle<Geom_Line> gl=occ::down_cast<Geom_Line>(g);occ::handle<Geom_Circle> gc=occ::down_cast<Geom_Circle>(g);
 gp_Pln pl=s1->Plane();ChFiDS_ChamfMode mode=ChFiDS_ChamfMode(op>0&&op<4?op-1:0);bool done=false;
 GA_TRY(
  if(s2->GetType()==GeomAbs_Plane){gp_Pln p2=s2->Plane();gp_Lin l=gl->Lin();
   if(op==0)done=ChFiKPart_MakeFillet(ds,d,pl,p2,or1,or2,a,l,first,ofpl);
   else if(op<4)done=ChFiKPart_MakeChamfer(ds,d,mode,pl,p2,or1,or2,a,b,l,first,ofpl);
   else done=ChFiKPart_MakeChAsym(ds,d,pl,p2,or1,or2,a,b,l,first,ofpl,dison);}
  else if(s2->GetType()==GeomAbs_Cylinder&&!gl.IsNull()){gp_Cylinder c=s2->Cylinder();gp_Lin l=gl->Lin();
   if(op==0)done=ChFiKPart_MakeFillet(ds,d,pl,c,fu,lu,or1,or2,a,l,first,ofpl,plandab);
   else if(op<4)done=ChFiKPart_MakeChamfer(ds,d,mode,pl,c,fu,lu,or1,or2,a,b,l,first,ofpl,plandab);
   else done=ChFiKPart_MakeChAsym(ds,d,pl,c,fu,lu,or1,or2,a,b,l,first,ofpl,plandab,dison);}
  else if(s2->GetType()==GeomAbs_Cylinder){gp_Cylinder c=s2->Cylinder();gp_Circ l=gc->Circ();
   if(op==0)done=ChFiKPart_MakeFillet(ds,d,pl,c,fu,lu,or1,or2,a,l,first,ofpl,plandab);
   else if(op<4)done=ChFiKPart_MakeChamfer(ds,d,mode,pl,c,fu,lu,or1,or2,a,b,l,first,ofpl,plandab);
   else done=ChFiKPart_MakeChAsym(ds,d,pl,c,fu,lu,or1,or2,a,b,l,first,ofpl,plandab,dison);}
  else {gp_Cone c=s2->Cone();gp_Circ l=gc->Circ();
   if(op==0)done=ChFiKPart_MakeFillet(ds,d,pl,c,fu,lu,or1,or2,a,l,first,ofpl,plandab);
   else if(op<4)done=ChFiKPart_MakeChamfer(ds,d,mode,pl,c,fu,lu,or1,or2,a,b,l,first,ofpl,plandab);
   else done=ChFiKPart_MakeChAsym(ds,d,pl,c,fu,lu,or1,or2,a,b,l,first,ofpl,plandab,dison);}
  ga_flag(done);fl_kpart_data(ds,d));
 ga_end();}

// `fl_kpart_corner kind S1 S2 S3 of1 of2 o1 o2 o3 r1 r2 P1 P2 P3 P4`:
// ChFiKPart_ComputeData::ComputeCorner, the torus (kind 0), the sphere (kind 1) or the
// torus joint of three planes (kind 2).
void fl_kpart_corner_case(std::istream& in){
 GaReader r{in};int kind=r.integer();occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r),s3=fl_surface(r);
 TopAbs_Orientation o[5];for(auto& x:o)x=TopAbs_Orientation(r.integer());double r1=r.real(),r2=r.real();
 gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d(),p3=r.pnt2d(),p4=r.pnt2d();
 TopOpeBRepDS_DataStructure ds;occ::handle<ChFiDS_SurfData> d=new ChFiDS_SurfData();bool done=false;
 GA_TRY(
  if(kind==0)done=ChFiKPart_ComputeData::ComputeCorner(ds,d,s1,s2,o[0],o[1],o[2],o[3],r1,r2,p1,p2,p3,p4);
  else if(kind==1)done=ChFiKPart_ComputeData::ComputeCorner(ds,d,s1,s2,o[0],o[1],o[2],o[3],r1,p1,p2,p3);
  else done=ChFiKPart_ComputeData::ComputeCorner(ds,d,s1,s2,s3,o[0],o[2],o[3],o[4],r1);
  ga_flag(done);fl_kpart_data(ds,d));
 ga_end();}
