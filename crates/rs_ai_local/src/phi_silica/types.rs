/// Availability state of Phi Silica on this Windows device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhiSilicaAvailability {
    /// Phi Silica is available and ready.
    Available,
    /// Phi Silica is not available on this device.
    Unavailable,
    /// Windows version does not support Phi Silica.
    WindowsVersionTooOld,
    /// NPU hardware was not detected.
    NpuNotDetected,
}
