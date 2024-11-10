use crate::error::Error;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug, Clone)]
pub struct HttpResponse {
    version: String,
    status_code: u32,
    reason: String,
    headers: Vec<Header>,
    body: String,
}

impl HttpResponse {
    pub fn new(raw_response: String) -> Result<Self, Error> {
        let preproccessed_response = raw_response.trim_start().replace("\n\r", "\n");
        let (status_line, remaining) = match preproccessed_response.split_once("\n") {
            Some((status_line, remaining)) => (status_line, remaining),
            None => {
                return Err(Error::Network(format!(
                    "invalid http response: {}",
                    preproccessed_response
                )))
            }
        };

        let (headers, body) = match remaining.split_once("\n\n") {
            Some((h, b)) => {
                let mut headers = Vec::new();
                for header in h.split("\n") {
                    let splitted_headers: Vec<&str> = header.splitn(2, ":").collect();
                    headers.push(Header::new(
                        splitted_headers[0].trim().to_string(),
                        splitted_headers[1].trim().to_string(),
                    ));
                }
                (headers, b)
            }
            None => (Vec::new(), remaining),
        };

        let statuses: Vec<&str> = status_line.split(" ").collect();
        Ok(Self {
            version: statuses[0].to_string(),
            status_code: statuses[1].parse().unwrap(),
            reason: statuses[2].to_string(),
            headers,
            body: body.to_string(),
        })
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn reason(&self) -> String {
        self.reason.clone()
    }

    pub fn headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn header_value(&self, name: &str) -> Result<String, String> {
        for h in &self.headers {
            if h.name == name {
                return Ok(h.value.clone());
            }
        }
        Err(format!("failed to find {} in headers", name))
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line_only() {
        let raw = "HTTP/1.1 200 OK\n\n".to_string();
        let res = HttpResponse::new(raw).expect("failed to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");
    }

    #[test]
    fn test_one_header() {
        let raw = "HTTP/1.1 200 OK\nContent-Type: text/html\n\n".to_string();
        let res = HttpResponse::new(raw).expect("failed to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");
        assert_eq!(
            res.header_value("Content-Type"),
            Ok("text/html".to_string())
        );
    }

    #[test]
    fn test_two_headers_with_white_space() {
        let raw = "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: 123\n\n".to_string();
        let res = HttpResponse::new(raw).expect("failed to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");
        assert_eq!(
            res.header_value("Content-Type"),
            Ok("text/html".to_string())
        );
        assert_eq!(res.header_value("Content-Length"), Ok("123".to_string()));
    }

    #[test]
    fn test_invalid() {
        let raw = "HTTP/1.1 200 OK".to_string();
        let res = HttpResponse::new(raw);
        assert!(res.is_err());
    }
}
