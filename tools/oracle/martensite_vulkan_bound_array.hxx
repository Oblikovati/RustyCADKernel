// The factory discards some index/bound configurations. Exercise the protected
// constructor's broader buffer contract, matching Martensite's retained inputs.
#pragma once
#include <Graphic3d_ArrayOfPrimitives.hxx>
class MartensiteOracleBoundArray : public Graphic3d_ArrayOfPrimitives {
public:
  MartensiteOracleBoundArray(Graphic3d_TypeOfPrimitiveArray kind, int count,
                             int bounds, int edges, Graphic3d_ArrayFlags flags)
      : Graphic3d_ArrayOfPrimitives(kind, count, bounds, edges, flags) {}
};
