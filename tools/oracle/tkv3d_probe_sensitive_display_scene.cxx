// Native entities, including a SensitiveCurve reached through its SensitivePoly base.
#include <SelectMgr.hxx>
#include <Select3D_SensitiveBox.hxx>
#include <Select3D_SensitiveCircle.hxx>
#include <Select3D_SensitiveCylinder.hxx>
#include <Select3D_SensitiveFace.hxx>
#include <Select3D_SensitivePoly.hxx>
#include <Select3D_SensitiveCurve.hxx>
#include <Select3D_SensitiveWire.hxx>
#include <Select3D_SensitiveTriangle.hxx>
#include <Select3D_SensitiveTriangulation.hxx>
#include <Select3D_SensitiveSphere.hxx>
static gp_Trsf v3_display_location(int kind){
  gp_Trsf location;if(kind==2)location.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(1.,2.,3.)),.7);
  if(kind==3)location.SetScale(gp_Pnt(1.,2.,3.),-1.25);
  if(kind!=0)location.SetTranslationPart(gp_Vec(4.,-3.,2.));return location;
}
static NCollection_Array1<gp_Pnt> v3_display_points(){
  NCollection_Array1<gp_Pnt> points(1,4);points(1)=gp_Pnt(0,0,0);points(2)=gp_Pnt(2,0,0);points(3)=gp_Pnt(2,3,1);points(4)=gp_Pnt(0,3,0);return points;
}
static occ::handle<Select3D_SensitiveEntity> v3_display_entity(const occ::handle<SelectMgr_EntityOwner>& owner,int kind);
static occ::handle<Select3D_SensitiveEntity> v3_display_wire(const occ::handle<SelectMgr_EntityOwner>& owner){
  occ::handle<Select3D_SensitiveWire> wire=new Select3D_SensitiveWire(owner);for(int kind:{1,7,0,6})wire->Add(v3_display_entity(owner,kind));return wire;
}
static occ::handle<Select3D_SensitiveEntity> v3_display_triangulation(const occ::handle<SelectMgr_EntityOwner>& owner){
  occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(4,2,false,false);auto points=v3_display_points();
  for(int i=1;i<=4;++i)mesh->SetNode(i,points(i));mesh->SetTriangle(1,Poly_Triangle(1,2,3));mesh->SetTriangle(2,Poly_Triangle(1,3,4));
  return new Select3D_SensitiveTriangulation(owner,mesh,TopLoc_Location(v3_display_location(1)),true);
}
static occ::handle<Select3D_SensitiveEntity> v3_display_entity(const occ::handle<SelectMgr_EntityOwner>& owner,int kind){
  auto p=v3_display_points();switch(kind){
    case 0:return new Select3D_SensitivePoint(owner,p(3));
    case 1:return new Select3D_SensitiveSegment(owner,p(1),p(3));
    case 2:return new Select3D_SensitiveBox(owner,-1.,-2.,-3.,4.,5.,6.);
    case 3:return new Select3D_SensitiveCircle(owner,gp_Circ(gp_Ax2(gp_Pnt(1,2,3),gp_Dir(0,1,1)),2.),false);
    case 4:case 14:return new Select3D_SensitiveCylinder(owner,kind==14?0.:2.,kind==14?0.:.5,4.,v3_display_location(2),false);
    case 5:return new Select3D_SensitiveFace(owner,p,Select3D_TOS_BOUNDARY);
    case 6:return new Select3D_SensitivePoly(owner,p,true);
    case 7:return new Select3D_SensitiveCurve(owner,p);
    case 8:return v3_display_wire(owner);
    case 9:return new Select3D_SensitiveTriangle(owner,p(1),p(2),p(3),Select3D_TOS_INTERIOR);
    case 10:return v3_display_triangulation(owner);
    default:return new Select3D_SensitiveSphere(owner,p(3),2.);
  }
}
static occ::handle<SelectMgr_Selection> v3_display_selection(int kind,int flags){
  occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(0);occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  if(kind==13)return selection;int count=kind==12?12:1;
  for(int i=0;i<count;++i){auto entity=v3_display_entity(owner,kind==12?i:kind);
    if(flags&1)entity->SetFlippingOptions(true,gp_Ax2(gp_Pnt(1,2,3),gp_Dir(0,0,1)));selection->Add(entity);}
  return selection;
}
