//! Concrete VisualSource implementations.

#[cfg(feature = "camera")]
pub mod camera;
#[cfg(feature = "camera")]
pub use camera::CameraSource;

#[cfg(feature = "iris")]
pub mod iris;
#[cfg(feature = "iris")]
pub use iris::IrisSource;

#[cfg(feature = "file-source")]
pub mod file;
#[cfg(feature = "file-source")]
pub use file::FileSource;
