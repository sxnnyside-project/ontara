//! The one place the application talks to the internet, and only when the author asked a
//! configured provider something. Everything above it is tested with a fake.

#[cfg(test)]
use std::sync::Mutex;
use std::time::Duration;

use serde_json::Value;

/// How long a provider may take before the call is given up.
const TIMEOUT: Duration = Duration::from_secs(120);

/// Sends one JSON request and returns the status and the JSON answer.
pub trait Transport: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &Value,
    ) -> Result<(u16, Value), String>;
}

pub struct HttpTransport;

impl Transport for HttpTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &Value,
    ) -> Result<(u16, Value), String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .http_status_as_error(false)
            .build()
            .into();
        let mut request = agent.post(url);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        // The error text of the library may quote the request; it never carries the key, which
        // travels only in a header, but it is reduced to a short reason all the same.
        let mut response = request
            .send_json(body)
            .map_err(|error| format!("the provider could not be reached: {error}"))?;
        let status = response.status().as_u16();
        let value: Value = response.body_mut().read_json().unwrap_or(Value::Null);
        Ok((status, value))
    }
}

/// A recorded request, for tests.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct Sent {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Value,
}

/// Answers with a fixed reply and remembers what it was asked.
#[cfg(test)]
pub struct FakeTransport {
    reply: Mutex<(u16, Value)>,
    pub sent: Mutex<Vec<Sent>>,
}

#[cfg(test)]
impl FakeTransport {
    #[must_use]
    pub fn replying(status: u16, body: Value) -> Self {
        Self {
            reply: Mutex::new((status, body)),
            sent: Mutex::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn requests(&self) -> usize {
        self.sent.lock().map_or(0, |sent| sent.len())
    }
}

#[cfg(test)]
impl Transport for FakeTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &Value,
    ) -> Result<(u16, Value), String> {
        if let Ok(mut sent) = self.sent.lock() {
            sent.push(Sent {
                url: url.to_owned(),
                headers: headers.to_vec(),
                body: body.clone(),
            });
        }
        self.reply
            .lock()
            .map(|reply| reply.clone())
            .map_err(|e| e.to_string())
    }
}
