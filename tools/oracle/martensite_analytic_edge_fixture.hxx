// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; authored analytic inputs, not root evidence.
#pragma once
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <array>
#include <vector>
using CensusControls = std::vector<std::array<double, 2>>;
static TopoDS_Shape CensusEdge(const CensusControls& controls, const std::vector<double>& weights = {})
{
  NCollection_Array1<gp_Pnt> poles(1, static_cast<int>(controls.size()));
  for (int i=1; i<=poles.Upper(); ++i) poles(i)=gp_Pnt(controls[i-1][0],controls[i-1][1],0.);
  occ::handle<Geom_BezierCurve> curve;
  if (weights.empty()) curve=new Geom_BezierCurve(poles);
  else {
    NCollection_Array1<double> native(1, static_cast<int>(weights.size()));
    for (int i=1; i<=native.Upper(); ++i) native(i)=weights[i-1];
    curve=new Geom_BezierCurve(poles,native);
  }
  return BRepBuilderAPI_MakeEdge(curve).Edge();
}
