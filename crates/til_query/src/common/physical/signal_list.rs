use tydi_common::{
    error::{Result, TryOptional, TryResult},
    name::Name,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// Use a function to convert the SignalList's fields from `T` to `R`
    pub fn map<F, R>(self, mut f: F) -> SignalList<R>
    where
        F: FnMut(T) -> R,
    {
        SignalList {
            valid: self.valid.map(|x| f(x)),
            ready: self.ready.map(|x| f(x)),
            data: self.data.map(|x| f(x)),
            last: self.last.map(|x| f(x)),
            stai: self.stai.map(|x| f(x)),
            endi: self.endi.map(|x| f(x)),
            strb: self.strb.map(|x| f(x)),
            user: self.user.map(|x| f(x)),
        }
    }

    /// Use a function to convert the SignalList's fields from `T` to `R`, inserts the canonical name of each signal into the function
    pub fn map_named<F, R>(self, mut f: F) -> SignalList<R>
    where
        F: FnMut(Name, T) -> R,
    {
        SignalList {
            valid: self.valid.map(|x| f(Name::try_new("valid").unwrap(), x)),
            ready: self.ready.map(|x| f(Name::try_new("ready").unwrap(), x)),
            data: self.data.map(|x| f(Name::try_new("data").unwrap(), x)),
            last: self.last.map(|x| f(Name::try_new("last").unwrap(), x)),
            stai: self.stai.map(|x| f(Name::try_new("stai").unwrap(), x)),
            endi: self.endi.map(|x| f(Name::try_new("endi").unwrap(), x)),
            strb: self.strb.map(|x| f(Name::try_new("strb").unwrap(), x)),
            user: self.user.map(|x| f(Name::try_new("user").unwrap(), x)),
        }
    }

    /// Apply a mutating function to the SignalList's fields
    pub fn apply<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut(&'a mut T),
    {
        self.valid.as_mut().map(|x| f(x));
        self.ready.as_mut().map(|x| f(x));
        self.data.as_mut().map(|x| f(x));
        self.last.as_mut().map(|x| f(x));
        self.stai.as_mut().map(|x| f(x));
        self.endi.as_mut().map(|x| f(x));
        self.strb.as_mut().map(|x| f(x));
        self.user.as_mut().map(|x| f(x));
    }

    /// Apply a mutating function to the SignalList's fields, inserts the canonical name of each signal into the function
    pub fn apply_named<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut(Name, &'a T),
    {
        self.valid
            .as_ref()
            .map(|x| f(Name::try_new("valid").unwrap(), x));
        self.ready
            .as_ref()
            .map(|x| f(Name::try_new("ready").unwrap(), x));
        self.data
            .as_ref()
            .map(|x| f(Name::try_new("data").unwrap(), x));
        self.last
            .as_ref()
            .map(|x| f(Name::try_new("last").unwrap(), x));
        self.stai
            .as_ref()
            .map(|x| f(Name::try_new("stai").unwrap(), x));
        self.endi
            .as_ref()
            .map(|x| f(Name::try_new("endi").unwrap(), x));
        self.strb
            .as_ref()
            .map(|x| f(Name::try_new("strb").unwrap(), x));
        self.user
            .as_ref()
            .map(|x| f(Name::try_new("user").unwrap(), x));
    }

    /// Modify the first `Some` signal
    ///
    /// Primarily used for setting documentation
    pub fn apply_first<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut(&'a mut T),
    {
        if let Some(valid) = self.valid.as_mut() {
            f(valid);
            return;
        } else if let Some(ready) = self.ready.as_mut() {
            f(ready);
            return;
        } else if let Some(data) = self.data.as_mut() {
            f(data);
            return;
        } else if let Some(last) = self.last.as_mut() {
            f(last);
            return;
        } else if let Some(stai) = self.stai.as_mut() {
            f(stai);
            return;
        } else if let Some(endi) = self.endi.as_mut() {
            f(endi);
            return;
        } else if let Some(strb) = self.strb.as_mut() {
            f(strb);
            return;
        } else if let Some(user) = self.user.as_mut() {
            f(user);
            return;
        }
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
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = vec![];
        if let Some(val) = self.valid() {
            result.push(val);
        }
        if let Some(val) = self.ready() {
            result.push(val);
        }
        if let Some(val) = self.data() {
            result.push(val);
        }
        if let Some(val) = self.last() {
            result.push(val);
        }
        if let Some(val) = self.stai() {
            result.push(val);
        }
        if let Some(val) = self.endi() {
            result.push(val);
        }
        if let Some(val) = self.strb() {
            result.push(val);
        }
        if let Some(val) = self.user() {
            result.push(val);
        }

        result.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() -> Result<()> {
        let empty: SignalList<i32> = SignalList::new();

        let empty_iter = (&empty).into_iter();
        assert_eq!(empty_iter.len(), 0);
        let empty_iter = empty.into_iter();
        assert_eq!(empty_iter.len(), 0);

        let full: SignalList<i32> = SignalList::new()
            .with_valid(0)?
            .with_ready(0)?
            .with_data(0)?
            .with_last(0)?
            .with_stai(0)?
            .with_endi(0)?
            .with_strb(0)?
            .with_user(0)?;

        let mut one_removed = full.clone();
        one_removed.set_user(None)?;

        let full_iter = (&full).into_iter();
        assert_eq!(full_iter.len(), 8);
        let full_iter = full.into_iter();
        assert_eq!(full_iter.len(), 8);

        let one_removed_iter = (&one_removed).into_iter();
        assert_eq!(one_removed_iter.len(), 7);
        let one_removed_iter = one_removed.into_iter();
        assert_eq!(one_removed_iter.len(), 7);

        Ok(())
    }

    #[test]
    fn test_map() -> Result<()> {
        let full: SignalList<i32> = SignalList::new()
            .with_valid(0)?
            .with_ready(1)?
            .with_data(2)?
            .with_last(3)?
            .with_stai(4)?
            .with_endi(5)?
            .with_strb(6)?
            .with_user(7)?;
        let mut one_removed = full.clone();
        one_removed.set_ready(None)?;

        let mapped = full.map(|x| x.to_string());

        assert_eq!(mapped.valid, Some("0".to_string()));
        assert_eq!(mapped.ready, Some("1".to_string()));
        assert_eq!(mapped.data, Some("2".to_string()));
        assert_eq!(mapped.last, Some("3".to_string()));
        assert_eq!(mapped.stai, Some("4".to_string()));
        assert_eq!(mapped.endi, Some("5".to_string()));
        assert_eq!(mapped.strb, Some("6".to_string()));
        assert_eq!(mapped.user, Some("7".to_string()));

        let mapped = one_removed.map(|x| x.to_string());

        assert_eq!(mapped.valid, Some("0".to_string()));
        assert_eq!(mapped.ready, None);
        assert_eq!(mapped.data, Some("2".to_string()));
        assert_eq!(mapped.last, Some("3".to_string()));
        assert_eq!(mapped.stai, Some("4".to_string()));
        assert_eq!(mapped.endi, Some("5".to_string()));
        assert_eq!(mapped.strb, Some("6".to_string()));
        assert_eq!(mapped.user, Some("7".to_string()));

        Ok(())
    }

    #[test]
    fn test_map_named() -> Result<()> {
        let full: SignalList<i32> = SignalList::new()
            .with_valid(0)?
            .with_ready(1)?
            .with_data(2)?
            .with_last(3)?
            .with_stai(4)?
            .with_endi(5)?
            .with_strb(6)?
            .with_user(7)?;

        let mapped = full.map_named(|n, x| format!("{} {}", n, x.to_string()));

        assert_eq!(mapped.valid, Some("valid 0".to_string()));
        assert_eq!(mapped.ready, Some("ready 1".to_string()));
        assert_eq!(mapped.data, Some("data 2".to_string()));
        assert_eq!(mapped.last, Some("last 3".to_string()));
        assert_eq!(mapped.stai, Some("stai 4".to_string()));
        assert_eq!(mapped.endi, Some("endi 5".to_string()));
        assert_eq!(mapped.strb, Some("strb 6".to_string()));
        assert_eq!(mapped.user, Some("user 7".to_string()));

        Ok(())
    }
}
