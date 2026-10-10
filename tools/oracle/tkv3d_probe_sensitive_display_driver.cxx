// A named recording driver captures real native buffer callbacks without rendering.
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_Text.hxx>
#include <Graphic3d_IndexBuffer.hxx>
struct V3DisplayTrace { std::vector<double> events; std::vector<occ::handle<Graphic3d_Text>> texts; int fail=-1,groups=0; occ::handle<SelectMgr_Selection> appendSelection; occ::handle<SelectMgr_SelectableObject> appendObject; };
class V3DisplayGroup: public SvGroup {
  V3DisplayTrace& trace;
public:
  V3DisplayGroup(const occ::handle<Graphic3d_Structure>& prs,V3DisplayTrace& log):SvGroup(prs),trace(log){}
  void AddText(const occ::handle<Graphic3d_Text>& text, const bool evaluate=true) override {trace.texts.push_back(text); SvGroup::AddText(text,evaluate);}
  void aspect(const occ::handle<Graphic3d_Aspects>& value){
    auto c=value->InteriorColor();trace.events.insert(trace.events.end(),{3.,c.Red(),c.Green(),c.Blue(),double(value->LineType()),double(value->LineWidth()),double(value->MarkerType()),double(value->MarkerScale())});
  }
  void SetPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& value) override {aspect(value);SvGroup::SetPrimitivesAspect(value);}
  void SetGroupPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& value) override {aspect(value);SvGroup::SetGroupPrimitivesAspect(value);}
  void appendCallbacks(){
    auto selection=trace.appendSelection;trace.appendSelection.Nullify();auto object=trace.appendObject;trace.appendObject.Nullify();
    if(!selection.IsNull()){trace.events.push_back(6.);selection->Add(new Select3D_SensitivePoint(new SelectMgr_EntityOwner(0),gp_Pnt(9,8,7)));}
    if(!object.IsNull()){trace.events.push_back(7.);occ::handle<SelectMgr_Selection> added=new SelectMgr_Selection(1);
      added->Add(new Select3D_SensitivePoint(new SelectMgr_EntityOwner(0),gp_Pnt(6,5,4)));added->SetSelectionState(SelectMgr_SOS_Activated);object->AddSelection(added,1);}
  }
  void SetFlippingOptions(bool enabled,const gp_Ax2& plane) override {auto p=plane.Location();trace.events.insert(trace.events.end(),{5.,double(enabled),p.X(),p.Y(),p.Z()});appendCallbacks();}
  void AddPrimitiveArray(Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& attribs,const occ::handle<Graphic3d_BoundBuffer>& bounds,bool evaluate) override {
    trace.events.insert(trace.events.end(),{4.,double(kind),double(attribs->NbElements)});
    int attribute;size_t stride;auto start=attribs->AttributeData(Graphic3d_TOA_POS,attribute,stride);
    for(int i=0;i<attribs->NbElements;++i){float vertex[3];std::memcpy(vertex,start+i*stride,sizeof(vertex));for(float v:vertex)trace.events.push_back(double(v));}
    trace.events.push_back(indices.IsNull()?0:indices->NbElements);
    if(!indices.IsNull())for(int i=0;i<indices->NbElements;++i)trace.events.push_back(indices->Index(i));
    Graphic3d_Group::AddPrimitiveArray(kind,indices,attribs,bounds,evaluate);
  }
};
class V3DisplayStructure:public SvCStructure {
  V3DisplayTrace& trace;
public:
  V3DisplayStructure(const occ::handle<Graphic3d_StructureManager>& manager,V3DisplayTrace& log):SvCStructure(manager),trace(log){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override {
    trace.events.push_back(2.);++trace.groups;if(trace.groups==trace.fail)throw Standard_ProgramError("injected group failure");
    occ::handle<Graphic3d_Group> group=new V3DisplayGroup(prs,trace);myGroups.Append(group);return group;
  }
};
class V3DisplayDriver:public SvDriver {
  V3DisplayTrace& trace;
public:
  V3DisplayDriver(V3DisplayTrace& log):trace(log){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override {
    trace.events.push_back(1.);return new V3DisplayStructure(manager,trace);
  }
};
