use tydi_common::error::{Result, TryOptional, TryResult};

#[derive(Debug, Clone, Hash)]
pub struct SignalList<T> {
    valid: Option<T>,
    ready: Option<T>,
    data: Option<T>,
    last: Option<T>,
    stai: Option<T>,
    endi: Option<T>,
    strb: Option<T>,
    user: Option<T>,
}

impl<T> SignalList<T> {
    pub fn new() -> Self {
        SignalList {
            valid: None,
            ready: None,
            data: None,
            last: None,
            stai: None,
            endi: None,
            strb: None,
            user: None,
        }
    }

    pub fn try_new(
        valid: impl TryOptional<T>,
        ready: impl TryOptional<T>,
        data: impl TryOptional<T>,
        last: impl TryOptional<T>,
        stai: impl TryOptional<T>,
        endi: impl TryOptional<T>,
        strb: impl TryOptional<T>,
        user: impl TryOptional<T>,
    ) -> Result<Self> {
        Ok(SignalList {
            valid: valid.try_optional()?,
            ready: ready.try_optional()?,
            data: data.try_optional()?,
            last: last.try_optional()?,
            stai: stai.try_optional()?,
            endi: endi.try_optional()?,
            strb: strb.try_optional()?,
            user: user.try_optional()?,
        })
    }

    pub fn with_valid(mut self, valid: impl TryResult<T>) -> Result<Self> {
        self.valid = Some(valid.try_result()?);
        Ok(self)
    }

    pub fn with_ready(mut self, ready: impl TryResult<T>) -> Result<Self> {
        self.ready = Some(ready.try_result()?);
        Ok(self)
    }

    pub fn with_data(mut self, data: impl TryResult<T>) -> Result<Self> {
        self.data = Some(data.try_result()?);
        Ok(self)
    }

    pub fn with_last(mut self, last: impl TryResult<T>) -> Result<Self> {
        self.last = Some(last.try_result()?);
        Ok(self)
    }

    pub fn with_stai(mut self, stai: impl TryResult<T>) -> Result<Self> {
        self.stai = Some(stai.try_result()?);
        Ok(self)
    }

    pub fn with_endi(mut self, endi: impl TryResult<T>) -> Result<Self> {
        self.endi = Some(endi.try_result()?);
        Ok(self)
    }

    pub fn with_strb(mut self, strb: impl TryResult<T>) -> Result<Self> {
        self.strb = Some(strb.try_result()?);
        Ok(self)
    }

    pub fn with_user(mut self, user: impl TryResult<T>) -> Result<Self> {
        self.user = Some(user.try_result()?);
        Ok(self)
    }

    pub fn set_valid(&mut self, valid: impl TryOptional<T>) -> Result<()> {
        self.valid = valid.try_optional()?;
        Ok(())
    }

    pub fn set_ready(&mut self, ready: impl TryOptional<T>) -> Result<()> {
        self.ready = ready.try_optional()?;
        Ok(())
    }

    pub fn set_data(&mut self, data: impl TryOptional<T>) -> Result<()> {
        self.data = data.try_optional()?;
        Ok(())
    }

    pub fn set_last(&mut self, last: impl TryOptional<T>) -> Result<()> {
        self.last = last.try_optional()?;
        Ok(())
    }

    pub fn set_stai(&mut self, stai: impl TryOptional<T>) -> Result<()> {
        self.stai = stai.try_optional()?;
        Ok(())
    }

    pub fn set_endi(&mut self, endi: impl TryOptional<T>) -> Result<()> {
        self.endi = endi.try_optional()?;
        Ok(())
    }

    pub fn set_strb(&mut self, strb: impl TryOptional<T>) -> Result<()> {
        self.strb = strb.try_optional()?;
        Ok(())
    }

    pub fn set_user(&mut self, user: impl TryOptional<T>) -> Result<()> {
        self.user = user.try_optional()?;
        Ok(())
    }

    pub fn valid(&self) -> &Option<T> {
        &self.valid
    }

    pub fn ready(&self) -> &Option<T> {
        &self.ready
    }

    pub fn data(&self) -> &Option<T> {
        &self.data
    }

    pub fn last(&self) -> &Option<T> {
        &self.last
    }

    pub fn stai(&self) -> &Option<T> {
        &self.stai
    }

    pub fn endi(&self) -> &Option<T> {
        &self.endi
    }

    pub fn strb(&self) -> &Option<T> {
        &self.strb
    }

    pub fn user(&self) -> &Option<T> {
        &self.user
    }
}

impl<T> IntoIterator for SignalList<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = vec![];
        if let Some(val) = self.valid {
            result.push(val);
        }
        if let Some(val) = self.ready {
            result.push(val);
        }
        if let Some(val) = self.data {
            result.push(val);
        }
        if let Some(val) = self.last {
            result.push(val);
        }
        if let Some(val) = self.stai {
            result.push(val);
        }
        if let Some(val) = self.endi {
            result.push(val);
        }
        if let Some(val) = self.strb {
            result.push(val);
        }
        if let Some(val) = self.user {
            result.push(val);
        }

        result.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SignalList<T> {
    type Item = &'a T;
    type IntoIter = SignalListIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SignalListIterator {
            list: self,
            index: 0,
        }
    }
}

pub struct SignalListIterator<'a, T> {
    list: &'a SignalList<T>,
    index: usize,
}

impl<'a, T> Iterator for SignalListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => match self.list.valid() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            1 => match self.list.ready() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            2 => match self.list.data() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            3 => match self.list.last() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            4 => match self.list.stai() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            5 => match self.list.endi() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            6 => match self.list.strb() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            7 => match self.list.user() {
                Some(val) => val,
                None => {
                    self.index += 1;
                    return self.next();
                }
            },
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}