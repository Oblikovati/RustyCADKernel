// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! OCCT's StdFail package: the exceptions algorithms raise when a result is not available.
use crate::standard::exceptions::define_standard_exceptions;

define_standard_exceptions! {
    STD_FAIL_HIERARCHY;
    infinite_solutions => "StdFail_InfiniteSolutions" : "Standard_Failure";
    not_done => "StdFail_NotDone" : "Standard_Failure";
    undefined => "StdFail_Undefined" : "Standard_Failure";
    undefined_derivative => "StdFail_UndefinedDerivative" : "Standard_DomainError";
    undefined_value => "StdFail_UndefinedValue" : "Standard_DomainError";
}
