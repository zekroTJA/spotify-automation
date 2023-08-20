use std::{collections::HashMap, error::Error, ops::Deref, str::FromStr};

use url::Url;
use vercel_runtime::Request;

pub fn parse_url(req: &Request) -> Result<Url, url::ParseError> {
    Url::parse(&req.uri().to_string())
}

pub fn get_query_param(req: &Request, key: &str) -> Result<Option<String>, url::ParseError> {
    let url = parse_url(req)?;
    let query_map: HashMap<_, _> = url.query_pairs().collect();
    let v = query_map.get(key).map(|v| v.deref().to_owned());
    Ok(v)
}

pub fn get_query_param_parsed<T, E>(req: &Request, key: &str) -> Result<Option<T>, Box<dyn Error>>
where
    E: Error + 'static,
    T: FromStr<Err = E>,
{
    let v = get_query_param(req, key)?;
    let v = v.map(|v| v.parse()).transpose()?;
    Ok(v)
}
