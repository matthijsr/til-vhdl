use crate::common::vhdl_name::VhdlName;

pub trait Label: Sized {
    fn with_label(mut self, label: impl Into<VhdlName>) -> Self {
        self.set_label(label);
        self
    }

    fn set_label(&mut self, label: impl Into<VhdlName>);

    fn label(&self) -> Option<&VhdlName>;
}
