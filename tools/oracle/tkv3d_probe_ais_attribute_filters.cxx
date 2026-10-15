// AIS_TypeFilter, AIS_SignatureFilter, AIS_AttributeFilter, AIS_ExclusionFilter,
// AIS_BadEdgeFilter, AIS_C0RegularityFilter and AIS_GraphicTool, the twin of
// crates/tkv3d/examples/v3_probe/ais_attribute_filters.rs. A request is
// `v3_ais_attr_filter Op Object Color Width Values...`.
#include <AIS_AttributeFilter.hxx>
#include <AIS_BadEdgeFilter.hxx>
#include <AIS_C0RegularityFilter.hxx>
#include <AIS_Circle.hxx>
#include <AIS_ColoredShape.hxx>
#include <AIS_ExclusionFilter.hxx>
#include <AIS_GraphicTool.hxx>
#include <AIS_Point.hxx>
#include <AIS_SignatureFilter.hxx>
#include <AIS_TypeFilter.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Geom_Circle.hxx>
#include <StdSelect_BRepOwner.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>

// Object: 0 point (datum 1), 1 circle (datum 6), 2 colored shape (shape 0), 3 no selectable.
static occ::handle<AIS_InteractiveObject> v3_attr_object(int kind){
  switch(kind){
    case 0:return new AIS_Point(new Geom_CartesianPoint(1.,2.,3.));
    case 1:return new AIS_Circle(new Geom_Circle(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0)),2.));
    case 2:return new AIS_ColoredShape(BRepBuilderAPI_MakeVertex(gp_Pnt(0,0,0)).Vertex());
    default:return occ::handle<AIS_InteractiveObject>();
  }
}
static TopoDS_Shape v3_attr_shape(int kind){
  const gp_Ax3 ax(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0));
  if(kind==0)return BRepBuilderAPI_MakeFace(gp_Cylinder(ax,2.),0.,2.*M_PI,0.,1.).Face();
  if(kind==1)return BRepBuilderAPI_MakeFace(gp_Sphere(ax,1.)).Face();
  return BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(1,0,0)).Edge();
}
static void v3_attr_list(const NCollection_List<int>& list){ga_int(list.Extent());for(int v:list)ga_int(v);}
void v3_ais_attr_filter(std::istream& input){
  GaReader r{input};const int op=r.integer(),objectKind=r.integer(),color=r.integer();const double width=r.real();
  occ::handle<AIS_InteractiveObject> object=v3_attr_object(objectKind);
  if(!object.IsNull()){if(color>=0)object->SetColor(Quantity_Color(Quantity_NameOfColor(color)));if(width>0.)object->SetWidth(width);}
  occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(object,0);
  switch(op){
    case 0:{const int kind=r.integer();ga_flag((new AIS_TypeFilter(AIS_KindOfInteractive(kind)))->IsOk(owner));break;}
    case 1:{const int kind=r.integer(),sig=r.integer();ga_flag((new AIS_SignatureFilter(AIS_KindOfInteractive(kind),sig))->IsOk(owner));break;}
    case 2:{const int mode=r.integer(),col=r.integer();const double wid=r.real();const int edits=r.integer();
      occ::handle<AIS_AttributeFilter> f=mode==0?new AIS_AttributeFilter():mode==1?new AIS_AttributeFilter(Quantity_NameOfColor(col)):new AIS_AttributeFilter(wid);
      for(int i=0;i<edits;++i){switch(r.integer()){case 0:f->SetColor(Quantity_NameOfColor(r.integer()));break;case 1:f->SetWidth(r.real());break;case 2:f->UnsetColor();break;default:f->UnsetWidth();}}
      ga_flag(f->HasColor());ga_flag(f->HasWidth());ga_flag(f->IsOk(owner));break;}
    case 3:{const int ctor=r.integer(),kind=r.integer(),sig=r.integer();const bool flag=r.flag();
      occ::handle<AIS_ExclusionFilter> f=ctor==0?new AIS_ExclusionFilter(flag):ctor==1?new AIS_ExclusionFilter(AIS_KindOfInteractive(kind),flag)
        :new AIS_ExclusionFilter(AIS_KindOfInteractive(kind),sig,flag);
      const int edits=r.integer();
      for(int i=0;i<edits;++i){const int code=r.integer(),k=r.integer(),s=r.integer();const AIS_KindOfInteractive kk=AIS_KindOfInteractive(k);
        switch(code){case 0:ga_flag(f->Add(kk));break;case 1:ga_flag(f->Add(kk,s));break;case 2:ga_flag(f->Remove(kk));break;case 3:ga_flag(f->Remove(kk,s));break;
          case 4:f->Clear();break;default:f->SetExclusionFlag(s!=0);}}
      ga_flag(f->IsExclusionFlagOn());ga_flag(f->IsOk(owner));
      for(int k=0;k<7;++k){ga_flag(f->IsStored(AIS_KindOfInteractive(k)));NCollection_List<int> sigs;f->ListOfSignature(AIS_KindOfInteractive(k),sigs);v3_attr_list(sigs);}
      NCollection_List<int> types;f->ListOfStoredTypes(types);v3_attr_list(types);break;}
    case 4:{occ::handle<Prs3d_Drawer> dr=new Prs3d_Drawer();dr->SetupOwnDefaults();const int attribute=r.integer();
      GA_TRY(ga_int(AIS_GraphicTool::GetLineColor(dr,AIS_TypeOfAttribute(attribute))))
      GA_TRY(Quantity_Color c;AIS_GraphicTool::GetLineColor(dr,AIS_TypeOfAttribute(attribute),c);ga_real(c.Red());ga_real(c.Green());ga_real(c.Blue()))
      GA_TRY(ga_real(AIS_GraphicTool::GetLineWidth(dr,AIS_TypeOfAttribute(attribute))))
      GA_TRY(ga_int(AIS_GraphicTool::GetLineType(dr,AIS_TypeOfAttribute(attribute))))
      GA_TRY(Quantity_NameOfColor n;double w;Aspect_TypeOfLine t;AIS_GraphicTool::GetLineAtt(dr,AIS_TypeOfAttribute(attribute),n,w,t);ga_int(n);ga_real(w);ga_int(t))
      GA_TRY(ga_int(AIS_GraphicTool::GetInteriorColor(dr)))
      GA_TRY(ga_int(AIS_GraphicTool::GetMaterial(dr).Name()))break;}
    case 5:{const TopoDS_Shape s=v3_attr_shape(r.integer());occ::handle<AIS_BadEdgeFilter> f=new AIS_BadEdgeFilter();
      std::vector<TopoDS_Shape> edges;for(TopExp_Explorer e(s,TopAbs_EDGE);e.More();e.Next())edges.push_back(e.Current());
      const int edits=r.integer();
      for(int i=0;i<edits;++i){const int code=r.integer(),a=r.integer(),b=r.integer();
        if(code==0)f->SetContour(a);else if(code==1)f->AddEdge(TopoDS::Edge(edges[a%edges.size()]),b);else f->RemoveEdges(a);}
      ga_flag(f->ActsOn(TopAbs_EDGE));ga_flag(f->ActsOn(TopAbs_FACE));ga_flag(f->IsOk(owner));
      for(const TopoDS_Shape& e:edges)ga_flag(f->IsOk(new StdSelect_BRepOwner(e,0,false)));break;}
    case 6:{const TopoDS_Shape s=v3_attr_shape(r.integer());occ::handle<AIS_C0RegularityFilter> f;GA_TRY(f=new AIS_C0RegularityFilter(s))
      if(f.IsNull())break;
      ga_flag(f->ActsOn(TopAbs_EDGE));ga_flag(f->ActsOn(TopAbs_VERTEX));ga_flag(f->IsOk(owner));
      for(TopExp_Explorer e(s,TopAbs_EDGE);e.More();e.Next())ga_flag(f->IsOk(new StdSelect_BRepOwner(e.Current(),0,false)));
      for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next())ga_flag(f->IsOk(new StdSelect_BRepOwner(e.Current(),0,false)));break;}
  }
  ga_end();
}
