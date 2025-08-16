#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignatureVersions {
    V4,
    /// Mostlly deprecated. This library does not support V2.
    #[deprecated(note = "V2 is deprecated and not supported by this library")]
    V2,
}
