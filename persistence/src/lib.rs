pub mod errors;
pub mod noop;
pub mod redis;

use errors::Result;

pub trait KV {
    fn set(&self, key: impl AsRef<str>, val: impl AsRef<str>) -> Result<()>;
    fn get(&self, key: impl AsRef<str>) -> Result<Option<String>>;
}
