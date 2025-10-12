use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DurationSeconds(pub chrono::Duration);
impl Serialize for DurationSeconds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let secs = self.0.num_seconds();
        serializer.serialize_i64(secs)
    }
}
impl<'de> serde::Deserialize<'de> for DurationSeconds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let secs = i64::deserialize(deserializer)?;
        Ok(DurationSeconds(chrono::Duration::seconds(secs)))
    }
}
impl From<chrono::Duration> for DurationSeconds {
    fn from(value: chrono::Duration) -> Self {
        Self(value)
    }
}
impl From<DurationSeconds> for chrono::Duration {
    fn from(value: DurationSeconds) -> Self {
        value.0
    }
}
