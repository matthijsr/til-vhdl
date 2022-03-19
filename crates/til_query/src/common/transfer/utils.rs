use tydi_common::error::{Error, Result};

pub fn bits_from_str<'a>(data: &'a str) -> Result<Vec<bool>> {
    if data.chars().all(|x| x == '0' || x == '1') {
        Ok(data.chars().map(|x| x == '1').collect())
    } else {
        Err(Error::InvalidArgument(
            "String must consist of '0' and '1' only".to_string(),
        ))
    }
}
