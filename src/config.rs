#[derive(Clone, Debug)]
pub struct OtlpConfig {
    pub host: String,
    pub port: u16,
    pub org: String,
    pub auth_header: String,
}

impl OtlpConfig {
    /// Constructs the base OpenObserve OTLP URL
    pub fn base_url(&self) -> String {
        format!("http://{}:{}/api/{}", self.host, self.port, self.org)
    }
}
