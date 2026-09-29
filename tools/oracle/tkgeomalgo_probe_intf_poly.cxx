// Intf interference probe operations: Intf_InterferencePolygon2d over a probe polygon,
// and the generic Intf_InterferencePolygonPolyhedron.gxx instantiated here over probe
// polygon and polyhedron classes (as IntCurveSurface and HLRBRep instantiate it). The
// Rust twin is crates/tkgeomalgo/examples/ga_probe/intf_poly.rs.
#include <Bnd_BoundSortBox.hxx>
#include <Intf_InterferencePolygon2d.hxx>
#include <Intf_Polygon2d.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_HArray1.hxx>
#include <Precision.hxx>
// A polyline `closed deflection n x y...`: segment i joins points i and i+1; the box
// holds the points, enlarged by the deflection.
class GaPolygon2d : public Intf_Polygon2d{
public:
 explicit GaPolygon2d(GaReader& r){
  closed=r.flag();deflection=r.real();int n=r.integer();
  for(int i=0;i<n;++i){points.push_back(r.pnt2d());myBox.Add(points.back());}
  myBox.Enlarge(deflection);
 }
 bool Closed() const override{return closed;}
 double DeflectionOverEstimation() const override{return deflection;}
 int NbSegments() const override{return int(points.size())-1;}
 void Segment(const int index,gp_Pnt2d& begin,gp_Pnt2d& end) const override{
  if(index<1||index>NbSegments())throw Standard_OutOfRange("GaPolygon2d::Segment");
  begin=points[index-1];end=points[index];}
private:
 bool closed;double deflection;std::vector<gp_Pnt2d> points;
};
void ga_ipoly2d_print(const Intf_InterferencePolygon2d& interference){
 ga_interference_print(interference);
 for(int i=1;i<=interference.NbSectionPoints()+1;++i)GA_TRY(ga_pnt2d(interference.Pnt2dValue(i)));
}
// Performs on one interference: `0 A B` (Perform of two polygons), `1 A` (Perform of
// one polygon), `2 A B` / `3 A` (the constructors, replacing the interference).
void ga_intf_ipoly2d_case(std::istream& input){
 GaReader r{input};Intf_InterferencePolygon2d interference;int count=r.integer();
 ga_ipoly2d_print(interference);
 for(int k=0;k<count;++k){
  int kind=r.integer();GaPolygon2d first(r);
  if(kind==0||kind==2){GaPolygon2d second(r);
   if(kind==0)GA_TRY(interference.Perform(first,second))else GA_TRY(interference=Intf_InterferencePolygon2d(first,second));}
  else if(kind==1)GA_TRY(interference.Perform(first))
  else GA_TRY(interference=Intf_InterferencePolygon2d(first));
  ga_ipoly2d_print(interference);
 }
 ga_end();
}
// A 3d polyline `closed deflection n x y z...` with its box enlarged by the deflection.
struct GaPolygon3d{
 bool closed;double deflection;std::vector<gp_Pnt> points;Bnd_Box box;
 explicit GaPolygon3d(GaReader& r){
  closed=r.flag();deflection=r.real();int n=r.integer();
  for(int i=0;i<n;++i){points.push_back(r.pnt());box.Add(points.back());}
  box.Enlarge(deflection);
 }
 const gp_Pnt& Point(int index) const{
  if(index<1||index>int(points.size()))throw Standard_OutOfRange("GaPolygon3d::Point");
  return points[index-1];}
};
struct GaPolygon3dTool{
 static const Bnd_Box& Bounding(const GaPolygon3d& p){return p.box;}
 static double DeflectionOverEstimation(const GaPolygon3d& p){return p.deflection;}
 static bool Closed(const GaPolygon3d& p){return p.closed;}
 static int NbSegments(const GaPolygon3d& p){return int(p.points.size())-1;}
 static const gp_Pnt& BeginOfSeg(const GaPolygon3d& p,const int index){return p.Point(index);}
 static const gp_Pnt& EndOfSeg(const GaPolygon3d& p,const int index){return p.Point(index+1);}
};
// A triangulation `deflection border npoints x y z... ntriangles i j k... nbound i j...`:
// each triangle box holds its points enlarged by the deflection; the listed point pairs
// are the boundary edges.
struct GaPolyhedron{
 double deflection,border;std::vector<gp_Pnt> points;std::vector<std::array<int,3>> triangles;
 std::vector<std::pair<int,int>> bounds;Bnd_Box box;occ::handle<NCollection_HArray1<Bnd_Box>> boxes;
 explicit GaPolyhedron(GaReader& r){
  deflection=r.real();border=r.real();int n=r.integer();
  for(int i=0;i<n;++i){points.push_back(r.pnt());box.Add(points.back());}
  box.Enlarge(deflection);int t=r.integer();boxes=new NCollection_HArray1<Bnd_Box>(1,t);
  for(int i=0;i<t;++i){int a=r.integer(),b=r.integer(),c=r.integer();triangles.push_back({a,b,c});
   Bnd_Box& tb=boxes->ChangeValue(i+1);tb.Add(points[a-1]);tb.Add(points[b-1]);tb.Add(points[c-1]);tb.Enlarge(deflection);}
  int m=r.integer();for(int i=0;i<m;++i){int a=r.integer();bounds.push_back({a,r.integer()});}
 }
};
struct GaPolyhedronTool{
 static const Bnd_Box& Bounding(const GaPolyhedron& p){return p.box;}
 static const occ::handle<NCollection_HArray1<Bnd_Box>>& ComponentsBounding(const GaPolyhedron& p){return p.boxes;}
 static double DeflectionOverEstimation(const GaPolyhedron& p){return p.deflection;}
 static void Triangle(const GaPolyhedron& p,const int index,int& a,int& b,int& c){
  if(index<1||index>int(p.triangles.size()))throw Standard_OutOfRange("GaPolyhedron::Triangle");
  a=p.triangles[index-1][0];b=p.triangles[index-1][1];c=p.triangles[index-1][2];}
 static const gp_Pnt& Point(const GaPolyhedron& p,const int index){
  if(index<1||index>int(p.points.size()))throw Standard_OutOfRange("GaPolyhedron::Point");
  return p.points[index-1];}
 // The probe polyhedron has no connectivity: every edge is free.
 static int TriConnex(const GaPolyhedron&,const int,const int,const int,int& triCon,int& otherP){triCon=0;otherP=0;return 0;}
 static bool IsOnBound(const GaPolyhedron& p,const int a,const int b){
  for(const auto& e:p.bounds)if((e.first==a&&e.second==b)||(e.first==b&&e.second==a))return true;return false;}
 static double GetBorderDeflection(const GaPolyhedron& p){return p.border;}
};
// The instance header, as IntCurveSurface_TheInterferenceOfHInter.hxx declares it.
class GaInterferencePP : public Intf_Interference{
public:
 GaInterferencePP();
 GaInterferencePP(const GaPolygon3d& thePolyg,const GaPolyhedron& thePolyh);
 GaInterferencePP(const gp_Lin& theLin,const GaPolyhedron& thePolyh);
 GaInterferencePP(const NCollection_Array1<gp_Lin>& theLins,const GaPolyhedron& thePolyh);
 void Perform(const GaPolygon3d& thePolyg,const GaPolyhedron& thePolyh);
 void Perform(const gp_Lin& theLin,const GaPolyhedron& thePolyh);
 void Perform(const NCollection_Array1<gp_Lin>& theLins,const GaPolyhedron& thePolyh);
 GaInterferencePP(const GaPolygon3d& thePolyg,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 GaInterferencePP(const gp_Lin& theLin,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 GaInterferencePP(const NCollection_Array1<gp_Lin>& theLins,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 void Perform(const GaPolygon3d& thePolyg,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 void Perform(const gp_Lin& theLin,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 void Perform(const NCollection_Array1<gp_Lin>& theLins,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 void Interference(const GaPolygon3d& thePolyg,const GaPolyhedron& thePolyh,Bnd_BoundSortBox& theBoundSB);
 void Interference(const GaPolygon3d& thePolyg,const GaPolyhedron& thePolyh);
private:
 void Intersect(const gp_Pnt& BegO,const gp_Pnt& EndO,const bool Infinite,const int TTri,const GaPolyhedron& thePolyh);
 void Intersect(const gp_Pnt& BegO,const gp_Pnt& EndO,const bool Infinite,const int TTri,const GaPolyhedron& thePolyh,
                const gp_XYZ& TriNormal,const double TriDp,const double dBegTri,const double dEndTri);
 bool BeginOfClosedPolygon;
 int iLin;
};
#define Polygon3d GaPolygon3d
#define ToolPolygon3d GaPolygon3dTool
#define Polyhedron GaPolyhedron
#define ToolPolyh GaPolyhedronTool
#define Intf_InterferencePolygonPolyhedron GaInterferencePP
#include <Intf_InterferencePolygonPolyhedron.gxx>
#undef Polygon3d
#undef ToolPolygon3d
#undef Polyhedron
#undef ToolPolyh
#undef Intf_InterferencePolygonPolyhedron
// Lines `n (p d)...`, indexed from 1.
NCollection_Array1<gp_Lin> ga_lines(GaReader& r){
 int n=r.integer();NCollection_Array1<gp_Lin> lines(1,std::max(n,1));
 for(int i=1;i<=n;++i){gp_Pnt p=r.pnt();lines(i)=gp_Lin(p,r.dir());}
 return lines;
}
// One operation of the generic on `interference`: 0..5 are Perform with a polygon, a
// line or lines (3..5 with a grid of the polyhedron built here), 6/7 the Interference
// members (7 with the grid), 8..13 the constructors in the same order.
void ga_ipp_operation(GaReader& r,GaInterferencePP& interference,const GaPolyhedron& polyhedron,int kind){
 Bnd_BoundSortBox grid;grid.Initialize(polyhedron.box,polyhedron.boxes);
 int base=kind>=8?kind-8:kind;bool construct=kind>=8;
 if(base==0||base==3||kind==6||kind==7){GaPolygon3d polygon(r);
  if(kind==6){GA_TRY(interference.Interference(polygon,polyhedron));return;}
  if(kind==7){GA_TRY(interference.Interference(polygon,polyhedron,grid));return;}
  if(base==0)GA_TRY(if(construct)interference=GaInterferencePP(polygon,polyhedron);else interference.Perform(polygon,polyhedron))
  else GA_TRY(if(construct)interference=GaInterferencePP(polygon,polyhedron,grid);else interference.Perform(polygon,polyhedron,grid))
  return;}
 if(base==1||base==4){gp_Pnt p=r.pnt();gp_Lin line(p,r.dir());
  if(base==1)GA_TRY(if(construct)interference=GaInterferencePP(line,polyhedron);else interference.Perform(line,polyhedron))
  else GA_TRY(if(construct)interference=GaInterferencePP(line,polyhedron,grid);else interference.Perform(line,polyhedron,grid))
  return;}
 NCollection_Array1<gp_Lin> lines=ga_lines(r);
 if(base==2)GA_TRY(if(construct)interference=GaInterferencePP(lines,polyhedron);else interference.Perform(lines,polyhedron))
 else GA_TRY(if(construct)interference=GaInterferencePP(lines,polyhedron,grid);else interference.Perform(lines,polyhedron,grid))
}
// `polyhedron count (kind operand)...`: the operations on one interference.
void ga_intf_ipp_case(std::istream& input){
 GaReader r{input};GaPolyhedron polyhedron(r);GaInterferencePP interference;int count=r.integer();
 for(int k=0;k<count;++k){ga_ipp_operation(r,interference,polyhedron,r.integer());ga_interference_print(interference);}
 ga_end();
}
