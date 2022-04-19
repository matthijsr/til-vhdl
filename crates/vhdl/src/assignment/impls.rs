use tydi_common::error::Result;

use crate::{
    common::vhdl_name::VhdlName,
    usings::{ListUsings, Usings},
};

use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecValue, AssignDeclaration, Assignment,
    AssignmentKind, DirectAssignment, ValueAssignment,
};

impl ListUsings for AssignmentKind {
    fn list_usings(&self) -> Result<Usings> {
        let mut usings = Usings::new_empty();
        match self {
            AssignmentKind::Object(_) => (),
            AssignmentKind::Direct(direct) => match direct {
                DirectAssignment::Value(value) => match value {
                    ValueAssignment::Bit(_) => (),
                    ValueAssignment::BitVec(bitvec) => match bitvec {
                        BitVecValue::Others(_) => (),
                        BitVecValue::Full(_) => (),
                        BitVecValue::Unsigned(_) | BitVecValue::Signed(_) => {
                            usings.add_using(VhdlName::try_new("ieee")?, "numeric_std.all")?;
                        }
                    },
                    ValueAssignment::Time(_) => (),
                    ValueAssignment::Boolean(_) => (),
                },
                DirectAssignment::FullRecord(rec) => {
                    for fa in rec {
                        usings.combine(&fa.assignment().list_usings()?);
                    }
                }
                DirectAssignment::FullArray(arr) => match arr {
                    ArrayAssignment::Direct(direct) => {
                        for ak in direct {
                            usings.combine(&ak.list_usings()?);
                        }
                    }
                    ArrayAssignment::Sliced { direct, others } => {
                        for ra in direct {
                            usings.combine(&ra.assignment().list_usings()?);
                        }
                        if let Some(value) = others {
                            usings.combine(&value.list_usings()?);
                        }
                    }
                    ArrayAssignment::Others(ak) => {
                        usings.combine(&ak.list_usings()?);
                    }
                },
            },
        }
        Ok(usings)
    }
}

impl ListUsings for Assignment {
    fn list_usings(&self) -> Result<Usings> {
        self.kind().list_usings()
    }
}

impl ListUsings for AssignDeclaration {
    fn list_usings(&self) -> Result<Usings> {
        self.assignment().list_usings()
    }
}
