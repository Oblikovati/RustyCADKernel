// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

// Exact input generator for MathLin_EigenSearch_Test.RandomParityAndOrthogonality.
// Pinned source: 3d097a0328e71b826377d4814ab05ec3c3d23871; Linux libstdc++ profile.
#include <random>
#include <iostream>
#include <iomanip>
int main(){std::mt19937 gen(123456u);std::uniform_int_distribution<int> dim(2,32);std::uniform_real_distribution<double> val(-100,100);std::cout<<std::setprecision(17);for(int c=0;c<120;++c){int n=dim(gen);std::cout<<n;for(int i=0;i<n;++i){double d=val(gen),e=i==0?0:val(gen);std::cout<<' '<<d<<' '<<e;}std::cout<<'\n';}}
