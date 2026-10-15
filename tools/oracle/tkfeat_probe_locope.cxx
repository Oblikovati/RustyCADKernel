// LocOpe probes of tools/oracle/tkfeat_probe.cxx (the swept shapes, the curve-shape
// intersections, the confused edges, the splitter and the gluer); the Rust twin is
// crates/tkfeat/examples/ft_probe/locope.rs.
#include <Geom_Curve.hxx>
#include <LocOpe_CurveShapeIntersector.hxx>
#include <LocOpe_DPrism.hxx>
#include <LocOpe_FindEdges.hxx>
#include <LocOpe_Gluer.hxx>
#include <LocOpe_LinearForm.hxx>
#include <LocOpe_Prism.hxx>
#include <LocOpe_Revol.hxx>
#include <LocOpe_RevolutionForm.hxx>
#include <LocOpe_Spliter.hxx>
#include <LocOpe_WiresOnShape.hxx>
#include <NCollection_Sequence.hxx>
#include <gp_Ax1.hxx>

// The swept shape of a request and its first and last shapes.
template<class Sweep> void ft_lo_shapes(const Sweep& op){
 ft_shape(op.Shape());
 ga_int(ft_count(op.FirstShape(),TopAbs_FACE));ga_int(ft_count(op.LastShape(),TopAbs_FACE));}
// The curves of a sweep: their number and the ranges of the first and of the barycentre
// curve.
template<class Sweep> void ft_lo_curves(Sweep& op){
 NCollection_Sequence<occ::handle<Geom_Curve>> scur;op.Curves(scur);ga_int(scur.Length());
 if(scur.Length()>0&&!scur(1).IsNull()){ga_real(scur(1)->FirstParameter());ga_real(scur(1)->LastParameter());}
 occ::handle<Geom_Curve> b=op.BarycCurve();
 if(!b.IsNull()){ga_real(b->FirstParameter());ga_real(b->LastParameter());}}

// `ft_lo_sweep kind a b c`: the sweep of the square [0, 10]^2 (the rectangle x in
// [2, 4], z in [0, 2] of the plane y = 0 for the revolutions): 0 the prism along
// (a, b, c), 1 the prism along (0, 0, a) of the base moved by (0, 0, b), 2 the drafted
// prism of height a and draft b, 3 the drafted prism of heights a and b and draft c, 4 the
// revolution of a around Z, 5 the revolution of a of the base turned by b, 6 the linear
// form along (0, 0, a), 7 the revolution form of a.
void ft_lo_sweep_case(std::istream& in){
 GaReader r{in};int kind=r.integer();double a=r.real(),b=r.real(),c=r.real();
 GA_TRY(
  TopoDS_Face sq=ft_rect(0.,0.,10.,10.,0.);
  TopoDS_Face rc=ft_polygon_face({gp_Pnt(2.,0.,0.),gp_Pnt(4.,0.,0.),gp_Pnt(4.,0.,2.),gp_Pnt(2.,0.,2.)});
  gp_Ax1 z(gp_Pnt(0.,0.,0.),gp_Dir(0.,0.,1.));
  if(kind==0){LocOpe_Prism p(sq,gp_Vec(a,b,c));ft_lo_shapes(p);ft_lo_curves(p);}
  else if(kind==1){LocOpe_Prism p(sq,gp_Vec(0.,0.,a),gp_Vec(0.,0.,b));ft_lo_shapes(p);ft_lo_curves(p);}
  else if(kind==2){LocOpe_DPrism p(sq,a,b);ft_lo_shapes(p);ft_lo_curves(p);}
  else if(kind==3){LocOpe_DPrism p(sq,a,b,c);ft_lo_shapes(p);ft_lo_curves(p);}
  else if(kind==4){LocOpe_Revol p;p.Perform(rc,z,a);ft_lo_shapes(p);ft_lo_curves(p);}
  else if(kind==5){LocOpe_Revol p;p.Perform(rc,z,a,b);ft_lo_shapes(p);ft_lo_curves(p);}
  else if(kind==6){LocOpe_LinearForm p(sq,gp_Vec(0.,0.,a),gp_Pnt(0.,0.,0.),gp_Pnt(10.,0.,0.));ft_lo_shapes(p);}
  else{LocOpe_RevolutionForm p;p.Perform(rc,z,a);ft_lo_shapes(p);});
 ga_end();}

