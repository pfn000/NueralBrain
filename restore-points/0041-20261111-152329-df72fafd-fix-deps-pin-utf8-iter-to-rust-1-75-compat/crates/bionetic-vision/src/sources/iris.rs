//! IrisSource — browser/page capture adapter.
//!
//! The local shim intentionally fails at launch until the actual Iris
//! crate and API are provided; it does not claim to capture a browser.

use crate::error::{VisionError, VisionResult};
use crate::frame::{Frame, PixelFormat};
use crate::source::VisualSource;

#[allow(dead_code)]
mod iris_shim {
    /// Local placeholder for an Iris browser handle.
    pub struct Browser;
    /// Local placeholder for a browser page.
    pub struct Page;
    /// Screenshot returned by a future Iris adapter.
    pub struct Screenshot {
        /// Captured width.
        pub width: u32,
        /// Captured height.
        pub height: u32,
        /// Row-major RGBA bytes.
        pub rgba: Vec<u8>,
    }
    impl Browser {
        pub fn launch(_headless: bool) -> Result<Self, String> {
            Err("Iris adapter is a stub; connect the actual Iris crate/API".into())
        }
        pub fn new_page(&mut self) -> Result<Page, String> {
            Err("Iris adapter is not connected".into())
        }
    }
    impl Page {
        pub fn goto(&mut self, _url: &str) -> Result<(), String> {
            Err("Iris adapter is not connected".into())
        }
        pub fn screenshot(&mut self) -> Result<Screenshot, String> {
            Err("Iris adapter is not connected".into())
        }
        pub fn set_viewport(&mut self, _width: u32, _height: u32) -> Result<(), String> {
            Err("Iris adapter is not connected".into())
        }
    }
}
use iris_shim::{Browser, Page};

/// Captures frames from an Iris-controlled page after the shim is replaced.
pub struct IrisSource {
    #[allow(dead_code)]
    browser: Browser,
    page: Page,
    width: u32,
    height: u32,
    url: String,
    name: String,
}

impl IrisSource {
    /// Attempt to launch a headless browser and open a URL.
    pub fn open(url: &str, width: u32, height: u32) -> VisionResult<Self> {
        if width == 0 || height == 0 {
            return Err(VisionError::InvalidDimensions { w: width, h: height });
        }
        let mut browser = Browser::launch(true)
            .map_err(|error| VisionError::SourceInit(format!("iris launch: {error}")))?;
        let mut page = browser.new_page()
            .map_err(|error| VisionError::SourceInit(format!("iris new_page: {error}")))?;
        page.set_viewport(width, height)
            .map_err(|error| VisionError::SourceInit(format!("iris viewport: {error}")))?;
        page.goto(url)
            .map_err(|error| VisionError::SourceInit(format!("iris goto: {error}")))?;
        Ok(Self {
            browser,
            page,
            width,
            height,
            url: url.to_string(),
            name: format!("iris@{url}"),
        })
    }

    /// Navigate to another URL without recreating the browser.
    pub fn navigate(&mut self, url: &str) -> VisionResult<()> {
        self.page.goto(url)
            .map_err(|error| VisionError::Backend(format!("iris navigate: {error}")))?;
        self.url = url.to_string();
        self.name = format!("iris@{url}");
        Ok(())
    }

    /// Return the current page URL.
    pub fn current_url(&self) -> &str { &self.url }
}

impl VisualSource for IrisSource {
    fn capture_frame(&mut self) -> VisionResult<Frame> {
        let shot = self.page.screenshot()
            .map_err(|error| VisionError::CaptureFailed(format!("iris screenshot: {error}")))?;
        let mut frame = Frame::new(shot.width, shot.height, PixelFormat::Rgba8, shot.rgba);
        frame.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        Ok(frame)
    }

    fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
    fn name(&self) -> &str { &self.name }
}
