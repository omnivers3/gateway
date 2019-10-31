// #[macro_use]
extern crate log;
extern crate serde;
extern crate url;

#[cfg(feature = "mockito-enabled")]
extern crate mockito;

#[cfg(test)]
extern crate serde_json;

use std::fmt;

#[derive(Debug)]
/// The set of error types which all service types should be able to represent
pub enum Error {
    /// Base URL failed to parse
    UrlParseFailed(url::ParseError),
    #[cfg(feature = "mockito-enabled")]
    /// Tried to replace Url host with mockito but failed
    UrlBaseReplacementError(url::ParseError),
}

/// Endpoint associates expected response and error types with the implementing targets
pub trait Endpoint {
    /// The type a service endpoint call should respond with
    type TResponse: fmt::Debug + serde::de::DeserializeOwned;
    /// The error type a service endpoint call will try to deserialize into
    type TError: fmt::Debug + serde::de::DeserializeOwned;
}

/// ServiceResult
pub enum ServiceResult<TResponse, TServiceError, TErrorSerde> where
    TResponse: Endpoint,
{
    /// Service call succeeded and the result was successfully parsed into the expected type
    Ok (TResponse::TResponse),
    /// Service call failed and the returned error message was successfully parsed into the expected type
    Err (TServiceError, TResponse::TError),
    /// Service call failed and was unable to deserialize the returned error context into the expected type
    Fail (TServiceError, Option<TErrorSerde>),
}

impl<TResponse, TServiceError, TErrorSerde> ServiceResult<TResponse, TServiceError, TErrorSerde> where
    TResponse: Endpoint,
{
    pub fn as_result(self) -> Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TErrorSerde>>)> {
        match self {
            ServiceResult::Ok (response) => Ok (response),
            ServiceResult::Err (svc_err, err) => Err ((svc_err, Some(Ok(err)))),
            ServiceResult::Fail (svc_err, opt_serde_err) => {
                Err ((svc_err, opt_serde_err.map(|serde_err| Err(serde_err))))
            }
        }
    }
}

impl<TResponse, TServiceError, TErrorSerde> Into<Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TErrorSerde>>)>> for ServiceResult<TResponse, TServiceError, TErrorSerde> where
    TResponse: Endpoint,
{
    fn into(self) -> Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TErrorSerde>>)> {
        self.as_result()
    }
}

pub trait Service {
    /// Defines the request types that can be executed by the implementing service.
    /// E.g. in an http api variant this could represent Get, Post, Put, etc.
    type TRequestType;
    type TServiceError;
    type TErrorSerde;

    fn exec<TRequest>(&self, req: TRequest) -> ServiceResult<TRequest, Self::TServiceError, Self::TErrorSerde> where
        TRequest: Into<Self::TRequestType> + Endpoint + fmt::Debug;
}

#[cfg(feature = "mockito-enabled")]
fn mockito(url_str: url::Url) -> Result<url::Url, Error> {
    let mockito_base = url::Url::parse(&mockito::server_url())
        .map_err(Error::UrlParseFailed)?;
    replace_host(url_str, mockito_base)
        .map_err(|err| Error::UrlBaseReplacementError(err))
}

/// Swaps host, scheme and port of the dest into the target while preserving the remaining path and query semantics
pub fn replace_host(src: url::Url, dest: url::Url) -> Result<url::Url, url::ParseError> {
    let mut src = src;
    match dest.host() {
        None => {},
        Some (host) => {
            let host = format!("{}", host);
            src
                .set_host(Some(&host))?;
        }
    }
    src.set_scheme(dest.scheme()).unwrap();
    dest
        .port()
        .map(|port| src.set_port(Some(port)));
    Ok(src)
}

/// Wraps a call to Url::parse with mockito override to the base in cfg(test) mode
pub fn parse_url(url_str: &str) -> Result<url::Url, Error> {
    #[cfg(not(feature = "mockito-enabled"))]
    {
        url::Url::parse(url_str).map_err(Error::UrlParseFailed)
    }
    #[cfg(feature = "mockito-enabled")]
    {
        url::Url::parse(url_str).map_err(Error::UrlParseFailed).and_then(|url| {
            println!("Replace base: {:?}", url);
            mockito(url)
        })
    }
}

#[cfg(test)]
mod test {
    use super::{ replace_host };

    #[test]
    fn replace_url_host() {
        let src = url::Url::parse("https://www.baz.com:90/foo?bar=10").unwrap();
        let target = url::Url::parse("https://fiz.net").unwrap();
        let expected = url::Url::parse("https://fiz.net:90/foo?bar=10").unwrap();
        let actual = replace_host(src, target).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_url_port() {
        let src = url::Url::parse("https://www.baz.com:90/foo?bar=10").unwrap();
        let target = url::Url::parse("https://baz.net:9090").unwrap();
        let expected = url::Url::parse("https://baz.net:9090/foo?bar=10").unwrap();
        let actual = replace_host(src, target).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_url_schema() {
        let src = url::Url::parse("https://www.baz.com:90/foo?bar=10").unwrap();
        let target = url::Url::parse("http://baz.net").unwrap();
        let expected = url::Url::parse("http://baz.net:90/foo?bar=10").unwrap();
        let actual = replace_host(src, target).unwrap();
        assert_eq!(expected, actual);
    }
}