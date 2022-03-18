use tydi_common::error::{Error, Result};

pub fn bits_from_str<'a, const BIT_WIDTH: usize>(data: &'a str) -> Result<[bool; BIT_WIDTH]> {
    if data.len() != BIT_WIDTH {
        Err(Error::InvalidArgument(format!(
            "String with length {} does not match bit width {}",
            data.len(),
            BIT_WIDTH
        )))
    } else if data.chars().all(|x| x == '0' || x == '1') {
        let mut data_result = [false; BIT_WIDTH];
        // NOTE: Reversed left being LSB
        for (idx, val) in data.chars().rev().enumerate() {
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
