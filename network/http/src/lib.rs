pub use http::*;
use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum HttpParser<'a, T: Display> {
    Request(&'a Request<T>),
    Response(&'a Response<T>),
}
impl<'a, T: Display> HttpParser<'a, T> {
    pub fn from_request(request: &'a Request<T>) -> Self {
        HttpParser::Request(request)
    }
    pub fn from_response(response: &'a Response<T>) -> Self {
        HttpParser::Response(response)
    }
}
impl<T: Display> HttpParser<'_, T> {
    fn parse_version(&self) -> Option<String> {
        let version = match self {
            HttpParser::Request(request) => {
                request.version()
            }
            HttpParser::Response(response) => {
                response.version()
            }
        };
        match version {
            Version::HTTP_09 => Some(String::from("HTTP/0.9")),
            Version::HTTP_10 => Some(String::from("HTTP/1.0")),
            Version::HTTP_11 => Some(String::from("HTTP/1.1")),
            Version::HTTP_2 => Some(String::from("HTTP/2")),
            Version::HTTP_3 => Some(String::from("HTTP/3")),
            _ => None,
        }
    }
    fn parse_header(&self) -> String {
        let headers = match self {
            HttpParser::Request(request) => {
                request.headers()
            }
            HttpParser::Response(response) => {
                response.headers()
            }
        };
        headers.iter()
            .map(|(key, value)| {
                let value = String::from_utf8_lossy(value.as_bytes());
                format!("{key}:{value}")
            })
            .collect::<Vec<_>>()
            .join("\r\n")
    }
    fn parse_request(&self) -> Option<String> {
        if let HttpParser::Request(request) = self {
            let method = request.method();
            let uri = request.uri();
            let version = self.parse_version()?;
            let header = self.parse_header();
            let body = request.body();

            return Some(format!("{method} {uri} {version}\r\n{header}\r\n{body}"));
        }
        None
    }
    fn parse_response(&self) -> Option<String> {
        if let HttpParser::Response(response) = self {
            let version = self.parse_version()?;
            let status_code = response.status();
            let header = self.parse_header();
            let body = response.body();

            return Some(format!("{version} {status_code}\r\n{header}\r\n{body}"));
        }
        None
    }
}
impl<T: Display> HttpParser<'_, T> {
    pub fn parse(&self) -> Option<String> {
        match self {
            HttpParser::Request(_) => self.parse_request(),
            HttpParser::Response(_) => self.parse_response(),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::HttpParser;
    use http::{Method, Request, Response, StatusCode, Version};

    #[test]
    fn it_works() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("http://localhost/")
            .version(Version::HTTP_11)
            .body("")
            .unwrap();
        let response = Response::builder()
            .status(StatusCode::OK)
            .version(Version::HTTP_11)
            .body("<h1>hello</h1>")
            .unwrap();
        let request = HttpParser::from_request(&request).parse().unwrap();
        let response = HttpParser::from_response(&response).parse().unwrap();
        assert_eq!(request, "GET http://localhost/ HTTP/1.1\r\n\r\n");
        assert_eq!(response, "HTTP/1.1 200 OK\r\n\r\n<h1>hello</h1>");
    }
}