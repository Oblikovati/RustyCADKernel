// All IntTools_TopolTool surface branches, explicit samples and retained arrays on reinitialization.
#include <IntTools_TopolTool.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <NCollection_Array2.hxx>
occ::handle<Geom_Surface> bo_topol_spline(int n,bool bspline) {
 NCollection_Array2<gp_Pnt> poles(1,n,1,n);
 for(int i=1;i<=n;++i)for(int j=1;j<=n;++j)poles(i,j)=gp_Pnt(double(i),double(j),(i+j)%2==0?1.0:-1.0);
 if(!bspline)return new Geom_BezierSurface(poles);
 NCollection_Array1<double> knots(1,2);knots(1)=0.;knots(2)=1.;NCollection_Array1<int> mult(1,2);mult.Init(n);
 return new Geom_BSplineSurface(poles,knots,knots,mult,mult,n-1,n-1,false,false);
}
occ::handle<Adaptor3d_Surface> bo_topol_surface(const std::vector<double>& v) {
 occ::handle<Geom_Surface> s;int kind=int(v[0]);double r=v[1];
 switch(kind) {
  case 1:s=new Geom_CylindricalSurface(gp_Ax3(),r);break;case 2:s=new Geom_ConicalSurface(gp_Ax3(),0.3,r);break;
  case 3:s=new Geom_SphericalSurface(gp_Ax3(),r);break;case 4:s=new Geom_ToroidalSurface(gp_Ax3(),r*2.,r);break;
  case 5:s=new Geom_SurfaceOfLinearExtrusion(new Geom_Line(gp_Ax1()),gp_Dir(1.,0.,0.));break;
  case 6:s=new Geom_SurfaceOfRevolution(new Geom_Line(gp_Ax1()),gp_Ax1(gp_Pnt(1.,0.,0.),gp_Dir(1.,0.,0.)));break;
  case 7:case 8:s=bo_topol_spline(int(r),kind==8);break;default:s=new Geom_Plane(gp_Ax3());break;
 }
 return new GeomAdaptor_Surface(s,v[2],v[3],v[4],v[5],0.,0.);
}
void bo_topol_point(IntTools_TopolTool& t,int i) {
 try {gp_Pnt2d uv;gp_Pnt p;t.SamplePoint(i,uv,p);bo_real(uv.X());bo_real(uv.Y());bo_real(p.X());bo_real(p.Y());bo_real(p.Z());}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
void bo_topol_state(IntTools_TopolTool& t) {
 bo_int(t.NbSamplesU());bo_int(t.NbSamplesV());bo_int(t.NbSamples());
 for(int index:{1,t.NbSamplesU(),t.NbSamples()-1})bo_topol_point(t,index);
}
void bo_topol_parameters(IntTools_TopolTool& t) {
 NCollection_Array1<double> u,v;t.UParameters(u);t.VParameters(v);
 for(const auto* a:{&u,&v}) {bo_int(a->Length());for(double value:*a)bo_real(value);}
}
void bo_topol_vertices(Adaptor3d_TopolTool& t,const occ::handle<Adaptor2d_Curve2d>& arc) {
 t.Initialize(arc);t.InitVertexIterator();while(t.MoreVertex()){auto vertex=t.Vertex();bo_int(t.Orientation(vertex));bo_int(t.Identical(vertex,vertex));
 try{bo_real(t.Tol3d(vertex));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}t.NextVertex();}bo_int(t.MoreVertex());
}
void bo_topol_domain(Adaptor3d_TopolTool& t) {
 bo_int(t.DomainIsInfinite());bo_int(t.Has3d());bo_int(t.Classify(gp_Pnt2d(),1e-7,true));bo_int(t.IsThePointOn(gp_Pnt2d(),1e-7,true));bo_int(t.IsUniformSampling());t.Init();
 while(t.More()){auto arc=t.Value();bo_int(t.Orientation(arc));bo_int(t.Edge()!=nullptr);try{bo_real(t.Tol3d(arc));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_topol_vertices(t,arc);t.Next();}
 bo_int(t.More());t.SamplePnts(0.1,4,5);NCollection_Array1<double> u,v;t.UParameters(u);t.VParameters(v);bo_real(u(1));bo_real(v(1));
}
void bo_topol_case(const std::vector<double>& v) {
 auto s=bo_topol_surface(v);IntTools_TopolTool t(s);
 try{t.Initialize();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 bo_int(t.More());bo_topol_state(t);t.ComputeSamplePoints();bo_topol_state(t);
 t.SamplePnts(0.1,4,5);bo_topol_state(t);bo_topol_parameters(t);
 t.Initialize(s);bo_topol_point(t,1);bo_topol_state(t);bo_topol_parameters(t);
 t.SamplePnts(0.2,6,7);bo_topol_state(t);bo_topol_domain(t);
}
