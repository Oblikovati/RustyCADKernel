// StdSelect_EdgeFilter, StdSelect_FaceFilter and StdSelect_ShapeTypeFilter on the BRep owners
// of single edges, faces and vertices, a plain owner and a null shape (kinds below).
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <Geom_Line.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <StdSelect_BRepOwner.hxx>
#include <StdSelect_EdgeFilter.hxx>
#include <StdSelect_FaceFilter.hxx>
#include <StdSelect_ShapeTypeFilter.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Elips.hxx>
#include <gp_Pln.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>

// Kind: 0 line, 1 circle, 2 ellipse edges; 3 plane, 4 cylinder, 5 cone, 6 sphere, 7 torus,
// 8 revolution faces; 9 vertex; 10 null shape; 11 plain owner.
TopoDS_Shape v3_stdsel_shape(int kind){
  const gp_Ax2 ax(gp_Pnt(1.,2.,3.),gp_Dir(0.,0.,1.),gp_Dir(1.,0.,0.));
  switch(kind){
    case 0:return BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(0.,0.,0.),gp_Dir(1.,0.,0.)),0.,2.).Edge();
    case 1:return BRepBuilderAPI_MakeEdge(gp_Circ(ax,2.)).Edge();
    case 2:return BRepBuilderAPI_MakeEdge(gp_Elips(ax,3.,1.)).Edge();
    case 3:return BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(1.,2.,3.),gp_Dir(0.,0.,1.)),0.,1.,0.,1.).Face();
    case 4:return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(ax),2.),0.,1.,0.,1.).Face();
    case 5:return BRepBuilderAPI_MakeFace(gp_Cone(gp_Ax3(ax),0.5,2.),0.,1.,0.,1.).Face();
    case 6:return BRepBuilderAPI_MakeFace(gp_Sphere(gp_Ax3(ax),2.),0.,1.,0.,1.).Face();
    case 7:return BRepBuilderAPI_MakeFace(gp_Torus(gp_Ax3(ax),3.,1.),0.,1.,0.,1.).Face();
    case 8:{occ::handle<Geom_Curve> line=new Geom_Line(gp_Pnt(2.,0.,0.),gp_Dir(1.,0.,1.));
      occ::handle<Geom_Surface> surface=new Geom_SurfaceOfRevolution(line,gp_Ax1(gp_Pnt(0.,0.,0.),gp_Dir(0.,0.,1.)));
      return BRepBuilderAPI_MakeFace(surface,0.,1.,0.,1.,1.e-7).Face();}
    case 9:return BRepBuilderAPI_MakeVertex(gp_Pnt(1.,1.,1.)).Vertex();
    default:return TopoDS_Shape();
  }
}

// `v3_stdsel_filter Family Type Kind`: Family 0 edge, 1 face, 2 shape type filter; prints IsOk
// and ActsOn for the nine shape modes.
void v3_stdsel_filter(std::istream& input){
  GaReader r{input};const int family=r.integer(),type=r.integer(),kind=r.integer();
  occ::handle<SelectMgr_EntityOwner> owner=kind==11?new SelectMgr_EntityOwner(0):
    occ::handle<SelectMgr_EntityOwner>(new StdSelect_BRepOwner(v3_stdsel_shape(kind),5,false));
  occ::handle<SelectMgr_Filter> filter;
  if(family==0)filter=new StdSelect_EdgeFilter(StdSelect_TypeOfEdge(type));
  else if(family==1)filter=new StdSelect_FaceFilter(StdSelect_TypeOfFace(type));
  else filter=new StdSelect_ShapeTypeFilter(TopAbs_ShapeEnum(type));
  GA_TRY(ga_flag(filter->IsOk(owner)))
  for(int mode=0;mode<9;++mode)ga_flag(filter->ActsOn(TopAbs_ShapeEnum(mode)));
  if(family==0){occ::handle<StdSelect_EdgeFilter> edge=occ::down_cast<StdSelect_EdgeFilter>(filter);
    ga_int(edge->Type());edge->SetType(StdSelect_TypeOfEdge((type+1)%3));ga_int(edge->Type());}
  if(family==1){occ::handle<StdSelect_FaceFilter> face=occ::down_cast<StdSelect_FaceFilter>(filter);
    ga_int(face->Type());face->SetType(StdSelect_TypeOfFace((type+1)%7));ga_int(face->Type());}
  if(family==2)ga_int(occ::down_cast<StdSelect_ShapeTypeFilter>(filter)->Type());
  ga_end();
}
