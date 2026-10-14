// StdPrs face and surface presentations: StdPrs_ShapeTool, the restricted face isolines
// (fixed and deflection), the adaptor surface wireframes and shading, the pole presentations,
// StdPrs_ToolVertex and the HLR shapes, the twin of crates/tkv3d/examples/v3_probe/std_prs_faces.rs.
// A request is `v3_stdprs_face Op Face Values...` (the ops and faces below).
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Builder.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Plane.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Graphic3d_Camera.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <StdPrs_HLRPolyShape.hxx>
#include <StdPrs_HLRShape.hxx>
#include <StdPrs_PoleCurve.hxx>
#include <StdPrs_ShadedSurface.hxx>
#include <StdPrs_ShapeTool.hxx>
#include <StdPrs_ToolVertex.hxx>
#include <StdPrs_WFDeflectionRestrictedFace.hxx>
#include <StdPrs_WFDeflectionSurface.hxx>
#include <StdPrs_WFPoleSurface.hxx>
#include <StdPrs_WFRestrictedFace.hxx>
#include <StdPrs_WFSurface.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>

// Face: 0 plane patch, 1 full cylinder (seam), 2 half cylinder, 3 sphere, 4 cone patch,
// 5 torus, 6 B-spline patch, 7 Bezier patch, 8 infinite plane.
static occ::handle<Geom_Surface> v3_spf_pole_surface(bool bspline){
  NCollection_Array2<gp_Pnt> poles(1,3,1,3);
  for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)poles(i,j)=gp_Pnt(i,j,(i==2&&j==2)?2.:0.1*i*j);
  if(!bspline)return new Geom_BezierSurface(poles);
  NCollection_Array1<double> knots(1,2);knots(1)=0.;knots(2)=1.;NCollection_Array1<int> mults(1,2);mults(1)=3;mults(2)=3;
  return new Geom_BSplineSurface(poles,knots,knots,mults,mults,2,2);
}
static TopoDS_Face v3_spf_face(int face){
  const gp_Ax3 ax(gp_Pnt(0.5,-1.,0.25),gp_Dir(0.,0.,1.),gp_Dir(1.,0.,0.));
  switch(face){
    case 0:return BRepBuilderAPI_MakeFace(gp_Pln(ax),0.,2.,0.,1.).Face();
    case 1:return BRepBuilderAPI_MakeFace(gp_Cylinder(ax,2.),0.,2.*M_PI,0.,3.).Face();
    case 2:return BRepBuilderAPI_MakeFace(gp_Cylinder(ax,2.),0.,M_PI,-1.,1.).Face();
    case 3:return BRepBuilderAPI_MakeFace(gp_Sphere(ax,1.5)).Face();
    case 4:return BRepBuilderAPI_MakeFace(gp_Cone(ax,0.4,1.),0.,4.,0.,2.).Face();
    case 5:return BRepBuilderAPI_MakeFace(gp_Torus(ax,3.,1.)).Face();
    case 6:return BRepBuilderAPI_MakeFace(v3_spf_pole_surface(true),1.e-7).Face();
    case 7:return BRepBuilderAPI_MakeFace(v3_spf_pole_surface(false),1.e-7).Face();
    default:return BRepBuilderAPI_MakeFace(gp_Pln(ax)).Face();
  }
}
static occ::handle<Prs3d_Drawer> v3_spf_drawer(GaReader& r){
  occ::handle<Prs3d_Drawer> dr=new Prs3d_Drawer();dr->SetupOwnDefaults();
  dr->UIsoAspect()->SetNumber(r.integer());dr->VIsoAspect()->SetNumber(r.integer());dr->SetDiscretisation(r.integer());
  dr->SetMaximalChordialDeviation(r.real());dr->EnableDrawHiddenLine();dr->SetMaximalParameterValue(r.real());
  dr->SetTypeOfDeflection(r.flag()?Aspect_TOD_RELATIVE:Aspect_TOD_ABSOLUTE);return dr;}
static void v3_spf_box(const Bnd_Box& b){ga_flag(b.IsVoid());if(!b.IsVoid()){double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);ga_real(x0);ga_real(y0);ga_real(z0);ga_real(x1);ga_real(y1);ga_real(z1);}}
static void v3_spf_curves(const NCollection_List<occ::handle<NCollection_HSequence<gp_Pnt>>>& curves){
  ga_int(curves.Extent());
  for(const occ::handle<NCollection_HSequence<gp_Pnt>>& c:curves){ga_int(c->Length());for(int i=1;i<=c->Length();++i)ga_pnt(c->Value(i));}}
