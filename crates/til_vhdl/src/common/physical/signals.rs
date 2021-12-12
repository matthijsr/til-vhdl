use std::convert::TryFrom;

use crate::common::{
    error::{Error, Result},
    integers::{NonNegative, Positive},
    util::log2_ceil,
};

use super::stream::PhysicalStream;

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Valid,
    Ready,
    Data(NonNegative),
    Last(NonNegative),
    StartIndex(IndexSignal),
    EndIndex(IndexSignal),
    Strobe(NonNegative),
    User(NonNegative),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexSignal {
    bit_count: NonNegative,
    element_lanes: Positive,
}

impl IndexSignal {
    pub(crate) fn new(element_lanes: Positive) -> IndexSignal {
        IndexSignal {
            bit_count: log2_ceil(element_lanes),
            element_lanes: element_lanes,
        }
    }

    pub(crate) fn bit_count(&self) -> NonNegative {
        self.bit_count
    }

    pub(crate) fn element_lanes(&self) -> Positive {
        self.element_lanes
    }

    fn format_number(&self, number: NonNegative) -> String {
        format!(
            "{:0width$b}",
            number,
            width = usize::try_from(self.bit_count).unwrap()
        )
    }

    pub(crate) fn max(&self) -> String {
        self.format_number(self.element_lanes.get() - 1)
    }

    pub(crate) fn min(&self) -> String {
        self.format_number(0)
    }
}

impl Signal {
    pub(crate) fn new_data(stream: &PhysicalStream) -> Signal {
        Signal::Data(stream.data_bit_count())
    }

    pub(crate) fn new_last(stream: &PhysicalStream) -> Signal {
        Signal::Last(stream.last_bit_count() * stream.element_lanes().get())
    }

    pub(crate) fn new_stai(stream: &PhysicalStream) -> Signal {
        Signal::StartIndex(IndexSignal::new(stream.element_lanes()))
    }

    pub(crate) fn new_endi(stream: &PhysicalStream) -> Signal {
        Signal::EndIndex(IndexSignal::new(stream.element_lanes()))
    }

    pub(crate) fn new_strb(stream: &PhysicalStream) -> Signal {
        Signal::Strobe(stream.element_lanes().get())
    }

    pub(crate) fn new_user(stream: &PhysicalStream) -> Signal {
        Signal::User(stream.user_bit_count())
    }

    pub(crate) fn default(&self) -> String {
        match self {
            Signal::Valid | Signal::Ready => "1".to_string(),
            Signal::Data(bit_count)
            | Signal::User(bit_count)
            | Signal::Last(bit_count)
            | Signal::Strobe(bit_count) => {
                format!("{:0<1$}", "", usize::try_from(bit_count.clone()).unwrap())
            }
            Signal::StartIndex(index) => index.min(),
            Signal::EndIndex(index) => index.max(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_fmt() {
        let isig = IndexSignal::new(Positive::new(11).unwrap());
        assert_eq!("0000", isig.min());
        assert_eq!("1010", isig.max());
    }
}
