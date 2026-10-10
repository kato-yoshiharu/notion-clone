use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeUtc(pub DateTime<Utc>);

#[cfg(test)]
#[allow(clippy::new_without_default)]
impl DateTimeUtc {
    pub fn new() -> Self {
        Self(Utc::now())
    }
}
