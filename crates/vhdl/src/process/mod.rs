pub mod statement;

use tydi_common::map::InsertionOrderedMap;
use tydi_intern::Id;

use crate::{common::vhdl_name::VhdlName, declaration::ObjectDeclaration};

use self::statement::SequentialStatement;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Process {
    /// The (optional) label of this process.
    label: Option<VhdlName>,
    /// The sensitivity list of this process, indexed by the names of the
    /// objects.
    sensitivity_list: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// Variable declarations on this process. Indexed by their names.
    ///
    /// While a process's declarative part can technically contain different
    /// declarations. For our purposes, only variables relevant at this time.
    variable_declarations: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// The process's statements.
    statements: Vec<SequentialStatement>,
}
