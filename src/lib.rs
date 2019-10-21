// #[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

#[cfg(feature = "mockito-enabled")]
extern crate mockito;

#[cfg(test)]
extern crate serde_json;

pub mod contracts;

use contracts::v1 as v1;

#[derive(Debug)]
pub enum Message {
    /// Wrapper for V1 style api error response schema
    V1(v1::Message),
    /// Fallback for proxy implementations to return simple String error messages
    Unstructured(String)
}

impl From<v1::Message> for Message {
    fn from(err: v1::Message) -> Message {
        Message::V1(err)
    }
}

/// MessageResult is a type that represents either success (Ok) or failure (Err)
/// where Ok is of type Message and Err is of type TParseError, such as
/// serde_json::error::Error
pub type MessageResult<TMessageSerde> = Result<Message, TMessageSerde>;

#[derive(Debug)]
pub enum Error<TPayloadSerde, TMessageSerde> {
    /// Base URL failed to parse
    UrlParseFailed(url::ParseError),
    #[cfg(feature = "mockito-enabled")]
    /// Tried to replace Url host with mockito but failed
    UrlBaseReplacementError(url::ParseError),
    /// Call to backing service failed
    RequestFailed,
    /// Unable to parse api response to extract payload content
    ReadBodyFailed,
    /// API returned a failure, such as invalid HTTP status code
    ResultFailed {
        message: MessageResult<TMessageSerde>,
    },
    /// Api call succeeded, e.g. with 200 OK, but payload did not parse successfully
    InvalidPayload {
        error: TPayloadSerde,
        payload: String,
        message: MessageResult<TMessageSerde>,
    },
}

impl<TPayloadSerde, TMessageSerde> From<url::ParseError> for Error<TPayloadSerde, TMessageSerde> {
    fn from(err: url::ParseError) -> Error<TPayloadSerde, TMessageSerde> {
        Error::<TPayloadSerde, TMessageSerde>::UrlParseFailed(err)
    }
}



// impl From<Request> for String {
//     fn from(_req: Request) -> url::Url {
//         url::Url::parse("http://www.foo.bar").unwrap()
//     }
// }

// pub trait Request {
//     fn get_url_string<TRequest>(svc: &Service, req: TRequest) -> String;
// }

#[derive(Debug, Default)]
/// Carries contextual data along with Service errors
pub struct ServiceError<TContext, TError> {
    pub context: TContext,
    pub error: TError,
}

impl<TContext, TPayloadSerde, TMessageSerde> AsRef<Error<TPayloadSerde, TMessageSerde>> for ServiceError<TContext, Error<TPayloadSerde, TMessageSerde>> {
    fn as_ref(&self) -> &Error<TPayloadSerde, TMessageSerde> {
        &self.error
    }
}

/// Provides a simple surface for proxying requests back to origin api servers
pub trait Service {
    type TRequest;
    type TContext;
    type TPayloadSerdeError;
    type TMessageSerdeError;

    fn exec<TRequest, TResponse>(&self, req: TRequest) -> Result<TResponse, ServiceError<Self::TContext, Error<Self::TPayloadSerdeError, Self::TMessageSerdeError>>> where
        TRequest: Into<Self::TRequest> + std::fmt::Debug,
        TResponse: serde::de::DeserializeOwned + std::fmt::Debug;
}

#[cfg(feature = "mockito-enabled")]
fn mockito<TPayloadSerde, TMessageSerde>(url_str: url::Url) -> Result<url::Url, Error<TPayloadSerde, TMessageSerde>> {
    let mockito_base = url::Url::parse(&mockito::server_url())
        .map_err(|err| Error::from(err))?;
    replace_host(url_str, mockito_base)
        .map_err(|err| Error::<TPayloadSerde, TMessageSerde>::UrlBaseReplacementError(err))
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
pub fn parse_url<TPayloadSerde, TMessageSerde>(url_str: &str) -> Result<url::Url, Error<TPayloadSerde, TMessageSerde>> {
    #[cfg(not(feature = "mockito-enabled"))]
    {
        url::Url::parse(url_str).map_err(|err| err.into())
    }
    #[cfg(feature = "mockito-enabled")]
    {
        url::Url::parse(url_str).map_err(|err| err.into()).and_then(|url| {
            println!("Replace base: {:?}", url);
            mockito(url)
        })
    }
}

#[cfg(test)]
mod test {
    use super::{ replace_host, Error, ServiceError };

    #[test]
    fn match_service_error_to_error_as_ref() {
        let ctx = ServiceError {
            context: (),
            error: Error::<(), ()>::RequestFailed,
        };
        match ctx.as_ref() {
            Error::RequestFailed => {},
            _ => assert!(false, "boo"),
        }
    }

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