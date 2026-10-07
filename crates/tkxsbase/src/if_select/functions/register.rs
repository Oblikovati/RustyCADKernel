// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: `Init`.

//! `IFSelect_Functions::Init`: the commands registered once per process, in the native
//! order, in the group "DE: General".
use super::IFSelectFunctions;
use super::command_table::COMMANDS_TAIL;
use super::entities::*;
use super::general::*;
use super::lists::*;
use super::modifiers::*;
use crate::if_select::{IFSelectAct, IFSelectActFunc};
use rustycad_tkernel::standard::Failure;
use std::sync::atomic::{AtomicBool, Ordering};

/// The native static `THE_IFSelect_Functions_initactor`.
static INIT_ACTOR: AtomicBool = AtomicBool::new(false);

/// A command: its name, its help, its function, and true for a command creating an
/// item (`AddFSet`).
pub(super) type Command = (&'static str, &'static str, IFSelectActFunc, bool);

/// The commands of `Init`, in the native order (continued in `command_table`).
const COMMANDS: &[Command] = &[
    (
        "xstatus",
        "Lists XSTEP Status : Version, System Name ...",
        funstatus,
        false,
    ),
    (
        "handler",
        "Toggle status catch Handler Error of the session",
        fun1,
        false,
    ),
    (
        "xload",
        "file:string  : Read File -> Load Model",
        fun3,
        false,
    ),
    (
        "xread",
        "file:string  : Read File -> Load Model",
        fun3,
        false,
    ),
    (
        "writeall",
        "file:string  : Write all model (no split)",
        fun4,
        false,
    ),
    (
        "writesel",
        "file:string sel:Selection : Write Selected (no split)",
        fun5,
        false,
    ),
    (
        "writeent",
        "file:string  n1ent n2ent...:integer : Write Entite(s) (no split)",
        fun6,
        false,
    ),
    (
        "writent",
        "file:string  n1ent n2ent...:integer : Write Entite(s) (no split)",
        fun6,
        false,
    ),
    (
        "elabel",
        "nument:integer   : Displays Label Model of an entity",
        fun7,
        false,
    ),
    (
        "enum",
        "label:string  : Displays entities n0.s of which Label Model ends by..",
        fun8,
        false,
    ),
    (
        "listtypes",
        "List nb entities per type. Optional selection name  else all model",
        fun9,
        false,
    ),
    ("count", "Count : counter [selection]", funcount, false),
    (
        "listcount",
        "List Counted : counter [selection [nument]]",
        funcount,
        false,
    ),
    (
        "sumcount",
        "Summary Counted : counter [selection [nument]]",
        funcount,
        false,
    ),
    ("signtype", "Sign Type [newone]", funsigntype, false),
    (
        "signcase",
        "signature : displays possible cases",
        funsigncase,
        false,
    ),
    (
        "estatus",
        "ent/nument : displays status of an entity",
        fun10,
        false,
    ),
    (
        "data",
        "Data (DumpModel); whole help : data tout court",
        fun11,
        false,
    ),
    (
        "entity",
        "give n0 ou id of entity [+ level]",
        fundumpent,
        false,
    ),
    (
        "signature",
        "signature name + n0/ident entity",
        funsign,
        false,
    ),
    (
        "queryparent",
        " give 2 n0s/labels of entities : dad son",
        funqp,
        false,
    ),
    (
        "dumpshare",
        "Dump Share (dispatches, IntParams)",
        fun12,
        false,
    ),
    (
        "listitems",
        "List Items [label else all]  ->Type,Label[,Name]",
        fun13,
        false,
    ),
    ("integer", "value:integer : cree un IntParam", fun14, true),
    (
        "setint",
        "name:IntParam   newValue:integer  : Change valeur IntParam",
        fun15,
        false,
    ),
    ("text", "value:string  : cree un TextParam", fun16, true),
    (
        "settext",
        "Name:TextParam  newValue:string   : Change valeur TextParam",
        fun17,
        false,
    ),
    (
        "dumpsel",
        "Dump Selection followed by Name of Selection to dump",
        fun19,
        false,
    ),
    (
        "evalsel",
        "name:Selection [num/sel]  : Evalue une Selection",
        fun20,
        false,
    ),
    (
        "givelist",
        "num/sel [num/sel ...]  : Evaluates GiveList",
        fun20,
        false,
    ),
    (
        "giveshort",
        "num/sel [num/sel ...]  : GiveList in short form",
        fun20,
        false,
    ),
    (
        "givepointed",
        "num/sel [num/sel ...]  : GiveList to fill a SelectPointed",
        fun20,
        false,
    ),
    (
        "makelist",
        "listname [givelist] : Makes a List(SelectPointed) from GiveList",
        fun20,
        false,
    ),
    (
        "givecount",
        "num/sel [num/sel ...]  : Counts GiveList",
        fun20c,
        false,
    ),
    (
        "selsuite",
        "sel sel ...  : Creates a SelectSuite",
        funselsuite,
        true,
    ),
    (
        "clearitems",
        "Clears all items (selections, dispatches, etc)",
        fun21,
        false,
    ),
    (
        "cleardata",
        "mode:a-g-c-p  : Clears all or some data (model, check...)",
        fun22,
        false,
    ),
    (
        "itemlabel",
        "xxx xxx : liste items having this label",
        fun24,
        false,
    ),
    (
        "param",
        "[-p Pattern] - displays all parameters or filtered by pattern;\npar_name - displays parameter;\npar_name par_value - changes parameter's value",
        fun27,
        false,
    ),
    ("sentfiles", "Lists files sent from last Load", fun29, false),
    (
        "fileprefix",
        "prefix:string    : definit File Prefix",
        fun30,
        false,
    ),
    (
        "fileext",
        "extent:string    : definit File Extension",
        fun31,
        false,
    ),
    (
        "fileroot",
        "disp:Dispatch  root:string  : definit File Root sur un Dispatch",
        fun32,
        false,
    ),
    (
        "filedef",
        "defroot:string   : definit File DefaultRoot",
        fun33,
        false,
    ),
    (
        "evalfile",
        "Evaluation du FileNaming et memorisation",
        fun34,
        false,
    ),
    ("clearfile", "Efface la liste d'EvalFile", fun35, false),
    (
        "xsplit",
        "[disp:Dispatch  sinon tout]  : Split, la grande affaire !",
        fun36,
        false,
    ),
    (
        "remaining",
        "options... : Remaining Entities, help complet par  remaining ?",
        fun37,
        false,
    ),
    (
        "setcontent",
        "sel:Selection mode:k ou r  : Restreint contenu du modele",
        fun38,
        false,
    ),
    ("listmodif", "List Final Modifiers", fun40, false),
    (
        "dumpmodif",
        "modif:Modifier  : Affiche le Statut d'un Modifier",
        fun41,
        false,
    ),
    (
        "modifsel",
        "modif:Modifier [sel:Selection]  : Change/Cancel Selection of Modifier",
        fun42,
        false,
    ),
    (
        "setapplied",
        "modif:Modifier [name:un item sinon sortie fichier]  : Applique un Modifier",
        fun43,
        false,
    ),
    (
        "resetapplied",
        "modif:Modifier  : Remove a Modifier from file output",
        fun44,
        false,
    ),
    (
        "modifmove",
        "modif:Modifier M(model)/F(file) avant,apres:integer  : Deplace un Modifier (sortie fichier)",
        fun45,
        false,
    ),
];

impl IFSelectFunctions {
    /// `Init`: the commands registered (once per process) in the group "DE: General".
    pub fn init() -> Result<(), Failure> {
        if INIT_ACTOR.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        IFSelectAct::set_group("DE: General", "");
        for (name, help, func, set) in COMMANDS.iter().chain(COMMANDS_TAIL) {
            if *set {
                IFSelectAct::add_f_set(name, help, *func)?;
            } else {
                IFSelectAct::add_func(name, help, *func)?;
            }
        }
        Ok(())
    }
}
