use super::{eval_implementation::ImplementationDef, eval_interface::InterfaceDef, Def};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamletDef {
    interface: Def<InterfaceDef>,
    implementation: Def<ImplementationDef>,
}

impl StreamletDef {
    pub fn interface(&self) -> &Def<InterfaceDef> {
        &self.interface
    }

    pub fn implementation(&self) -> &Def<ImplementationDef> {
        &self.implementation
    }
}
