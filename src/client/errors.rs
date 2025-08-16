use std::{error::Error, fmt::Display};
#[derive(Debug)]
pub enum HttpResponseError {
    Response(reqwest::Response),
    ReqwestError(reqwest::Error),
}
impl From<reqwest::Response> for HttpResponseError {
    fn from(response: reqwest::Response) -> Self {
        HttpResponseError::Response(response)
    }
}
impl From<reqwest::Error> for HttpResponseError {
    fn from(error: reqwest::Error) -> Self {
        HttpResponseError::ReqwestError(error)
    }
}
impl Display for HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpResponseError::Response(response) => {
                let status_code = response.status();
                write!(f, "HTTP Response Error: Status Code: {}", status_code)?;
            }
            HttpResponseError::ReqwestError(error) => write!(f, "Reqwest Error: {}", error)?,
        }
        Ok(())
    }
}
impl Error for HttpResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HttpResponseError::Response(_) => None,
            HttpResponseError::ReqwestError(error) => Some(error),
        }
    }
}
