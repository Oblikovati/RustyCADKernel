// BRepFeat form probes of tools/oracle/tkfeat_probe.cxx (the cylindrical holes, the
// linear and revolution forms, the gluer and the shape splitter); the Rust twin is
// crates/tkfeat/examples/ft_probe/forms.rs.
#include <BRepFeat_Gluer.hxx>
#include <BRepFeat_MakeCylindricalHole.hxx>
#include <BRepFeat_MakeLinearForm.hxx>
#include <BRepFeat_MakeRevolutionForm.hxx>
#include <BRepFeat_SplitShape.hxx>
#include <Geom_Plane.hxx>

// `ft_hole basis x method r a b cont`: the hole of radius r along the axis from
// (x, 5, 12) downwards in the basis (0 box, 2 plates): 0 through all, 1 through the next
// exit, 2 up to the end, 3 between the parameters a and b, 4 blind of length a; the
// status after the placement and after the build, then the shape when built.
void ft_hole_case(std::istream& in){
 GaReader r{in};int kind=r.integer();double x=r.real();int method=r.integer();double rad=r.real(),a=r.real(),bb=r.real();bool cont=r.flag();
 GA_TRY(
  BRepFeat_MakeCylindricalHole h;h.Init(ft_basis(kind),gp_Ax1(gp_Pnt(x,5.,12.),gp_Dir(0.,0.,-1.)));
  if(method==0)h.Perform(rad);
  else if(method==1)h.PerformThruNext(rad,cont);
  else if(method==2)h.PerformUntilEnd(rad,cont);
  else if(method==3)h.Perform(rad,a,bb,cont);
  else h.PerformBlind(rad,a,cont);
  ga_int(h.Status());h.Build();ga_int(h.Status());
  if(h.Status()==BRepFeat_NoError)ft_shape(h.Shape()););
 ga_end();}

// A form after its Perform: done, its status error, then its shape when done.
template<class Form> void ft_form(Form& f){
 ga_flag(f.IsDone());ga_int(f.CurrentStatusError());
 if(f.IsDone()){ft_shape(f.Shape());ga_int(f.FirstShape().Extent());ga_int(f.LastShape().Extent());}}

// `ft_lform mode slide back h`: the rib (mode 1) or slot (0) of the open wire
// (2, 5, 10) (2, 5, 10 + h) (8, 5, 10 + h) (8, 5, 10) in the plane y = 5 on the box,
// along +Y and back along -Y (`back`), sliding on request.
void ft_lform_case(std::istream& in){
 GaReader r{in};int mode=r.integer();bool slide=r.flag(),back=r.flag();double h=r.real();
 GA_TRY(
  TopoDS_Wire w=ft_polygon_wire({gp_Pnt(2.,5.,10.),gp_Pnt(2.,5.,10.+h),gp_Pnt(8.,5.,10.+h),gp_Pnt(8.,5.,10.)});
  occ::handle<Geom_Plane> pl=new Geom_Plane(gp_Pnt(0.,5.,0.),gp_Dir(0.,1.,0.));
  BRepFeat_MakeLinearForm f(ft_basis(0),w,pl,gp_Vec(0.,1.,0.),back?gp_Vec(0.,-1.,0.):gp_Vec(0.,0.,0.),mode,slide);
  if(f.IsDone())f.Perform();
  ft_form(f););
 ga_end();}

// `ft_rform basis mode slide h1 h2`: the rib (mode 1) or groove (0) of revolution of the
// open wire (2, 0, 10) (2, 0, 10 +- 2) (4, 0, 10 +- 2) (4, 0, 10) in the plane y = 0
// around the Z axis on the cylinder of radius 5 (basis 0) or the box [-5, 5]^2 x [0, 10]
// (1), between the heights h1 and h2.
void ft_rform_case(std::istream& in){
 GaReader r{in};int kind=r.integer(),mode=r.integer();bool slide=r.flag();double h1=r.real(),h2=r.real();
 GA_TRY(
  TopoDS_Shape b=kind?ft_box(-5.,-5.,0.,10.,10.,10.):BRepPrimAPI_MakeCylinder(5.,10.).Shape();
  double z1=mode?12.:8.;
  TopoDS_Wire w=ft_polygon_wire({gp_Pnt(2.,0.,10.),gp_Pnt(2.,0.,z1),gp_Pnt(4.,0.,z1),gp_Pnt(4.,0.,10.)});
  occ::handle<Geom_Plane> pl=new Geom_Plane(gp_Pnt(0.,0.,0.),gp_Dir(0.,1.,0.));
  BRepFeat_MakeRevolutionForm f(b,w,pl,gp_Ax1(gp_Pnt(0.,0.,0.),gp_Dir(0.,0.,1.)),h1,h2,mode,slide);
  if(f.IsDone())f.Perform();
  ft_form(f););
 ga_end();}

// `ft_gluer z h`: the box [2, 4]^2 x [z, z + h] glued on the box [0, 10]^3 through its
// face at the height 10 (fused above, cut below): the operation, the result, and the
// history of the top face of the box.
void ft_gluer_case(std::istream& in){
 GaReader r{in};double z=r.real(),h=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(0);TopoDS_Shape s=ft_box(2.,2.,z,2.,2.,h);TopoDS_Face top=ft_face_at(b,10.);
  BRepFeat_Gluer g(s,b);g.Bind(ft_face_at(s,10.),top);ga_int(g.OpeType());
  g.Build();ft_result(g);
  if(g.IsDone()){ga_int(g.Modified(top).Extent());ga_flag(g.IsDeleted(top));});
 ga_end();}

// `ft_split x n`: the top face of the box [0, 10]^3 split by n edges x + k across it:
// the result, the pieces of the top face, and the faces on the left and on the right.
void ft_split_case(std::istream& in){
 GaReader r{in};double x=r.real();int n=r.integer();
 GA_TRY(
  TopoDS_Shape b=ft_basis(0);TopoDS_Face top=ft_face_at(b,10.);
  BRepFeat_SplitShape sp(b);
  for(int k=0;k<n;k++)sp.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(x+k,0.,10.),gp_Pnt(x+k,10.,10.)).Edge(),top);
  sp.Build();ft_result(sp);
  if(sp.IsDone()){ga_int(sp.Modified(top).Extent());ga_int(sp.DirectLeft().Extent());
   ga_int(sp.Left().Extent());ga_int(sp.Right().Extent());});
 ga_end();}
