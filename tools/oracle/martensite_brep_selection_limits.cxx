// Compile the actual pinned private helper in this probe TU; no rewritten oracle or OCCT library rebuild.
#include "../../../OCCT/src/Visualization/TKV3d/StdSelect/StdSelect_BRepSelectionTool.cxx"
#include <iostream>
#include <limits>
#include <iomanip>
class SelectionLimitCurveProbe:public Adaptor3d_Curve
{
public:
  int scenario;
  mutable int calls=0;
  mutable double firstCall=0,lastCall=0;
  SelectionLimitCurveProbe(int index):scenario(index){}
  double FirstParameter()const override{return scenario==0?1.:scenario==3?1.:-Precision::Infinite();}
  double LastParameter()const override{return scenario==0?4.:scenario==2?4.:Precision::Infinite();}
  gp_Pnt EvalD0(double parameter)const override
  {
    if(calls++==0)firstCall=parameter;
    lastCall=parameter;
    return gp_Pnt(scenario==4?0.:scenario==5?std::numeric_limits<double>::quiet_NaN():parameter,0,0);
  }
};
int main()
{
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<7;++scenario)
  {
    SelectionLimitCurveProbe curve(scenario);
    double first=0,last=0;
    bool limited=FindLimits(curve,scenario==6?std::numeric_limits<double>::quiet_NaN():20.,first,last);
    std::cout<<"limits "<<scenario<<' '<<limited<<' '<<first<<' '<<last<<' '<<curve.calls<<' '<<curve.firstCall<<' '<<curve.lastCall<<'\n';
  }
}
