// Stateful subset fixtures; C++ bodies remain the pinned OCCT implementation.
#include <SelectMgr_SelectableObjectSet.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <PrsMgr_Presentation.hxx>
class V3BoxedSelectable : public SelectMgr_SelectableObject {
public:
  Bnd_Box bounds;int calls=0;
  V3BoxedSelectable(double offset,bool empty){setBounds(offset,empty);}
  void setBounds(double offset,bool empty){bounds.SetVoid();if(!empty)bounds.Update(offset,-2.,-3.,offset+1.,4.,5.);}
  void BoundingBox(Bnd_Box& result) override {++calls;result=bounds;}
  void Compute(const occ::handle<PrsMgr_PresentationManager>&,const occ::handle<Prs3d_Presentation>&,const int) override {}
  void ComputeSelection(const occ::handle<SelectMgr_Selection>&,const int) override {}
};
class V3FailingBoxPersistence : public Graphic3d_TransformPers {
public:
  V3FailingBoxPersistence():Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers){}
  NCollection_Mat4<double> ComputeApply(occ::handle<Graphic3d_Camera>&,const int,const int,const gp_Pnt*) const override {
    throw Standard_ProgramError("injected persistence failure");
  }
};
occ::handle<Graphic3d_TransformPers> v3_object_persistence(int code){
  if(code==10)return new V3FailingBoxPersistence();
  const int modes[]={0,2,64,258,320,8,10,32,128,514};const int mode=modes[code%10];
  return mode==0?occ::handle<Graphic3d_TransformPers>():new Graphic3d_TransformPers(Graphic3d_TransModeFlags(mode));
}
void v3_object_group_flip(const occ::handle<Graphic3d_Group>& group,bool enabled){
  // SvGroup omits render brackets. Install OpenGl_Group's selection metadata on this mutable fixture.
  if(!enabled)return;
  const_cast<occ::handle<Graphic3d_Flipper>&>(group->Flipper())=new Graphic3d_Flipper(gp_Ax2());
  group->Structure()->CStructure()->SetGroupFlipping(true);
}
void v3_object_groups(const occ::handle<PrsMgr_PresentationManager>& manager,const occ::handle<V3BoxedSelectable>& object,int flags){
  auto prs=manager->Presentation(object,0,true);auto group=prs->NewGroup();group->SetMinMaxValues(-2.,-3.,-4.,5.,6.,7.);
  v3_object_group_flip(group,(flags&1)!=0);group->SetTransformPersistence(v3_object_persistence((flags&2)!=0?1:0));
  auto ignored=prs->NewGroup();ignored->SetMinMaxValues(-100.,-100.,-100.,100.,100.,100.);
  auto empty=prs->NewGroup();empty->SetTransformPersistence(v3_object_persistence(1));
}
std::string v3_object_dump_pointers(std::string text){
  std::string prefix="\"SelectableObject\": \"";size_t offset=0;
  while(true){auto start=text.find(prefix,offset);if(start==std::string::npos)break;start+=prefix.size();
    auto end=text.find('"',start);text.replace(start,end-start,"present");offset=start+7;}return text;
}
struct V3ObjectSetFixture {
  SelectMgr_SelectableObjectSet set;
  std::vector<occ::handle<V3BoxedSelectable>> objects;
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera();
  occ::handle<PrsMgr_PresentationManager> manager;
  std::vector<occ::handle<BVH_Tree<double,3>>> retained;
  SelectMgr_SelectableObjectSet::Iterator cursor;
  V3ObjectSetFixture(int count,int seed,int projection){
    manager=new PrsMgr_PresentationManager(new Graphic3d_StructureManager(new SvDriver()));
    camera->SetProjectionType(projection==0?Graphic3d_Camera::Projection_Orthographic:Graphic3d_Camera::Projection_Perspective);camera->SetScale(12.);
    for(int i=0;i<5;++i)retained.push_back(set.BVH(SelectMgr_SelectableObjectSet::BVHSubset(i)));
    for(int i=0;i<count;++i){occ::handle<V3BoxedSelectable> object=new V3BoxedSelectable(i*3.-5.,(i+seed)%7==0);
      object->SetTransformPersistence(v3_object_persistence((i+seed)%5));objects.push_back(object);}
  }
  int objectId(const occ::handle<SelectMgr_SelectableObject>& object){
    for(size_t i=0;i<objects.size();++i)if(objects[i]==object)return int(i);throw std::runtime_error("unexpected selectable identity");
  }
  void transform(int a,int b){gp_Trsf trsf;trsf.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,0,1)),b/8.);trsf.SetTranslationPart(gp_Vec(b*0.5,1.,-2.));objects[a]->SetLocalTransformation(trsf);}
  void mutate(int op,int a,int b){switch(op){
    case 0:ga_flag(set.Append(objects[a]));break;case 1:ga_flag(set.Remove(objects[a]));break;
    case 2:objects[a]->SetTransformPersistence(v3_object_persistence(b));break;case 3:set.ChangeSubset(objects[a]);break;
    case 4:set.MarkDirty();break;case 5:set.UpdateBVH(a==0?occ::handle<Graphic3d_Camera>():camera,NCollection_Vec2<int>(800+b*17,600+b*11));break;
    case 6:camera->SetEye(gp_Pnt(a,b,10.));break;case 7:camera->SetScale(a+1.);break;
    case 8:camera->SetProjectionType(a%2==0?Graphic3d_Camera::Projection_Orthographic:Graphic3d_Camera::Projection_Perspective);break;
    case 9:objects[a]->setBounds(b,b==99);break;case 10:v3_object_groups(manager,objects[a],b);break;case 11:transform(a,b);break;
    case 12:{std::ostringstream stream;set.DumpJson(stream,b);ga_text(v3_object_dump_pointers(stream.str()));break;}
    case 13:ga_int(objectId(set.GetObjectById(SelectMgr_SelectableObjectSet::BVHSubset(a),b)));break;
    case 14:for(auto& object:objects)set.Remove(object);break;
    case 15:cursor.Init(set);break;case 16:ga_flag(cursor.More());break;case 17:cursor.Next();break;case 18:ga_int(objectId(cursor.Value()));break;
    case 21:{SelectMgr_SelectableObjectSet copy=set;copy.Remove(objects[a]);copy.MarkDirty();copy.UpdateBVH(camera,NCollection_Vec2<int>(800+b*17,600+b*11));break;}
    case 19:camera=new Graphic3d_Camera(*camera);break;
    case 20:{gp_Trsf trsf;trsf.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,0,1)),b/8.);trsf.SetTranslationPart(gp_Vec(b*0.5,1.,-2.));
      for(const auto& prs:objects[a]->Presentations())for(const auto& group:prs->Groups())group->SetTransformation(trsf);break;}
  }}
  void state(){ga_flag(set.IsEmpty());for(auto& object:objects)ga_flag(set.Contains(object));
    for(int i=0;i<5;++i){auto subset=SelectMgr_SelectableObjectSet::BVHSubset(i);ga_flag(set.IsEmpty(subset));
      std::vector<int> ids;for(size_t j=0;j<objects.size();++j){try{ids.push_back(objectId(set.GetObjectById(subset,int(j))));}catch(const Standard_Failure&){break;}}
      ga_int(int(ids.size()));for(int id:ids)ga_int(id);ga_flag(retained[i]==set.BVH(subset));v3_entity_set_tree(*retained[i]);}
    for(auto& object:objects)ga_int(object->calls);
  }
};
void v3_selectable_object_set(std::istream& input){
  GaReader r{input};int count=r.integer(),seed=r.integer(),projection=r.integer(),steps=r.integer();
  V3ObjectSetFixture fixture(count,seed,projection);fixture.state();
  for(int i=0;i<steps;++i){int op=r.integer(),a=r.integer(),b=r.integer();GA_TRY(fixture.mutate(op,a,b))fixture.state();}ga_end();
}
