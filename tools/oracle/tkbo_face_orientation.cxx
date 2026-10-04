#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
int main(){std::cout<<std::setprecision(17);
 for(int kind=0;kind<5;kind++)for(int copy=0;copy<3;copy++)for(int splitOri=0;splitOri<4;splitOri++)for(int originalOri=0;originalOri<4;originalOri++){
  auto first=pcurve_fixture(kind).face;
  auto second=copy?pcurve_fixture(kind).face:first;
  if(copy==2){auto surface=BRep_Tool::Surface(first);TopoDS_Face empty;BRep_Builder().MakeFace(empty,surface,1e-7);first=empty;}
  first.Orientation(TopAbs_Orientation(splitOri));second.Orientation(TopAbs_Orientation(originalOri));
  Handle(IntTools_Context) context=new IntTools_Context();int error=99;
  bool result=BOPTools_AlgoTools::IsSplitToReverse(first,second,context,&error);
  int shapeError=99;bool shapeResult=BOPTools_AlgoTools::IsSplitToReverse((TopoDS_Shape)first,(TopoDS_Shape)second,context,&shapeError);
  std::cout<<kind<<' '<<copy<<' '<<splitOri<<' '<<originalOri<<' '<<result<<' '<<error<<' '<<shapeResult<<' '<<shapeError<<' '<<BOPTools_AlgoTools::Sense(first,second,context)<<'\n';
 }
}