// `ft_lo_csinter x y z dx dy dz`: the line through (x, y, z) along (dx, dy, dz) and the
// box [0, 10]^3: its points (parameter and orientation), the first franc point after 0
// and the last one before the end (their orientation and indices only when found).
void ft_lo_csinter_case(std::istream& in){
 GaReader r{in};gp_Pnt p=r.pnt();gp_Dir d=r.dir();
 GA_TRY(
  LocOpe_CurveShapeIntersector asi(gp_Ax1(p,d),ft_basis(0));
  ga_flag(asi.IsDone());int n=asi.NbPoints();ga_int(n);
  for(int i=1;i<=n;i++){ga_real(asi.Point(i).Parameter());ga_int(asi.Point(i).Orientation());}
  TopAbs_Orientation o;int i1,i2;
  bool ok=asi.LocalizeAfter(0.,o,i1,i2);ga_flag(ok);if(ok){ga_int(o);ga_int(i1);ga_int(i2);}
  ok=asi.LocalizeBefore(n+1,o,i1,i2);ga_flag(ok);if(ok){ga_int(o);ga_int(i1);ga_int(i2);});
 ga_end();}

// `ft_lo_find_edges ox dz`: the confused edges of the top face of the box [0, 10]^3 and
// the bottom face of the box [ox, ox + 10] x [0, 10] x [10, 10 + dz].
void ft_lo_find_edges_case(std::istream& in){
 GaReader r{in};double ox=r.real(),dz=r.real();
 GA_TRY(
  TopoDS_Shape up=ft_box(ox,0.,10.,10.,10.,dz);
  LocOpe_FindEdges fe(ft_face_at(ft_basis(0),10.),ft_face_at(up,10.));
  int n=0;for(fe.InitIterator();fe.More();fe.Next())n++;ga_int(n););
 ga_end();}

// `ft_lo_spliter x`: the top face of the box [0, 10]^3 split by the edge x = const across
// it: the result, the pieces of the top face and the faces on the left.
void ft_lo_spliter_case(std::istream& in){
 GaReader r{in};double x=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(0);TopoDS_Face top=ft_face_at(b,10.);
  occ::handle<LocOpe_WiresOnShape> pw=new LocOpe_WiresOnShape(b);
  pw->Bind(BRepBuilderAPI_MakeEdge(gp_Pnt(x,0.,10.),gp_Pnt(x,10.,10.)).Edge(),top);
  LocOpe_Spliter sp(b);sp.Perform(pw);ga_flag(sp.IsDone());
  if(sp.IsDone()){ft_shape(sp.ResultingShape());ga_int(sp.DescendantShapes(top).Extent());
   ga_int(sp.DirectLeft().Extent());ga_int(sp.Left().Extent());});
 ga_end();}

// `ft_lo_gluer z h`: the box [2, 4]^2 x [z, z + h] glued on the box [0, 10]^3 through
// its face at the height 10: the operation and the result.
void ft_lo_gluer_case(std::istream& in){
 GaReader r{in};double z=r.real(),h=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(0);TopoDS_Shape s=ft_box(2.,2.,z,2.,2.,h);
  LocOpe_Gluer g(b,s);g.Bind(ft_face_at(s,10.),ft_face_at(b,10.));
  ga_int(g.OpeType());g.Perform();ga_flag(g.IsDone());
  if(g.IsDone()){ft_shape(g.ResultingShape());ga_int(g.Edges().Extent());});
 ga_end();}
