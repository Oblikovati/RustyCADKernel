// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the commands of `Init` (second part).

//! The second part of the commands of `IFSelect_Functions::Init` (from `dispsel`).
use super::dispatches::*;
use super::edit::{fun_editapply, fun_editclear, fun_editlist, fun_editload, fun_editvalue};
use super::register::Command;
use super::selection_items::*;
use super::selections::*;

/// The commands of `Init` from `dispsel`, in the native order.
pub(super) const COMMANDS_TAIL: &[Command] = &[
    (
        "dispsel",
        "disp:Dispatch sel:Selection  -> Final Selection of Dispatch",
        fun51,
        false,
    ),
    ("dispone", "cree DispPerOne", fun_dispone, true),
    ("dispglob", "cree DispGlobal", fun_dispglob, true),
    (
        "dispcount",
        "count:IntParam  : cree DispPerCount",
        fun_dispcount,
        true,
    ),
    (
        "dispfile",
        "files:IntParam  : cree DispPerFiles",
        fun_dispfiles,
        true,
    ),
    (
        "dispsign",
        "sign:Signature  : cree DispPerSignature",
        fun_dispsign,
        true,
    ),
    (
        "dumpdisp",
        "disp:Dispatch   : Affiche le Statut d'un Dispatch",
        fun56,
        false,
    ),
    (
        "xremove",
        "nom  : Remove a Control Item from the Session",
        fun57,
        false,
    ),
    (
        "evaldisp",
        "mode=[0-3]  disp:Dispatch  : Evaluates one or more Dispatch(es)",
        fun58,
        false,
    ),
    (
        "evaladisp",
        "mode=[0-3]  disp:Dispatch [givelist]  : Evaluates a Dispatch (on a GiveList)",
        fun_evaladisp,
        false,
    ),
    (
        "writedisp",
        "filepattern  disp:Dispatch [givelist]  : Writes Entities by Splitting by a Dispatch",
        fun_writedisp,
        false,
    ),
    (
        "evalcomplete",
        "Complete Evaluation of the Distribution",
        fun59,
        false,
    ),
    (
        "runcheck",
        "affiche LastRunCheckList (write,modif)",
        fun60,
        false,
    ),
    (
        "runtranformer",
        "transf:Transformer  : Applique un Transformer",
        fun61,
        false,
    ),
    (
        "copy",
        "cree TransformStandard, option Copy, vide",
        fun62,
        true,
    ),
    (
        "onthespot",
        "cree TransformStandard, option OntheSpot, vide",
        fun63,
        true,
    ),
    (
        "runcopy",
        "modif:ModelModifier [givelist] : Run <modif> via TransformStandard option Copy",
        fun6465,
        false,
    ),
    (
        "runonthespot",
        "modif:ModelModifier [givelist] : Run <modif> via TransformStandard option OnTheSpot",
        fun6465,
        false,
    ),
    ("reorder", "[f ou t] reordonne le modele", fun66, true),
    (
        "toggle",
        "sel:Selection genre Extract  : Toggle Direct/Reverse",
        fun70,
        false,
    ),
    (
        "input",
        "sel:Selection genre Deduct ou Extract  input:Selection  : Set Input",
        fun71,
        false,
    ),
    ("modelroots", "cree SelectModelRoots", fun72, true),
    (
        "range",
        "options... : create SelectRange ...; simply for help",
        fun73,
        true,
    ),
    ("roots", "cree SelectRoots (local roots)", fun74, true),
    ("shared", "cree SelectShared", fun75, true),
    (
        "diff",
        "[main:Selection diff:Selection]  : cree SelectDiff",
        fun76,
        true,
    ),
    (
        "selmain",
        "sel:Selection genre Control  main:Selection  : Set Main Input",
        fun77,
        false,
    ),
    (
        "selsecond",
        "sel:Selection genre Control  sec:Selection   : Set Second Input",
        fun78,
        false,
    ),
    ("modelall", "cree SelectModelAll", fun79, true),
    (
        "seladd",
        "sel:Selection genre Combine  input:Selection  : Add Selection",
        fun80,
        false,
    ),
    (
        "selrem",
        "sel:Selection genre Combine  input:Selection  : Remove Selection",
        fun81,
        false,
    ),
    (
        "number",
        "num:IntParam  : Cree SelectEntityNumber",
        fun82,
        true,
    ),
    (
        "union",
        "cree SelectUnion (vide), cf aussi combadd, combrem",
        fun83,
        true,
    ),
    (
        "intersect",
        "cree SelectIntersection (vide), cf aussi combadd, combrem",
        fun84,
        true,
    ),
    (
        "typexact",
        "type:string  : cree SelectTextType Exact",
        fun85,
        true,
    ),
    (
        "errors",
        "cree SelectErrorEntities (from file)",
        fun86,
        true,
    ),
    ("unknown", "cree SelectUnknownEntities", fun87, true),
    ("sharing", "cree SelectSharing", fun88, true),
    (
        "typecontain",
        "type:string  : cree SelectTextType Contains",
        fun89,
        true,
    ),
    (
        "pointed",
        "cree SelectPointed [num/sel num/sel]",
        fun90,
        true,
    ),
    (
        "setpointed",
        "sel:SelectPointed  : edit SelectPointed. simply for help",
        fun91,
        false,
    ),
    (
        "setlist",
        "sel:SelectPointed  : edit SelectPointed. simply for help",
        fun91,
        false,
    ),
    (
        "incorrect",
        "cree SelectIncorrectEntities (computed)",
        fun92,
        true,
    ),
    (
        "signsel",
        "sign:Signature|cnt:Counter text:string [e(D)|c] : cree SelectSignature",
        fun93,
        true,
    ),
    (
        "signcounter",
        "sign:Signature : cree SignCounter",
        fun94,
        true,
    ),
    (
        "nbselected",
        "applied:Selection : cree GraphCounter(=NbSelected)",
        funbselected,
        true,
    ),
    (
        "editlist",
        "editor or editform : lists defs + values",
        fun_editlist,
        false,
    ),
    (
        "editvalue",
        "editform paramname [newval or .] : lists-changes a value",
        fun_editvalue,
        false,
    ),
    (
        "editclear",
        "editform [paramname] : clears edition on all or one param",
        fun_editclear,
        false,
    ),
    (
        "editload",
        "editform [entity-id] : loads from model or an entity",
        fun_editload,
        false,
    ),
    (
        "editapply",
        "editform [keep] : applies on loaded data",
        fun_editapply,
        false,
    ),
];
