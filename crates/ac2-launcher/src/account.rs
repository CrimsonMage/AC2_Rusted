//! Signing in at an AC2e account service: `POST {url}/api/login` with the account and password
//! as JSON answers `{account, created, ticket, expires_in, game_host, game_port}`, the ticket
//! single-use and bound to this computer's address. A first sign-in creates the account.

use serde::Deserialize;

/// What the account service answered.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Signed {
    pub account: String,
    #[serde(default)]
    pub created: bool,
    pub ticket: String,
    #[serde(default)]
    pub game_host: Option<String>,
    #[serde(default)]
    pub game_port: Option<u16>,
}

#[derive(Deserialize)]
struct Message {
    message: String,
}

/// Whether `url` may carry a password: HTTPS, or plain HTTP to this computer only.
pub fn safe_for_password(url: &str) -> bool {
    if url.starts_with("https://") {
        return true;
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let host = rest.split(['/', '?']).next().unwrap_or("");
    let host = host.rsplit_once(':').map_or(host, |(h, _)| h);
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

/// Signs in at the account service at `url`. Errors are the service's own words where it gave
/// some; they never carry the password.
pub fn sign_in(url: &str, account: &str, password: &str) -> Result<Signed, String> {
    if !safe_for_password(url) {
        return Err(format!(
            "{url} is not a secure address; the password goes only over HTTPS (or to this computer)"
        ));
    }
    let endpoint = format!("{}/api/login", url.trim_end_matches('/'));
    let body = serde_json::json!({ "account": account, "password": password });
    match ureq::post(&endpoint)
        .timeout(std::time::Duration::from_secs(20))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
    {
        Ok(response) => response
            .into_json::<Signed>()
            .map_err(|e| format!("the account service's answer does not read: {e}")),
        Err(ureq::Error::Status(code, response)) => {
            let said = response.into_json::<Message>().map(|m| m.message);
            Err(said.unwrap_or_else(|_| format!("the account service answered {code}")))
        }
        Err(e) => Err(format!("the account service could not be reached: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_goes_only_over_https_or_to_this_computer() {
        assert!(safe_for_password("https://ac2.example.net"));
        assert!(safe_for_password("http://127.0.0.1:19400"));
        assert!(safe_for_password("http://localhost:19400/"));
        assert!(!safe_for_password("http://ac2.example.net"));
        assert!(!safe_for_password("http://127.0.0.1.evil.example"));
        assert!(!safe_for_password("ftp://127.0.0.1"));
    }

    #[test]
    fn an_insecure_service_is_refused_before_sending() {
        let error = sign_in("http://ac2.example.net", "a", "secret-password").unwrap_err();
        assert!(!error.contains("secret-password"), "{error}");
    }
}
