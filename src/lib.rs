// #[macro_use]
extern crate log;
extern crate serde;
// #[macro_use]
// extern crate serde_derive;
extern crate url;

#[cfg(feature = "mockito-enabled")]
extern crate mockito;

#[cfg(test)]
extern crate serde_json;

use std::fmt;

pub trait Response<TRequest> {
    type TResponse: serde::de::DeserializeOwned + std::fmt::Debug;
    type TError: serde::de::DeserializeOwned + std::fmt::Debug;
    // type TError: std::fmt::Debug;

    fn to_request(&self) -> TRequest;
}

/// ServiceResult is a type that represents either success (Ok) or failure (Err)
/// where Ok is of type Message and Err is of type TParseError, such as
/// serde_json::error::Error
// pub type ServiceResult<TErrorSerde> = Result<Message, TErrorSerde>;

#[derive(Debug)]
/// The set of error types which all service types should be able to represent
pub enum Error {
    /// Base URL failed to parse
    UrlParseFailed(url::ParseError),
    #[cfg(feature = "mockito-enabled")]
    /// Tried to replace Url host with mockito but failed
    UrlBaseReplacementError(url::ParseError),
}

pub trait Endpoint {
    type TResponse: fmt::Debug + serde::de::DeserializeOwned;
    type TError: fmt::Debug + serde::de::DeserializeOwned;
    // type TErrorSerde: fmt::Debug;
}

pub enum ServiceResult<TResponse, TServiceError, TErrorSerde> where
    TResponse: Endpoint,
{
    Ok (TResponse::TResponse),
    Err (TServiceError, TResponse::TError),
    // Fail (TServiceError, Option<TResponse::TErrorSerde>),
    Fail (TServiceError, Option<TErrorSerde>),
}

// impl<TResponse, TServiceError> From<ServiceResult<TResponse, TServiceError>> for Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TResponse::TErrorSerde>>)> where
//     TResponse: Endpoint,
// {
//     fn from(src: ServiceResult<TResponse, TServiceError>) -> Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TResponse::TErrorSerde>>)> {
        
//     }
// }

impl<TResponse, TServiceError, TErrorSerde> Into<Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TErrorSerde>>)>> for ServiceResult<TResponse, TServiceError, TErrorSerde> where
    TResponse: Endpoint,
{
    fn into(self) -> Result<TResponse::TResponse, (TServiceError, Option<Result<TResponse::TError, TErrorSerde>>)> {
        match self {
            ServiceResult::Ok (response) => Ok (response),
            ServiceResult::Err (svc_err, err) => Err ((svc_err, Some(Ok(err)))),
            ServiceResult::Fail (svc_err, opt_serde_err) => {
                Err ((svc_err, opt_serde_err.map(|serde_err| Err(serde_err))))
            }
        }
    }
}



// impl<TResponse, TServiceError> std::ops::Try for ServiceResult<TResponse, TServiceError> where
//     TResponse: Endpoint,
// {
//     type Ok = TResponse;
//     type Error = Option<TResponse::TError>;

//     fn into_result(self) -> Result<Self::Ok, Self::Error> {

//     }

//     fn from_error(src: Self::Error) -> Self {

//     }

//     fn from_ok(src: Self::Ok) -> Self {
//         ServiceResult::Ok(src)
//     }
// }

pub trait Service {
    /// Defines the request types that can be executed by the implementing service.
    /// E.g. in an http api variant this could represent Get, Post, Put, etc.
    type TRequestType;
    type TServiceError;
    type TErrorSerde;

    // fn exec<TRequest>(&self, req: TRequest) -> Result<TRequest::TResponse, (Self::TServiceError, <TRequest as Endpoint>::TError)> where
    fn exec<TRequest>(&self, req: TRequest) -> ServiceResult<TRequest, Self::TServiceError, Self::TErrorSerde> where
    // fn exec<TRequest, TError>(&self, req: TRequest) -> Result<TRequest::TResponse, TError> where
        TRequest: Into<Self::TRequestType> + Endpoint + fmt::Debug;
}

// impl<TError: std::fmt::Debug> From<url::ParseError> for Error<TError> {
//     fn from(err: url::ParseError) -> Error<TError> {
//         Error::<TError>::UrlParseFailed(err)
//     }
// }

// #[derive(Debug, Default)]
// /// Carries contextual data along with Service errors
// pub struct ServiceError<TContext, TError> {
//     pub context: TContext,
//     pub error: TError,
// }

// impl<TContext, TError: std::fmt::Debug> AsRef<Error<TError>> for ServiceError<TContext, Error<TError>> {
//     fn as_ref(&self) -> &Error<TError> {
//         &self.error
//     }
// }

// pub type ServiceResult<TContext, TResponse, TError> = Result<TResponse, ServiceError<TContext, Error<TError>>>;

// /// Provides a simple surface for proxying requests back to origin api servers
// pub trait Service {
//     /// Defines the request types that can be executed by the implementing service.
//     /// E.g. in an http api variant this could represent Get, Post, Put, etc.
//     type TRequest;
//     /// Provides the implementating service the ability to tunnel up specific details
//     /// of execution that aren't easily abstracted.  E.g. in an http api variant it might
//     /// carry reqwest specific Errors and Http Request structures.
//     type TContext;

//     fn exec<TRequest>(&self, req: TRequest) -> Result<TRequest::TResponse, ServiceError<Self::TContext, Error<<TRequest as Response>::TError>>> where
//         TRequest: Into<Self::TRequest> + Response + std::fmt::Debug;
//         // TRequest: Into<Self::TRequest> + Response + std::fmt::Debug;
//         // TResponse: serde::de::DeserializeOwned + std::fmt::Debug;
// }

#[cfg(feature = "mockito-enabled")]
fn mockito(url_str: url::Url) -> Result<url::Url, Error> {
    let mockito_base = url::Url::parse(&mockito::server_url())
        .map_err(Error::UrlParseFailed)?;//|err| Error::from(err))?;
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
    use super::{ replace_host };//, Error, ServiceError };

    // #[test]
    // fn match_service_error_to_error_as_ref() {
    //     let ctx = ServiceError {
    //         context: (),
    //         error: Error::<()>::ServiceError(()),
    //     };
    //     match ctx.as_ref() {
    //         Error::ServiceError(_) => {},
    //         _ => assert!(false, "baz"),
    //     }
    // }

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