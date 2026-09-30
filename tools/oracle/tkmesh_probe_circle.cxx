#define MC_TRY(expr) try {expr;std::cout<<"OK ";}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
#include <BRepMesh_CircleTool.hxx>
#include <BRepMesh_CircleInspector.hxx>
#include <gp_Circ2d.hxx>
#include <cstdlib>
static double mesh_circle_scalar(int index) {double values[]={-1.,-0.,0.,1e-9,0.1,1.,2.,NAN,INFINITY};return values[index];}
static gp_XY mesh_circle_point(int index) {double x=mesh_circle_scalar(index);return gp_XY(x,index%2==0?x:0.);}
static void circle_report(const BRepMesh_Circle& circle) {ga_real(circle.Location().X());ga_real(circle.Location().Y());ga_real(circle.Radius());}
static void circle_hits(const IMeshData::ListOfInteger& hits) {ga_int(hits.Extent());for(IMeshData::ListOfInteger::Iterator it(hits);it.More();it.Next())ga_int(it.Value());}
void mesh_circle_inspector(std::istream& input) {
 std::array<int,5>v;for(auto& value:v)input>>value;
 occ::handle<NCollection_IncAllocator> arena=new NCollection_IncAllocator;BRepMesh_CircleInspector inspector(mesh_circle_scalar(v[0]),3,arena);
 BRepMesh_Circle circle;circle_report(circle);circle.SetLocation(mesh_circle_point(v[1]));circle.SetRadius(mesh_circle_scalar(v[2]));circle_report(circle);
 MC_TRY(inspector.Bind(v[3],circle));ga_int(inspector.Circles().Length());inspector.SetPoint(mesh_circle_point(v[4]));
 for(int i=0;i<inspector.Circles().Length();++i)ga_int(inspector.Inspect(i)==CellFilter_Keep?0:1);
 circle_hits(inspector.GetShotCircles());inspector.GetShotCircles().Append(99);circle_hits(inspector.GetShotCircles());
 inspector.SetPoint(gp_XY());circle_hits(inspector.GetShotCircles());if(v[3]>=0){inspector.Circle(v[3]).SetRadius(2.);circle_report(inspector.Circle(v[3]));}
 ga_flag(BRepMesh_CircleInspector::IsEqual(v[3],0));const auto shifted=BRepMesh_CircleInspector::Shift(mesh_circle_point(v[1]),mesh_circle_scalar(v[0]));ga_real(shifted.X());ga_real(shifted.Y());
 for(int axis:{-1,0,1,2}){try{ga_real(BRepMesh_CircleInspector::Coord(axis,mesh_circle_point(v[1])));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}ga_end();
}
static void circle_queries(BRepMesh_CircleTool& tool) {
 for(auto p:{std::pair<double,double>(-2.,0.),{-1.,0.},{0.,0.},{1.,0.},{2.,0.},{0.,1.},{0.,2.}}) {
  try{auto& hits=tool.Select(gp_XY(p.first,p.second));std::cout<<"OK ";circle_hits(hits);hits.Append(99);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 }
}
static void circle_alter(BRepMesh_CircleTool& tool,int mode) {
 switch(mode) {
 case 0:tool.Delete(1);break;case 1:tool.MocBind(1);break;
 case 2:tool.Bind(1,gp_Circ2d(gp_Ax2d(gp_Pnt2d(),gp_Dir2d(1.,0.)),2.));break;
 case 3:ga_flag(tool.Bind(1,gp_XY(-1.,0.),gp_XY(0.,1.),gp_XY(1.,0.)));break;
 case 4:ga_flag(tool.Bind(1,gp_XY(),gp_XY(),gp_XY()));break;
 case 5:tool.SetCellSize(1.,2.);break;
 }
}
static void circle_tool_body(BRepMesh_CircleTool& tool,const std::array<int,5>& v) {
 ga_flag(tool.IsEmpty());
 if(v[1]<2){MC_TRY(tool.SetCellSize(v[1]==0?1.:-1.));}else{MC_TRY(tool.SetCellSize(0.5,1.));}
 if(v[2]==1)tool.SetMinMaxSize(gp_XY(-3.,-3.),gp_XY(3.,3.));else if(v[2]==2)tool.SetMinMaxSize(gp_XY(1.,1.),gp_XY(-1.,-1.));
 for(int i=0;i<3;++i){MC_TRY(tool.Bind(i,gp_Circ2d(gp_Ax2d(gp_Pnt2d(i-1.,0.),gp_Dir2d(1.,0.)),mesh_circle_scalar(v[3]))));}
 ga_flag(tool.IsEmpty());circle_queries(tool);MC_TRY(circle_alter(tool,v[4]));circle_queries(tool);
 tool.Init(500);ga_flag(tool.IsEmpty());circle_queries(tool);MC_TRY(tool.SetCellSize(1.));ga_flag(tool.IsEmpty());circle_queries(tool);ga_end();
}
void mesh_circle_tool(std::istream& input) {
 std::array<int,5>v;for(auto& value:v)input>>value;occ::handle<NCollection_IncAllocator> arena=new NCollection_IncAllocator;
 if(v[0]==0){BRepMesh_CircleTool tool(arena);circle_tool_body(tool,v);}else{BRepMesh_CircleTool tool(v[0],arena);circle_tool_body(tool,v);}
}
void mesh_circle_make(std::istream& input) {
 int a,b,c;input>>a>>b>>c;double scales[]={0.,1e-155,1e-10,1e-9,1e-8,1.,1e9,1e150,INFINITY,NAN},shifts[]={0.,1.,1e9,-1e9};double scale=scales[a],shift=shifts[b];
 gp_XY points[]={gp_XY(-scale+shift,shift),gp_XY(shift,scale+shift),gp_XY(scale+shift,shift)};
 if(c&1)points[1]=points[0];if(c&2)std::swap(points[0],points[2]);if(c&4)points[1].SetY(shift+scale*1e-12);
 gp_XY center(7.,8.);double radius=9.;ga_flag(BRepMesh_CircleTool::MakeCircle(points[0],points[1],points[2],center,radius));ga_real(center.X());ga_real(center.Y());ga_real(radius);ga_end();
}
void mesh_circle_random(std::istream&) {
 srand(42);int accepted=0,failed=0;const double precision=Precision::PConfusion()*Precision::PConfusion(),minArea=5.*M_PI/180.;
 while(accepted<100000) {
  gp_XY p[3];for(auto& point:p)point.SetCoord(static_cast<double>(rand())/RAND_MAX,static_cast<double>(rand())/RAND_MAX);
  const gp_XY first=p[1]-p[0],second=p[2]-p[0];if(first.SquareModulus()<=precision||second.SquareModulus()<=precision||(first^second)<=minArea)continue;
  ++accepted;for(auto& point:p){ga_real(point.X());ga_real(point.Y());}
  gp_XY center;double radius=0.;bool made=BRepMesh_CircleTool::MakeCircle(p[0],p[1],p[2],center,radius);ga_flag(made);ga_real(center.X());ga_real(center.Y());ga_real(radius);
  if(made){bool bad=false;for(auto& point:p)if(!((point-center).SquareModulus()-radius*radius<precision))bad=true;if(bad)++failed;}
 }ga_int(failed);ga_end();
}
void mesh_circle_invalid(std::istream& input) {
 int op,index;input>>op>>index;BRepMesh_CircleInspector inspector(1e-9,4,nullptr);inspector.Bind(0,BRepMesh_Circle());
 try{volatile double observed=0.;if(op==0)observed=inspector.Circle(index).Radius();else observed=inspector.Inspect(index);(void)observed;std::cout<<"OK ";}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}ga_end();
}

#undef MC_TRY