// The shape tool over a compound of the face, a free edge and a free vertex.
static void v3_spf_shape_tool(const TopoDS_Face& f,bool all){
  TopoDS_Compound c;BRep_Builder b;b.MakeCompound(c);b.Add(c,f);
  b.Add(c,BRepBuilderAPI_MakeEdge(gp_Pnt(5,5,5),gp_Pnt(6,5,5)).Edge());b.Add(c,BRepBuilderAPI_MakeVertex(gp_Pnt(7,7,7)).Vertex());
  StdPrs_ShapeTool t(c,all);
  for(t.InitFace();t.MoreFace();t.NextFace()){ga_flag(t.IsPlanarFace());ga_flag(t.HasSurface());v3_spf_box(t.FaceBound());
    TopLoc_Location l;ga_flag(t.CurrentTriangulation(l).IsNull());}
  ga_int(-1);
  for(t.InitCurve();t.MoreCurve();t.NextCurve()){ga_int(t.Neighbours());ga_int(t.FacesOfEdge()->Length());ga_flag(t.HasCurve());v3_spf_box(t.CurveBound());
    TopLoc_Location l;ga_flag(t.Polygon3D(l).IsNull());occ::handle<Poly_PolygonOnTriangulation> pt;occ::handle<Poly_Triangulation> tr;t.PolygonOnTriangulation(pt,tr,l);ga_flag(pt.IsNull());}
  ga_int(-1);
  for(t.InitVertex();t.MoreVertex();t.NextVertex()){double x,y,z;StdPrs_ToolVertex::Coord(t.GetVertex(),x,y,z);ga_real(x);ga_real(y);ga_real(z);}
}
static void v3_spf_pole_curve(GaReader& r,const occ::handle<Prs3d_Drawer>& dr,const occ::handle<Graphic3d_Structure>& prs,int kind){
  NCollection_Array1<gp_Pnt> poles(1,4);for(int i=1;i<=4;++i)poles(i)=gp_Pnt(i,(i%2)*1.5,0.25*i);
  occ::handle<Geom_Curve> c;
  if(kind==0)c=new Geom_BezierCurve(poles);
  else if(kind==1){NCollection_Array1<double> k(1,2);k(1)=0;k(2)=1;NCollection_Array1<int> m(1,2);m(1)=4;m(2)=4;c=new Geom_BSplineCurve(poles,k,m,3);}
  else c=new Geom_Line(gp_Pnt(0,0,0),gp_Dir(1,1,0));
  GeomAdaptor_Curve a(c,0.,1.);dr->SetLineArrowDraw(r.flag());
  GA_TRY(StdPrs_PoleCurve::Add(prs,a,dr))
  for(int i=0;i<4;++i){double x=r.real(),y=r.real(),z=r.real(),d=r.real();GA_TRY(ga_flag(StdPrs_PoleCurve::Match(x,y,z,d,a,dr)))GA_TRY(ga_int(StdPrs_PoleCurve::Pick(x,y,z,d,a,dr)))}
}
void v3_stdprs_face(std::istream& input){
  GaReader r{input};const int op=r.integer(),face=r.integer();
  occ::handle<Prs3d_Drawer> dr=v3_spf_drawer(r);
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  TopoDS_Face f;GA_TRY(f=v3_spf_face(face))
  if(f.IsNull()){ga_end();return;}
  occ::handle<BRepAdaptor_Surface> adaptor=new BRepAdaptor_Surface(f);
  switch(op){
    case 0:GA_TRY(v3_spf_shape_tool(f,r.flag()))break;
    case 1:{bool u=r.flag(),v=r.flag();int nu=r.integer(),nv=r.integer();NCollection_List<occ::handle<NCollection_HSequence<gp_Pnt>>> curves;
      GA_TRY(StdPrs_WFRestrictedFace::Add(prs,adaptor,u,v,nu,nv,dr,curves))v3_spf_curves(curves);break;}
    case 2:for(int i=0;i<4;++i){double x=r.real(),y=r.real(),z=r.real(),d=r.real();
      GA_TRY(ga_flag(StdPrs_WFRestrictedFace::Match(x,y,z,d,adaptor,dr)))GA_TRY(ga_flag(StdPrs_WFRestrictedFace::MatchUIso(x,y,z,d,adaptor,dr)))
      GA_TRY(ga_flag(StdPrs_WFRestrictedFace::MatchVIso(x,y,z,d,adaptor,dr)))}break;
    case 3:{bool u=r.flag(),v=r.flag();int nu=r.integer(),nv=r.integer();double defl=r.real();NCollection_List<occ::handle<NCollection_HSequence<gp_Pnt>>> curves;
      GA_TRY(StdPrs_WFDeflectionRestrictedFace::Add(prs,adaptor,u,v,defl,nu,nv,dr,curves))v3_spf_curves(curves);break;}
    case 4:for(int i=0;i<4;++i){double x=r.real(),y=r.real(),z=r.real(),d=r.real();
      GA_TRY(ga_flag(StdPrs_WFDeflectionRestrictedFace::Match(x,y,z,d,adaptor,dr)))GA_TRY(ga_flag(StdPrs_WFDeflectionRestrictedFace::MatchUIso(x,y,z,d,adaptor,dr)))
      GA_TRY(ga_flag(StdPrs_WFDeflectionRestrictedFace::MatchVIso(x,y,z,d,adaptor,dr)))}break;
    case 5:GA_TRY(StdPrs_WFSurface::Add(prs,adaptor,dr))break;
    case 6:GA_TRY(StdPrs_WFDeflectionSurface::Add(prs,adaptor,dr))break;
    case 7:GA_TRY(StdPrs_ShadedSurface::Add(prs,*adaptor,dr))break;
    case 8:GA_TRY(StdPrs_WFPoleSurface::Add(prs,*adaptor,dr))break;
    case 9:GA_TRY(v3_spf_pole_curve(r,dr,prs,face%3))break;
    case 10:case 11:{occ::handle<Graphic3d_Camera> cam=new Graphic3d_Camera();cam->SetProjectionType(r.flag()?Graphic3d_Camera::Projection_Orthographic:Graphic3d_Camera::Projection_Perspective);
      const gp_Pnt eye=r.pnt();const gp_Pnt center=r.pnt();cam->SetEyeAndCenter(eye,center);cam->SetUp(r.dir());cam->SetScale(r.real());
      if(op==11)BRepMesh_IncrementalMesh(f,0.1);
      occ::handle<StdPrs_HLRShapeI> hlr=op==10?occ::handle<StdPrs_HLRShapeI>(new StdPrs_HLRShape()):occ::handle<StdPrs_HLRShapeI>(new StdPrs_HLRPolyShape());
      GA_TRY(hlr->ComputeHLR(prs,f,dr,cam))break;}
  }
  v3_display_trace(trace);ga_int(prs->Groups().Size());ga_end();
}
