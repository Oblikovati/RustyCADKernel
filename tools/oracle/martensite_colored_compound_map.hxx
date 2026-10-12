// Actual protected native specialization shared by Martensite map lifecycle probes.
#pragma once
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <Prs3d_Drawer.hxx>
#include <iostream>
class ColoredCompoundMapProbe : public AIS_ColoredShape
{
public:
  using DataMap = DataMapOfDrawerCompd;
};
using DrawerCompounds = ColoredCompoundMapProbe::DataMap;
