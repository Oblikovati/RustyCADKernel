// AIS_Shape state, presentations, selection and HLR through the recording display driver, the
// twin of crates/tkv3d/examples/v3_probe/ais_shape.rs. A request is
// `v3_ais_shape Shape Link Width Count Op...` (shapes and ops below).
#include <AIS_Shape.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Builder.hxx>
#include <Graphic3d_Camera.hxx>
#include <Select3D_SensitiveEntity.hxx>
#include <StdSelect_BRepOwner.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Cylinder.hxx>

// Expose the protected HLR entry of AIS_Shape.
class V3ProbeShape : public AIS_Shape {
public:
  V3ProbeShape(const TopoDS_Shape& s):AIS_Shape(s){}
  void hlr(const occ::handle<Graphic3d_Camera>& c,const occ::handle<TopLoc_Datum3D>& t,const occ::handle<Prs3d_Presentation>& p){computeHLR(c,t,p);}
};
// Shape: 0 vertex, 1 edge, 2 cylinder patch, 3 compound (face, edge, vertex), 4 empty compound,
// 5 null, 6 infinite plane.
static TopoDS_Shape v3_ais_shape_of(int kind){
  const gp_Ax3 ax(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0));
  BRep_Builder b;TopoDS_Compound c;
  switch(kind){
    case 0:return BRepBuilderAPI_MakeVertex(gp_Pnt(1,2,3)).Vertex();
    case 1:return BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,1,0)).Edge();
    case 2:return BRepBuilderAPI_MakeFace(gp_Cylinder(ax,2.),0.,M_PI,0.,3.).Face();
    case 3:b.MakeCompound(c);b.Add(c,v3_ais_shape_of(2));b.Add(c,v3_ais_shape_of(1));b.Add(c,v3_ais_shape_of(0));return c;
    case 4:b.MakeCompound(c);return c;
    case 6:return BRepBuilderAPI_MakeFace(gp_Pln(ax)).Face();
    default:return TopoDS_Shape();
  }
}
static void v3_ais_shape_rgb(const Quantity_Color& c){ga_real(c.Red());ga_real(c.Green());ga_real(c.Blue());}
static void v3_ais_shape_state(const occ::handle<V3ProbeShape>& o){
  ga_int(o->Signature());ga_int(o->Type());for(int m=-1;m<=3;++m)ga_flag(o->AcceptDisplayMode(m));ga_flag(o->AcceptShapeDecomposition());
  ga_flag(o->HasColor());ga_flag(o->HasWidth());ga_real(o->Width());Quantity_Color c;o->Color(c);v3_ais_shape_rgb(c);
  ga_int(o->Material());ga_real(o->Transparency());ga_flag(o->HasMaterial());
  GA_TRY(const Bnd_Box& b=o->BoundingBox();ga_flag(b.IsVoid());if(!b.IsVoid()){double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);ga_real(x0);ga_real(y0);ga_real(z0);ga_real(x1);ga_real(y1);ga_real(z1);})
  ga_pnt2d(o->TextureRepeatUV());ga_pnt2d(o->TextureOriginUV());ga_pnt2d(o->TextureScaleUV());
  double a,b;ga_flag(o->OwnDeviationCoefficient(a,b));ga_real(a);ga_real(b);ga_flag(o->OwnDeviationAngle(a,b));ga_real(a);ga_real(b);
  ga_int(o->TypeOfHLR());
  const occ::handle<Prs3d_Drawer>& d=o->Attributes();ga_flag(d->HasOwnShadingAspect());ga_flag(d->HasOwnLineAspect());ga_flag(d->HasOwnWireAspect());
  if(!d->LineAspect().IsNull()){ga_real(d->LineAspect()->Aspect()->Width());v3_ais_shape_rgb(d->LineAspect()->Aspect()->Color());}
}
static void v3_ais_shape_select(const occ::handle<V3ProbeShape>& o,int mode){
  occ::handle<SelectMgr_Selection> sel=new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject*>(o.get())->ComputeSelection(sel,mode);
  ga_int(sel->Entities().Size());
  for(const auto& item:sel->Entities()){const occ::handle<Select3D_SensitiveEntity>& e=item->BaseSensitive();
    ga_text(e->DynamicType()->Name());ga_int(e->NbSubElements());
    occ::handle<StdSelect_BRepOwner> owner=occ::down_cast<StdSelect_BRepOwner>(e->OwnerId());
    ga_flag(!owner.IsNull());if(!owner.IsNull()){ga_int(owner->Shape().IsNull()?-1:owner->Shape().ShapeType());ga_int(owner->Priority());}}
}
static void v3_ais_shape_op(const occ::handle<V3ProbeShape>& o,const occ::handle<PrsMgr_PresentationManager>& m,const occ::handle<V3d_Viewer>& viewer,int op,double width){
  switch(op){
    case 0:o->SetColor(Quantity_NOC_RED);break;
    case 1:o->UnsetColor();break;
    case 2:o->SetWidth(width);break;
    case 3:o->UnsetWidth();break;
    case 4:o->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));break;
    case 5:o->UnsetMaterial();break;
    case 6:o->SetTransparency(0.4);break;
    case 7:o->UnsetTransparency();break;
    case 8:case 9:case 10:{const int mode=op-8;if(m->HasPresentation(o,mode))m->Update(o,mode);else m->Presentation(o,mode,true);break;}
    case 11:v3_ais_shape_select(o,0);break;
    case 12:v3_ais_shape_select(o,2);break;
    case 13:v3_ais_shape_select(o,4);break;
    case 14:o->SetOwnDeviationCoefficient(0.01);break;
    case 15:ga_flag(o->SetOwnDeviationCoefficient());break;
    case 16:o->SetOwnDeviationAngle(0.3);break;
    case 17:o->SetAngleAndDeviation(0.25);break;
    case 18:o->SetTypeOfHLR(Prs3d_TOH_Algo);break;
    case 19:o->SetTextureRepeatUV(gp_Pnt2d(2,3));o->SetTextureOriginUV(gp_Pnt2d(0.5,0.25));o->SetTextureScaleUV(gp_Pnt2d(4,5));break;
    case 20:o->SetShape(v3_ais_shape_of(1));break;
    case 21:case 22:{occ::handle<Graphic3d_Camera> cam=new Graphic3d_Camera();cam->SetEyeAndCenter(gp_Pnt(10,8,6),gp_Pnt(0,0,0));cam->SetUp(gp_Dir(0,0,1));
      occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
      gp_Trsf t;if(op==22)t.SetTranslation(gp_Vec(1,2,3));occ::handle<TopLoc_Datum3D> datum=new TopLoc_Datum3D(t);
      o->hlr(cam,op==22?datum:occ::handle<TopLoc_Datum3D>(),prs);ga_int(prs->Groups().Size());break;}
    case 23:ga_flag(o->SetOwnDeviationAngle());break;
    default:for(int k=0;k<=8;++k)ga_int(AIS_Shape::SelectionMode(TopAbs_ShapeEnum(k)));for(int k=-1;k<=9;++k)ga_int(AIS_Shape::SelectionType(k));
  }
}
void v3_ais_shape(std::istream& input){
  GaReader r{input};const int kind=r.integer(),link=r.integer();const double width=r.real();const int count=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<V3ProbeShape> o=new V3ProbeShape(v3_ais_shape_of(kind));
  if(link==0)o->Attributes()->SetupOwnDefaults();
  if(link==1){occ::handle<Prs3d_Drawer> d=new Prs3d_Drawer();d->SetupOwnDefaults();d->LineAspect()->SetWidth(2.5);o->Attributes()->SetLink(d);}
  v3_ais_shape_state(o);
  for(int i=0;i<count;++i){const int op=r.integer();GA_TRY(v3_ais_shape_op(o,manager,viewer,op,width))v3_ais_shape_state(o);}
  v3_display_trace(trace);ga_end();
}
