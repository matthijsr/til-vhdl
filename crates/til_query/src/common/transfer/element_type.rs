use bitvec::prelude::*;
use tydi_common::{
    error::{Error, Result},
    insertion_ordered_map::InsertionOrderedMap,
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
            ElementType::Union(union) => union.tag().len() + union.union().1.len(),
        }
    }

    pub fn flatten(&self) -> BitVec {
        match self {
            ElementType::Null => BitVec::new(),
            ElementType::Bits(bits) => bits.clone(),
            ElementType::Group(group) => group.iter().flat_map(|(_, typ)| typ.flatten()).collect(),
            ElementType::Union(union) => union.flatten(),
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

    pub fn tag(&self) -> &BitVec {
        &self.tag
    }

    pub fn union(&self) -> &(Name, Box<ElementType>) {
        &self.union
    }

    pub fn union_el(&self) -> &ElementType {
        self.union().1.as_ref()
    }

    pub fn max_len(&self) -> usize {
        self.max_len
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
    fn test_flatten() -> Result<()> {
        assert_eq!(ElementType::Null.flatten(), bitvec![]);

        Ok(())
    }
}
