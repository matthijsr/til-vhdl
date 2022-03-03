use til_query::common::physical::{
    complexity::Complexity, element::Element, fields::Fields, signal_list::SignalList,
};
use tydi_common::numbers::NonNegative;
use tydi_intern::Id;
use tydi_vhdl::declaration::{AliasDeclaration, ObjectDeclaration};

pub mod port;

pub struct VhdlPhysicalStream {
    /// The objects making up a physical stream.
    signal_list: SignalList<Id<ObjectDeclaration>>,
    /// Elements, as lanes with fields,
    /// aliasing the data, last, stai, endi and strb signals of the SignalList.
    elements: Vec<Element<AliasDeclaration>>,
    /// Dimensionality.
    dimensionality: NonNegative,
    /// Complexity.
    complexity: Complexity,
    /// User-defined transfer content.
    user: Fields<AliasDeclaration>,
}
