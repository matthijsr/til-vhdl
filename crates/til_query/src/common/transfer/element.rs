use std::ops::Range;

use tydi_common::{
    error::{Error, Result},
    numbers::NonNegative,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element<const ELEMENT_SIZE: usize, const MAX_DIMENSION: NonNegative> {
    /// The data transfered on the Element.
    ///
    /// If None, the data on this Element is considered inactive.
    data: Option<[bool; ELEMENT_SIZE]>,
    /// Indicates whether this is the last element for a dimension or set of
    /// dimensions.
    ///
    /// If None, this is not the last element of any dimension.
    last: Option<Range<NonNegative>>,
}

impl<const ELEMENT_SIZE: usize, const MAX_DIMENSION: NonNegative>
    Element<ELEMENT_SIZE, MAX_DIMENSION>
{
    pub fn new(
        data: Option<[bool; ELEMENT_SIZE]>,
        last: Option<Range<NonNegative>>,
    ) -> Result<Self> {
        let result = Self { data, last: None };
        if let Some(last) = last {
            result.with_last(last)
        } else {
            Ok(result)
        }
    }

    fn data_from_str(data: &str) -> Result<[bool; ELEMENT_SIZE]> {
        if data.len() != ELEMENT_SIZE {
            Err(Error::InvalidArgument(format!(
                "String with length {} exceeds element size {}",
                data.len(),
                ELEMENT_SIZE
            )))
        } else if data.chars().all(|x| x == '0' || x == '1') {
            let mut data_result = [false; ELEMENT_SIZE];
            // NOTE: Reversed left being LSB
            for (idx, val) in data.char_indices().rev() {
                if val == '1' {
                    data_result[idx] = true;
                }
            }
            Ok(data_result)
        } else {
            Err(Error::InvalidArgument(
                "String must consist of '0' and '1' only".to_string(),
            ))
        }
    }

    pub fn new_data_from_str(data: &str) -> Result<Self> {
        Ok(Self {
            data: Some(Self::data_from_str(data)?),
            last: None,
        })
    }

    pub fn new_data(data: [bool; ELEMENT_SIZE]) -> Self {
        Self {
            data: Some(data),
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
        if last.end > MAX_DIMENSION {
            Err(Error::InvalidArgument(format!(
                "{} exceeds the maximum dimension of {} for this element.",
                last.end, MAX_DIMENSION
            )))
        } else {
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
    }

    pub fn data(&self) -> &Option<[bool; ELEMENT_SIZE]> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_new() -> Result<()> {
        let inactive: Element<3, 2> = Element::new_inactive();
        assert!(!inactive.is_active());
        assert_eq!(inactive.last(), &None);

        let inactive = inactive.with_last(1..2)?;
        assert_eq!(inactive.last(), &Some(1..2));

        let data: Element<3, 2> = Element::new_data_from_str("101")?;
        assert!(data.is_active());
        assert_eq!(data.last(), &None);

        let data = data.with_last(0..2)?;
        assert_eq!(data.last(), &Some(0..2));

        Ok(())
    }

    #[test]
    fn invalid_new() -> Result<()> {
        let inactive: Element<3, 2> = Element::new_inactive();

        let inactive_result = inactive.with_last(1..3);
        assert_eq!(
            inactive_result,
            Err(Error::InvalidArgument(
                "3 exceeds the maximum dimension of 2 for this element.".to_string()
            ))
        );

        let data: Element<3, 2> = Element::new_data([false, true, false]);

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
