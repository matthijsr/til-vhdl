use std::{convert::TryFrom, ops::Range, str::FromStr};

use tydi_common::{
    error::{Error, Result},
    numbers::NonNegative,
};

use super::element_type::ElementType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element {
    /// The data transfered on the Element.
    ///
    /// If None, the data on this Element is considered inactive.
    data: Option<ElementType>,
    /// Indicates whether this is the last element for a dimension or set of
    /// dimensions.
    ///
    /// If None, this is not the last element of any dimension.
    last: Option<Range<NonNegative>>,
}

impl Element {
    pub fn new(
        data: Option<impl Into<ElementType>>,
        last: Option<Range<NonNegative>>,
    ) -> Result<Self> {
        let result = Self {
            data: data.map(|x| x.into()),
            last: None,
        };
        if let Some(last) = last {
            result.with_last(last)
        } else {
            Ok(result)
        }
    }

    /// Attempts to set the data field based on a string.
    ///
    /// "-" and "inactive" (case-insensitive) represent inactive data.
    ///
    /// "" and "null" (case-insenstive) represent Null.
    ///
    /// "010" and other sets of bits represent Bits.
    pub fn new_data_from_str<'a>(data: &'a str) -> Result<Self> {
        let data = if data == "-" || data.to_lowercase() == "inactive" {
            None
        } else {
            Some(ElementType::from_str(data)?)
        };
        Ok(Self {
            data: data,
            last: None,
        })
    }

    pub fn new_data(data: impl Into<ElementType>) -> Self {
        Self {
            data: Some(data.into()),
            last: None,
        }
    }

    pub fn new_inactive() -> Self {
        Self {
            data: None,
            last: None,
        }
    }

    pub fn with_last(mut self, last: Range<NonNegative>) -> Result<Self> {
        if self.is_active() && last.start > 0 {
            Err(Error::InvalidArgument(format!(
                            "Cannot assert this element as being the last of dimensions {}..{}. Elements with active data can only transferred as the innermost dimension (0).",
                            last.start, last.end
                        )))
        } else {
            self.last = Some(last);
            Ok(self)
        }
    }

    pub fn data(&self) -> &Option<ElementType> {
        &self.data
    }

    pub fn last(&self) -> &Option<Range<NonNegative>> {
        &self.last
    }

    pub fn is_active(&self) -> bool {
        if let Some(_) = self.data() {
            true
        } else {
            false
        }
    }

    pub fn element_size(&self) -> Option<usize> {
        match self.data() {
            Some(data) => Some(data.len()),
            None => None,
        }
    }

    /// Returns the maximum dimension this elements asserts to be last of.
    ///
    /// Will return 0 if this element was not asserted as last of any dimension.
    pub fn max_last(&self) -> NonNegative {
        match self.last() {
            Some(last) => last.end,
            None => 0,
        }
    }
}

impl Default for Element {
    fn default() -> Self {
        Self {
            data: Default::default(),
            last: Default::default(),
        }
    }
}

impl<T: Into<ElementType>> From<T> for Element {
    fn from(value: T) -> Self {
        Element::new_data(value.into())
    }
}

impl<'a> TryFrom<&'a str> for Element {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self> {
        Element::new_data_from_str(value)
    }
}

impl FromStr for Element {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Element::try_from(s)
    }
}

impl<T: Into<ElementType>> TryFrom<(T, Range<NonNegative>)> for Element {
    type Error = Error;

    fn try_from(value: (T, Range<NonNegative>)) -> Result<Self> {
        Element::new_data(value.0).with_last(value.1)
    }
}

impl<'a> TryFrom<(&'a str, Range<NonNegative>)> for Element {
    type Error = Error;

    fn try_from(value: (&'a str, Range<NonNegative>)) -> Result<Self> {
        Element::new_data_from_str(value.0)?.with_last(value.1)
    }
}

impl<T: Into<ElementType>> TryFrom<(T, Option<Range<NonNegative>>)> for Element {
    type Error = Error;

