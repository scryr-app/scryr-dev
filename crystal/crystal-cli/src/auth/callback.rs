//! Local OAuth callback listener.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use url::Url;

/// OAuth callback parameters received by the local redirect listener.
#[derive(Debug)]
pub(super) struct CallbackPayload {
    /// Authorization code returned by the provider.
    pub(super) code: String,
    /// CSRF state returned by the provider.
    pub(super) state: String,
}

/// Wait for one local OAuth callback in a background thread.
pub(super) fn spawn_callback_listener(
    listener: TcpListener,
) -> mpsc::Receiver<Result<CallbackPayload, String>> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = accept_single_callback(&listener);
        let _ = sender.send(result);
    });

    receiver
}

/// Accept and handle one OAuth callback request.
fn accept_single_callback(listener: &TcpListener) -> Result<CallbackPayload, String> {
    let (mut stream, _) = listener
        .accept()
        .map_err(|error| format!("Failed to accept local OAuth callback: {error}"))?;
    let request = read_http_request(&mut stream)?;
    let callback = parse_callback_request(&request)?;
    write_http_response(
        &mut stream,
        "200 OK",
        "Authentication complete. You can close this browser window and return to scryr.",
    )?;
    Ok(callback)
}

/// Read one HTTP request from the callback connection.
fn read_http_request(stream: &mut TcpStream) -> Result<String, String> {
    let mut buffer = [0_u8; 8192];
    let bytes_read = stream
        .read(&mut buffer)
        .map_err(|error| format!("Failed to read local OAuth callback request: {error}"))?;

    if bytes_read == 0 {
        return Err("Received an empty OAuth callback request.".to_string());
    }

    String::from_utf8(buffer[..bytes_read].to_vec())
        .map_err(|error| format!("OAuth callback request was not valid UTF-8: {error}"))
}

/// Parse the callback request line into OAuth callback values.
fn parse_callback_request(request: &str) -> Result<CallbackPayload, String> {
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| "OAuth callback request did not include a request line.".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next();
    let target = parts.next();

    if method != Some("GET") {
        return Err("OAuth callback used an unsupported HTTP method.".to_string());
    }

    let target = target
        .ok_or_else(|| "OAuth callback request did not include a target path.".to_string())?;
    let parsed = Url::parse(&format!("http://localhost{target}"))
        .map_err(|error| format!("Failed to parse OAuth callback URL: {error}"))?;
    let mut code = None;
    let mut state = None;
    let mut error_message = None;
    let mut error_description = None;
    let mut error_uri = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => error_message = Some(value.into_owned()),
            "error_description" => error_description = Some(value.into_owned()),
            "error_uri" => error_uri = Some(value.into_owned()),
            _ => {}
        }
    }

    if let Some(error_message) = error_message {
        let mut message = format!("Clerk OAuth login failed: {error_message}");
        if let Some(error_description) = error_description {
            let _ = write!(message, "; description: {error_description}");
        }
        if let Some(error_uri) = error_uri {
            let _ = write!(message, "; uri: {error_uri}");
        }
        let _ = write!(message, "; callback URL: {parsed}");
        return Err(message);
    }

    Ok(CallbackPayload {
        code: code
            .ok_or_else(|| "OAuth callback did not include an authorization code.".to_string())?,
        state: state.ok_or_else(|| "OAuth callback did not include a state value.".to_string())?,
    })
}

/// Write one plain-text callback response.
fn write_http_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("Failed to write local OAuth callback response: {error}"))
}

#[cfg(test)]
mod tests {
    use super::parse_callback_request;

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    #[test]
    fn parse_callback_request_extracts_code_and_state() -> TestResult {
        let callback = parse_callback_request(
            "GET /oauth/callback?code=abc123&state=xyz HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .map_err(std::io::Error::other)?;

        assert_eq!(callback.code, "abc123");
        assert_eq!(callback.state, "xyz");
        Ok(())
    }
}
