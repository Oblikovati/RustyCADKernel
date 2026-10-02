#include <ShapeFix_WireSegment.hxx>
void sh_ws_patch(const ShapeFix_WireSegment &segment,int index) {
  int a,b,c,d;
  segment.GetPatchIndex(index,a,b,c,d);
  ga_int(a);ga_int(b);ga_int(c);ga_int(d);
  ga_flag(segment.CheckPatchIndex(index));
}
void sh_ws_state(const ShapeFix_WireSegment &segment,const TpShapes &shapes) {
  ga_int(segment.Orientation());ga_int(segment.NbEdges());
  ga_flag(segment.WireData()->ManifoldMode());
  for(int i=1;i<=segment.NbEdges();++i) sh_wd_edge(shapes,segment.Edge(i));
  GA_TRY(tp_shape(segment.FirstVertex()))
  GA_TRY(tp_shape(segment.LastVertex()))
  GA_TRY(ga_flag(segment.IsClosed()))
  ga_flag(segment.IsVertex());tp_shape(segment.GetVertex());
  for(int i=0;i<=segment.NbEdges()+1;++i) {GA_TRY(sh_ws_patch(segment,i))}
  GA_TRY(sh_wd_edge(shapes,segment.Edge(-1)))
}
void sh_ws_edit(ShapeFix_WireSegment &segment,const TpShapes &shapes,int op,const std::vector<int> &a) {
  if(op==2)segment.Orientation(TopAbs_Orientation(a[0]));
  else if(op==3)segment.AddEdge(a[0],sh_edge(shapes,a[1]));
  else if(op==4)segment.AddEdge(a[0],sh_edge(shapes,a[1]),a[2],a[3],a[4],a[5]);
  else if(op==5)segment.SetEdge(a[0],sh_edge(shapes,a[1]));
  else if(op==6)segment.SetPatchIndex(a[0],a[1],a[2],a[3],a[4]);
  else if(op==7)segment.DefineIUMin(a[0],a[1]);
  else if(op==8)segment.DefineIUMax(a[0],a[1]);
  else if(op==9)segment.DefineIVMin(a[0],a[1]);
  else if(op==10)segment.DefineIVMax(a[0],a[1]);
  else if(op==11)segment.SetVertex(TopoDS::Vertex(sh_shape(shapes,a[0])));
  else if(op==13)segment.WireData()->Add(sh_edge(shapes,a[0]),a[1]);
  else if(op==14)segment.Load(segment.WireData());
  else if(op==16)segment.Load({});
}
void sh_ws_steps(ShapeFix_WireSegment *segments,const occ::handle<ShapeExtend_WireData> &source,const TpShapes &shapes,GaReader &reader) {
  int count=reader.integer();
  for(int k=0;k<count;++k) {
    int target=reader.integer(),op=reader.integer(),argc=reader.integer();
    std::vector<int> operands(argc);for(int &a:operands)a=reader.integer();
    GA_TRY({
      if(op==0)segments[target].Clear();
      else if(op==1)segments[target].Load(source);
      else if(op==12)segments[target]=segments[1-target];
      else if(op==15)segments[target].Load(segments[1-target].WireData());
      else sh_ws_edit(segments[target],shapes,op,operands);
    })
    for(auto *segment=segments;segment!=segments+2;++segment)sh_ws_state(*segment,shapes);
    ga_flag(segments[0].WireData()==segments[1].WireData());
    ga_flag(segments[0].WireData()==source);
  }
}
void sh_wire_segment_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY({
    TpShapes shapes=tp_shapes(reader);
    occ::handle<ShapeExtend_WireData> source=new ShapeExtend_WireData;
    source->ManifoldMode()=reader.flag();
    int count=reader.integer();for(int i=0;i<count;++i)source->Add(sh_edge(shapes,reader.integer()));
    int ctor=reader.integer(),orientation=reader.integer();
    ShapeFix_WireSegment segments[2];
    if(ctor==1)segments[0]=ShapeFix_WireSegment(source,TopAbs_Orientation(orientation));
    if(ctor==2) {
      occ::handle<ShapeExtend_WireData> nullWire;
      segments[0]=ShapeFix_WireSegment(nullWire,TopAbs_Orientation(orientation));
    }
    segments[1]=segments[0];
    sh_ws_state(segments[0],shapes);
    sh_ws_steps(segments,source,shapes,reader);
  })
  ga_end();
}