    fn try_from(value: (T, Option<Range<NonNegative>>)) -> Result<Self> {
        if let Some(range) = value.1 {
            Element::new_data(value.0).with_last(range)
        } else {
            Ok(Element::new_data(value.0))
        }
    }
}

impl<'a> TryFrom<(&'a str, Option<Range<NonNegative>>)> for Element {
    type Error = Error;

    fn try_from(value: (&'a str, Option<Range<NonNegative>>)) -> Result<Self> {
        if let Some(range) = value.1 {
            Element::new_data_from_str(value.0)?.with_last(range)
        } else {
            Element::new_data_from_str(value.0)
        }
    }
}

impl<'a> TryFrom<Option<&'a str>> for Element {
    type Error = Error;

    fn try_from(value: Option<&'a str>) -> Result<Self> {
        if let Some(data) = value {
            Element::new_data_from_str(data)
        } else {
            Ok(Element::new_inactive())
        }
    }
}

impl<T: Into<ElementType>> TryFrom<(Option<T>, Option<Range<NonNegative>>)> for Element {
    type Error = Error;

    fn try_from(value: (Option<T>, Option<Range<NonNegative>>)) -> Result<Self> {
        let el = if let Some(data) = value.0 {
            Element::new_data(data)
        } else {
            Element::new_inactive()
        };
        if let Some(range) = value.1 {
            el.with_last(range)
        } else {
            Ok(el)
        }
    }
}

impl<'a> TryFrom<(Option<&'a str>, Option<Range<NonNegative>>)> for Element {
    type Error = Error;

    fn try_from(value: (Option<&'a str>, Option<Range<NonNegative>>)) -> Result<Self> {
        let el = if let Some(data) = value.0 {
            Element::new_data_from_str(data)?
        } else {
            Element::new_inactive()
        };
        if let Some(range) = value.1 {
            el.with_last(range)
        } else {
            Ok(el)
        }
    }
}

impl TryFrom<Range<NonNegative>> for Element {
    type Error = Error;

    fn try_from(value: Range<NonNegative>) -> Result<Self> {
        Element::new_inactive().with_last(value)
    }
}

#[cfg(test)]
mod tests {
    use tydi_common::error::TryResult;

    use bitvec::prelude::*;

    use super::*;

    #[test]
    fn valid_new() -> Result<()> {
        let inactive: Element = Element::new_inactive();
        assert!(!inactive.is_active());
        assert_eq!(inactive.last(), &None);

        let inactive = inactive.with_last(1..2)?;
        assert_eq!(inactive.last(), &Some(1..2));

        let inactive_2 = (1..2).try_result()?;
        assert_eq!(inactive, inactive_2);

        let inactive_3 = ("-", 1..2).try_result()?;
        assert_eq!(inactive, inactive_3);

        let inactive_4 = ("inactive", 1..2).try_result()?;
        assert_eq!(inactive, inactive_4);

        let data: Element = Element::new_data_from_str("101")?;
        assert!(data.is_active());
        assert_eq!(data.last(), &None);

        let data_2 = "101".try_result()?;
        assert_eq!(data, data_2);

        let data = data.with_last(0..2)?;
        assert_eq!(data.last(), &Some(0..2));

        let data_2 = ("101", 0..2).try_result()?;
        assert_eq!(data, data_2);

        let null_data = Element::new_data(ElementType::Null);
        assert!(null_data.is_active());

        let null_data2 = "".try_result()?;
        assert_eq!(null_data, null_data2);

        let null_data3 = "null".try_result()?;
        assert_eq!(null_data, null_data3);

        Ok(())
    }

    #[test]
    fn invalid_new() -> Result<()> {
        let data = Element::new_data(bitvec![0, 1, 0]);

        let data_result = data.with_last(1..2);
        assert_eq!(
            data_result,
            Err(Error::InvalidArgument(
                "Cannot assert this element as being the last of dimensions 1..2. Elements with active data can only transferred as the innermost dimension (0).".to_string()
            ))
        );

        Ok(())
    }
}
