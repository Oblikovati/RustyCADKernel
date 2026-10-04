#pragma once
#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools3D.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
int seam_probe(bool periodic){
 std::cout<<std::setprecision(17);
 for(int kind:{1,2})for(int mode=0;mode<7;mode++)for(int eori=0;eori<4;eori++)for(int fori=0;fori<4;fori++){
  if(periodic&&mode==3)continue;
  auto face=pcurve_fixture(kind).face;TopoDS_Edge origin,ordinary;
  for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next()){auto edge=TopoDS::Edge(ex.Current());if(BRep_Tool::IsClosed(edge,face)){if(origin.IsNull())origin=edge;}else ordinary=edge;}
  if(mode==5&&ordinary.IsNull())continue;
  origin.Orientation(mode==1?TopAbs_REVERSED:TopAbs_FORWARD);
  double first,last;auto spatial=BRep_Tool::Curve(origin,first,last);TopoDS_Edge split;BRep_Builder builder;builder.MakeEdge(split,spatial,1e-7);builder.Range(split,first,last);
  if(mode==4)split=TopoDS::Edge(origin.EmptyCopied());
  else if(mode!=3){Handle(Geom2d_Curve) curve=BRep_Tool::CurveOnSurface(origin,face,first,last);curve=occ::down_cast<Geom2d_Curve>(curve->Copy());if(mode==2)curve->Translate(gp_Vec2d(.1,.1));if(mode==6){double f=curve->ReversedParameter(last),l=curve->ReversedParameter(first);curve->Reverse();first=f;last=l;}builder.UpdateEdge(split,curve,face,1e-7);builder.Range(split,face,first,last);}
  if(mode==5)origin=ordinary;
  split.Orientation(TopAbs_Orientation(eori));face.Orientation(TopAbs_Orientation(fori));
  bool result=periodic?BOPTools_AlgoTools3D::DoSplitSEAMOnFace(split,face):BOPTools_AlgoTools3D::DoSplitSEAMOnFace(origin,split,face);
  std::cout<<kind<<' '<<mode<<' '<<eori<<' '<<fori<<' '<<result<<' '<<BRep_Tool::IsClosed(split,face);
  face.Orientation(TopAbs_FORWARD);
  for(int side=0;side<2;side++){split.Orientation(TopAbs_Orientation(side));auto curve=BRep_Tool::CurveOnSurface(split,face,first,last);std::cout<<' '<<curve.IsNull()<<' '<<first<<' '<<last;if(!curve.IsNull()){auto point=curve->Value(.5*(first+last));std::cout<<' '<<point.X()<<' '<<point.Y();}}
  std::cout<<'\n';
 }
 return 0;
}
