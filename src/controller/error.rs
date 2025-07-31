use std::fmt;

#[derive(Debug)]
pub enum GoogleControllerError {
    JwtKeyFormationError,
    HttpRequestError(String, u16, String)
}

impl fmt::Display for GoogleControllerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GoogleControllerError::JwtKeyFormationError => write!(f, "Could not build the JWT Key"),
            GoogleControllerError::HttpRequestError(endpoint, response_code, error_string) => write!(f, "The HTTP request to the endpoint {} returned\nStatus Code: {}\nError String: {}", endpoint, response_code, error_string)
            /*GoogleControllerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            GoogleControllerError::NumberOutOfRange(num) => write!(f, "Number {} is out of range", num),
            GoogleControllerError::Io(err) => write!(f, "IO error: {}", err),
            GoogleControllerError::ParseInt(err) => write!(f, "Parse int error: {}", err),
            */
        }
    }
}

impl From<jsonwebtoken::errors::Error> for GoogleControllerError {
    fn from(_err: jsonwebtoken::errors::Error) -> Self {
        GoogleControllerError::JwtKeyFormationError
    }
}

impl From<reqwest::Error> for GoogleControllerError {
    fn from(err: reqwest::Error) -> Self {
        let url = match err.url() {
            Some(u) => String::from(u.as_str()),
            None => String::from("<UNKNOWN URL>")
        };

        let status_code = match err.status() {
            Some(s) => s.as_u16(),
            None => 0
        };

        let body =  err.to_string();

        GoogleControllerError::HttpRequestError(url, status_code, body)
    }
}