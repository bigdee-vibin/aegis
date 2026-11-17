//! Minimal HTTP/1.1 client for tests. One request per connection, no TLS, no
//! chunked responses (the mock servers always send `Content-Length`).

use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A parsed HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Status code.
    pub status: u16,
    /// Header name/value pairs, names lower-cased.
    pub headers: Vec<(String, String)>,
    /// Body bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Body as lossy UTF-8.
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

fn bad(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

/// Send one request and read the whole response.
///
/// # Errors
/// Returns an I/O error on connection failure, or `InvalidData` for a response
/// that cannot be parsed or uses chunked encoding.
pub async fn request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> io::Result<HttpResponse> {
    let mut stream = TcpStream::connect(addr).await?;
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body).await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    parse(&raw)
}

/// POST a JSON body.
///
/// # Errors
/// See [`request`].
pub async fn post_json(
    addr: SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> io::Result<HttpResponse> {
    let mut all = vec![("Content-Type", "application/json")];
    all.extend_from_slice(headers);
    request(addr, "POST", path, &all, body).await
}

fn parse(raw: &[u8]) -> io::Result<HttpResponse> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| bad("no header terminator"))?;
    let head = std::str::from_utf8(&raw[..split]).map_err(|_| bad("non-utf8 headers"))?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().ok_or_else(|| bad("empty response"))?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| bad("bad status line"))?;
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    if headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v.contains("chunked"))
    {
        return Err(bad("chunked responses are not supported by this client"));
    }
    Ok(HttpResponse {
        status,
        headers,
        body: raw[split + 4..].to_vec(),
    })
}
