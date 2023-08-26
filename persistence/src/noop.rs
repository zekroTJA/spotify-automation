use crate::KV;

pub struct NoOp;

impl KV for NoOp {
    fn set(&self, _: impl AsRef<str>, _: impl AsRef<str>) -> crate::errors::Result<()> {
        Ok(())
    }

    fn get(&self, _: impl AsRef<str>) -> crate::errors::Result<Option<String>> {
        Ok(None)
    }
}
