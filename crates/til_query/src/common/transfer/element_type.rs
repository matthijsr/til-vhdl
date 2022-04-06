use core::fmt;
use std::str::FromStr;

use bitvec::prelude::*;
use tydi_common::{
    error::{Error, Result},
    map::InsertionOrderedMap,
    name::Name,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementType {
    Null,
    Bits(BitVec),
    Group(InsertionOrderedMap<Name, ElementType>),
    Union(UnionElement),
}

impl ElementType {
    pub fn union(tag: BitVec, name: Name, union: ElementType, max_len: usize) -> Result<Self> {
        Ok(ElementType::Union(UnionElement::new(
            tag, name, union, max_len,
        )?))
    }

    pub fn len(&self) -> usize {
        match self {
            ElementType::Null => 0,
            ElementType::Bits(bits) => bits.len(),
            ElementType::Group(group) => group.iter().fold(0, |acc, (_, typ)| acc + typ.len()),
            ElementType::Union(union) => union.len(),
        }
    }

    /// Flattens the Element Type into a (Lsb0) BitVec.
    ///
    /// Can be used to address element lane signals directly.
    pub fn flatten(&self) -> BitVec {
        match self {
            ElementType::Null => BitVec::new(),
            ElementType::Bits(bits) => bits.clone(),
            ElementType::Group(group) => group.iter().flat_map(|(_, typ)| typ.flatten()).collect(),
            ElementType::Union(union) => union.flatten(),
        }
    }
}

impl fmt::Display for ElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementType::Null => write!(f, "Null"),
            ElementType::Bits(bits) => write!(f, "Bits({})", bits),
            ElementType::Group(group) => write!(
                f,
                "Group({})",
                group
                    .iter()
                    .map(|(n, x)| format!("{}: {}", n, x))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ElementType::Union(union) => write!(
                f,
                "Union(tag: {} ({}), union: {})",
                union.tag(),
                union.union().0,
                union.union_el()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionElement {
    tag: BitVec,
    union: (Name, Box<ElementType>),
    max_len: usize,
}

impl UnionElement {
    pub fn new(tag: BitVec, name: Name, union: ElementType, max_len: usize) -> Result<Self> {
        if union.len() > max_len {
            Err(Error::InvalidArgument(format!(
                "Union size exceeds max_len: {} > {}",
                union.len(),
                max_len
            )))
        } else {
            Ok(Self {
                tag,
                union: (name, Box::new(union)),
                max_len,
            })
        }
    }

    /// Creates a tag for a Union based on the number of fields in the Union
    /// type and the index of the selected field.
    pub fn new_tag(fields: usize, field_no: usize) -> Result<BitVec> {
        if fields == 0 {
            Err(Error::InvalidArgument(
                "The number of fields in a union can't be 0".to_string(),
            ))
        } else if field_no > fields {
            Err(Error::InvalidArgument(format!(
                "field_no > fields, {} > {}",
                field_no, fields
            )))
        } else {
            let tag = field_no.view_bits::<Lsb0>().to_bitvec();
            Ok(match (fields - 1).view_bits::<Lsb0>().last_one() {
                Some(last_one) => tag[0..=last_one].to_bitvec(),
                None => tag[0..=0].to_bitvec(),
            })
        }
    }

    pub fn tag(&self) -> &BitVec {
        &self.tag
    }

    pub fn union(&self) -> &(Name, Box<ElementType>) {
        &self.union
    }

    pub fn union_el(&self) -> &ElementType {
        self.union().1.as_ref()
    }

    /// The maximum length of the fields in this union. (Excludes the tag.)
    pub fn max_len(&self) -> usize {
        self.max_len
    }

    /// Returns the flat length of this Union element, based on the tag size
    /// and the maximum length of a field in the Union.
    pub fn len(&self) -> usize {
        self.tag().len() + self.max_len()
    }

    pub fn flatten(&self) -> BitVec {
        let mut result = self.tag().clone();
        let len_diff = self.max_len() - self.union_el().len();
        result.extend(self.union_el().flatten());
        if len_diff > 0 {
            result.extend(vec![false; len_diff].iter());
        }
        result
    }
}

impl FromStr for ElementType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "" || s.to_lowercase() == "null" {
            Ok(ElementType::Null)
        } else {
            let mut result = BitVec::new();
            for c in s.chars() {
                if c == '1' {
                    result.push(true);
                } else if c == '0' {
                    result.push(false);
                } else {
                    return Err(Error::InvalidArgument(format!(
                        "ElementType cannot be parsed from string \"{}\"",
                        s
                    )));
                }
            }
            Ok(ElementType::Bits(result))
        }
    }
}

impl From<BitVec> for ElementType {
    fn from(bits: BitVec) -> Self {
        ElementType::Bits(bits)
    }
}

impl From<UnionElement> for ElementType {
    fn from(el: UnionElement) -> Self {
        ElementType::Union(el)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_tag() -> Result<()> {
        assert_eq!(UnionElement::new_tag(5, 0)?, bitvec![0, 0, 0]);
        assert_eq!(UnionElement::new_tag(5, 1)?, bitvec![1, 0, 0]);
        assert_eq!(UnionElement::new_tag(1, 0)?, bitvec![0]);
        assert_eq!(UnionElement::new_tag(1, 1)?, bitvec![1]);

        Ok(())
    }

    #[test]
    fn test_flatten() -> Result<()> {
        assert_eq!(ElementType::Null.flatten(), bitvec![]);
        assert_eq!(
            ElementType::Bits(bitvec![0, 0, 1]).flatten(),
            bitvec![0, 0, 1]
        );
        let mut group_fields = InsertionOrderedMap::new();
        group_fields.try_insert(Name::try_new("a")?, ElementType::Bits(bitvec![0, 0, 1]))?;
        group_fields.try_insert(Name::try_new("b")?, ElementType::Bits(bitvec![0, 1, 1]))?;
        assert_eq!(
            ElementType::Group(group_fields).flatten(),
            bitvec![0, 0, 1, 0, 1, 1]
        );
        assert_eq!(
            ElementType::union(
                bitvec![1, 0],
                Name::try_new("a")?,
                ElementType::Bits(bitvec![0, 0, 1]),
                4
            )?
            .flatten(),
            bitvec![1, 0, 0, 0, 1, 0]
        );

        Ok(())
    }

    #[test]
    fn from_str_element_type() -> Result<()> {
        assert_eq!(ElementType::from_str("")?, ElementType::Null);
        assert_eq!(ElementType::from_str("null")?, ElementType::Null);
        assert_eq!(ElementType::from_str("Null")?, ElementType::Null);
        assert_eq!(ElementType::from_str("nULL")?, ElementType::Null);
        assert_eq!(
            ElementType::from_str("0100")?,
            ElementType::Bits(bitvec![0, 1, 0, 0])
        );

        Ok(())
    }
}
