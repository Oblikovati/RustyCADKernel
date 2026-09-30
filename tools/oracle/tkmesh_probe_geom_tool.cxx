#include <BRepMesh_GeomTool.hxx>
static std::pair<gp_XY,gp_XY> geom_segment(int index){
  const double values[][4]={{-1,-1,1,1},{-1,1,1,-1},{0,0,2,0},{1,0,3,0},{2,0,3,0},{0,0,0,0},{0,0,0,1},{1,0,1,1},{1e-10,0,1e-10,1},{1e-8,0,1e-8,1},{1,1,2,1},{1,0,0,0},{-0.,-0.,1e-200,1e-200},{1e5,1e-10,-1e5,1e-10},{1e-10,0,1,1e-10}};
  auto v=values[index];return {gp_XY(v[0],v[1]),gp_XY(v[2],v[3])};
}
void mesh_geom_intersection(std::istream& input){
  int i,j,touch,on;input>>i>>j>>touch>>on;auto[a,b]=geom_segment(i);auto[c,d]=geom_segment(j);gp_XY point(17,19);double parameters[]={23,29};
  ga_int(BRepMesh_GeomTool::IntLinLin(a,b,c,d,point,parameters));ga_real(point.X());ga_real(point.Y());ga_real(parameters[0]);ga_real(parameters[1]);gp_Pnt2d p(17,19);
  ga_int(BRepMesh_GeomTool::IntSegSeg(a,b,c,d,touch,on,p));ga_pnt2d(p);ga_real(BRepMesh_GeomTool::SquareDeflectionOfSegment(gp_Pnt(a.X(),a.Y(),0),gp_Pnt(b.X(),b.Y(),0),gp_Pnt(c.X(),c.Y(),.3)));ga_end();
}
void mesh_geom_surface(std::istream& input){
  int kind,orient,coord,bound,vert,def;input>>kind>>orient>>coord>>bound>>vert>>def;auto face=range_face(kind,1e-7);auto topology=face->GetFace();topology.Orientation(TopAbs_Orientation(orient));occ::handle<BRepAdaptor_Surface> surface=new BRepAdaptor_Surface(topology);
  const double coords[][2]={{.3,.7},{0.,-2./std::sin(.3)},{1.,1.}};gp_Pnt point(11,13,17);gp_Dir normal(1,0,0);ga_flag(BRepMesh_GeomTool::Normal(surface,coords[coord][0],coords[coord][1],point,normal));ga_pnt(point);ga_real(normal.X());ga_real(normal.Y());ga_real(normal.Z());
  const double bounds[][4]={{0,1,0,1},{-1,8,-2,3},{.2,.7,.1,.8},{0,0,0,1},{0,1,-9,-7},{0,.001,0,.002}};
  BRepMesh_DefaultRangeSplitter splitter;splitter.Reset(face,IMeshTools_Parameters());splitter.AddPoint(gp_Pnt2d(bounds[bound][0],bounds[bound][2]));splitter.AddPoint(gp_Pnt2d(bounds[bound][1],bounds[bound][3]));splitter.AdjustRange();const int verts[]={0,1,4,31,1000};const double defs[]={0,.01,1.};auto counts=BRepMesh_GeomTool::CellsCount(surface,verts[vert],defs[def],&splitter);ga_int(counts.first);ga_int(counts.second);counts=BRepMesh_GeomTool::CellsCount(surface,0,0,nullptr);ga_int(counts.first);ga_int(counts.second);ga_end();
}
void mesh_geom_sampling(std::istream& input){
  int kind,located,iso,replace;input>>kind>>located>>iso>>replace;auto[edge,face]=refinement_fixture(kind,located,false);auto surface=face->GetSurface();BRepAdaptor_Curve adaptor(edge->GetEdge());
  std::unique_ptr<BRepMesh_GeomTool> tool;if(!iso)tool=std::make_unique<BRepMesh_GeomTool>(adaptor,0.,1.,.1,.5,2,1e-7);else tool=std::make_unique<BRepMesh_GeomTool>(surface,iso==1?GeomAbs_IsoU:GeomAbs_IsoV,.5,0.,1.,.1,.5,2,1e-7);
  ga_int(tool->NbPoints());ga_int(tool->AddPoint(gp_Pnt(7,8,9),.5,replace));ga_int(tool->AddPoint(gp_Pnt(2,3,4),.5,replace));ga_int(tool->NbPoints());
  for(int index=0;index<=tool->NbPoints()+1;++index){double parameter=23.;gp_Pnt point(11,13,17);gp_Pnt2d uv(17,19);ga_flag(tool->Value(index,.37,parameter,point,uv));ga_real(parameter);ga_pnt(point);ga_pnt2d(uv);ga_flag(tool->Value(index,surface,parameter,point,uv));ga_real(parameter);ga_pnt(point);ga_pnt2d(uv);}ga_end();
}
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <GeomAdaptor_Surface.hxx>
static occ::handle<Geom_Surface> cells_surface(int kind){
 if(kind==0)return new Geom_Plane(gp_Pln());if(kind==1)return new Geom_SphericalSurface(gp_Ax3(),2.);if(kind==2)return new Geom_ToroidalSurface(gp_Ax3(),3.,1.);
 NCollection_Array1<double> knots(1,2);knots(1)=0;knots(2)=1;NCollection_Array1<int> mults(1,2);mults.Init(4);
 if(kind<6){int nu=kind==4?2:4,nv=kind==3?2:4;NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);for(int u=1;u<=nu;++u)for(int v=1;v<=nv;++v)poles(u,v)=gp_Pnt(u,v,((u*v)%3)*.25);if(kind!=5)return new Geom_BezierSurface(poles);return new Geom_BSplineSurface(poles,knots,knots,mults,mults,3,3,false,false);}
 occ::handle<Geom_Curve> curve;if(kind<8)curve=new Geom_Line(gp_Ax1(gp_Pnt(2,0,0),gp_Dir(0,0,1)));else{int degree=kind<10?3:1;NCollection_Array1<gp_Pnt> poles(1,degree+1);for(int i=1;i<=degree+1;++i)poles(i)=gp_Pnt(2+(i%2)*.25,0,i);mults.Init(degree+1);curve=new Geom_BSplineCurve(poles,knots,mults,degree,false);}
 if(kind%2==0)return new Geom_SurfaceOfLinearExtrusion(curve,gp_Dir(0,1,0));return new Geom_SurfaceOfRevolution(curve,gp_Ax1());
}
class CellsRangeFixture:public BRepMesh_DefaultRangeSplitter {public:CellsRangeFixture(){myRangeU={0,3};myRangeV={-1,2};myDelta={.25,.75};}};
void mesh_geom_cells(std::istream& input){
 int kind,vert,def;input>>kind>>vert>>def;occ::handle<Adaptor3d_Surface> surface=kind==12?new GeomAdaptor_Surface():new GeomAdaptor_Surface(cells_surface(kind));CellsRangeFixture splitter;const int vs[]={-1,0,1,4,31,1000};const double ds[]={0,1e-5,.01,1};auto counts=BRepMesh_GeomTool::CellsCount(surface,vs[vert],ds[def],&splitter);ga_int(counts.first);ga_int(counts.second);ga_end();
}
void mesh_range_overflow(std::istream&){
 auto face=range_face(0,1e-7);face->SetDeflection(.1);BRepMesh_ConeRangeSplitter splitter;IMeshTools_Parameters p;p.MinSize=.001;splitter.Reset(face,p);splitter.AddPoint(gp_Pnt2d(0,0));splitter.AddPoint(gp_Pnt2d(1e100,1));std::pair<int,int> counts;auto steps=splitter.GetSplitSteps(p,counts);ga_int(counts.first);ga_int(counts.second);ga_real(steps.first);ga_real(steps.second);ga_end();
}
