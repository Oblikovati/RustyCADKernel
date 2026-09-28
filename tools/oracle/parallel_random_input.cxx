// Copyright (c) 2026 OPEN CASCADE SAS
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

// Input generation copied from OSD_Parallel_Test.cxx at the pinned OCCT revision.
// Run the native generator once to retain GCC's exact std::mt19937(42) sequence.
#include <random>
#include <iostream>
int main() {
  std::mt19937 generator(42);
  for (int index=0; index<5000; ++index) {
    std::cout << generator()%1000 << ((index+1)%25==0 ? '\n' : ' ');
  }
}
