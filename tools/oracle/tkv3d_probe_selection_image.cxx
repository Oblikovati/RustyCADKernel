// Selection fillers operate on the actual OCCT palette and image implementations.
#include <SelectMgr_SelectionImageFiller.hxx>
#include <Select3D_SensitiveSegment.hxx>
class V3ImageScene {
public:
  V3TraversalScene scene;
  std::vector<occ::handle<SelectMgr_Selection>> retained;
  std::vector<occ::handle<Select3D_SensitiveEntity>> entities;
  V3ImageScene(int mode,int failing):scene(V3TraversalConfig{1,1,0,0,0,0,1,0,failing,3}) {
    for(auto entity:scene.entities)entities.push_back(entity);
    entities.push_back(new Select3D_SensitiveSegment(scene.owners[0],gp_Pnt(-1.,0.,0.),gp_Pnt(1.,0.,0.)));
    Quantity_NameOfColor names[]={Quantity_NOC_RED,Quantity_NOC_GREEN,Quantity_NOC_BLUE1};
    for(int i=0;i<3;++i){auto object=scene.objects[i];retained.push_back(object->Selection(0));object->Attributes()->SetColor(Quantity_Color(names[i]));
      occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);selection->Add(entities[i*3]);
      if(i==0)selection->Add(entities[9]);object->AddSelection(selection,mode);
    }
  }
  void seed(int tag,double depth,const NCollection_Vec3<float>& normal){
    if(tag<0){scene.selector->Clear();return;}
    scene.selector->imageSeed(entities[tag],entities[tag]->OwnerId(),depth,normal);
  }
};
static void v3_image_init(Image_PixMap& image,int format,int width,int height,int top,int padding){
  image.InitZero(Image_Format(format),width,height,width*Image_PixMap::SizePixelBytes(Image_Format(format))+padding,17);image.SetTopDown(top!=0);
}
static void v3_image_state(const Image_PixMap& image){ga_int(image.SizeBytes());for(size_t i=0;i<image.SizeBytes();++i)ga_int(image.Data()[i]);}
void v3_selection_image(std::istream& input){
  GaReader r{input};int kind=r.integer(),format=r.integer(),width=r.integer(),height=r.integer(),top=r.integer(),padding=r.integer(),mode=r.integer();
  V3ImageScene fixture(mode,-1);Image_PixMap image;v3_image_init(image,format,width,height,top,padding);
  occ::handle<SelectMgr_SelectionImageFiller> filler;GA_TRY(filler=SelectMgr_SelectionImageFiller::CreateFiller(image,fixture.scene.selector.get(),StdSelect_TypeOfSelectionImage(kind)))ga_flag(!filler.IsNull());
  int count=r.integer();for(int i=0;i<count;++i){int tag=r.integer(),rank=r.integer();double depth=r.real();float nx=float(r.real()),ny=float(r.real()),nz=float(r.real());
    fixture.seed(tag,depth,NCollection_Vec3<float>(nx,ny,nz));if(!filler.IsNull()){GA_TRY(filler->Fill(i%width,i/width,rank))}
  }
  if(!filler.IsNull()){GA_TRY(filler->Flush())}v3_image_state(image);ga_end();
}
void v3_selector_pixmap(std::istream& input){
  GaReader r{input};int kind=r.integer(),format=r.integer(),width=r.integer(),height=r.integer(),top=r.integer(),padding=r.integer(),mode=r.integer(),picked=r.integer(),failing=r.integer();
  V3ImageScene fixture(mode,failing);Image_PixMap image;v3_image_init(image,format,width,height,top,padding);
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());auto view=viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();window->SetSize(80,60);view->SetWindow(window);
  view->SetCamera(v3_traversal_volume(V3TraversalConfig{1,1,0,0,0,0,1,0,-1,3}).Camera());window->SetSize(width,height);
  fixture.scene.selector->SetPixelTolerance(1);
  GA_TRY(ga_flag(fixture.scene.selector->ToPixMap(image,view,StdSelect_TypeOfSelectionImage(kind),picked)))
  v3_image_state(image);ga_int(fixture.scene.selector->NbPicked());ga_int(fixture.scene.selector->sorts);
  ga_int(fixture.scene.events.size());for(int event:fixture.scene.events)ga_int(event);ga_end();
}
