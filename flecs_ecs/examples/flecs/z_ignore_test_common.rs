use std::sync::{Arc, Mutex};

#[cfg(feature = "flecs_nightly_tests")]
pub use capture_stdio::*;
#[cfg(feature = "flecs_nightly_tests")]
use std::io::{set_output_capture, Error};

/// Wrap `std::io::set_output_capture`
#[cfg(feature = "flecs_nightly_tests")]
pub struct OutputCapture {
    local_stream: Arc<Mutex<Vec<u8>>>,
    original: Option<Arc<Mutex<Vec<u8>>>>,
    restored: bool,
}

#[cfg(feature = "flecs_nightly_tests")]
impl Capture for OutputCapture {
    fn capture() -> Result<Self, Error> {
        let local_stream = Arc::new(Mutex::new(vec![]));
        let original = set_output_capture(Some(local_stream.clone()));
        Ok(Self {
            local_stream,
            original,
            restored: false,
        })
    }

    fn restore(&mut self) {
        assert!(!self.restored, "cannot restore it twice");

        set_output_capture(self.original.clone());
        self.restored = true;
    }
}

#[cfg(feature = "flecs_nightly_tests")]
impl OutputCapture {
    /// Get the captured output
    pub fn output(&self) -> Arc<Mutex<Vec<u8>>> {
        self.local_stream.clone()
    }

    pub fn output_string(&self) -> String {
        String::from_utf8(self.output().lock().unwrap().clone()).unwrap()
    }

    pub fn test(&self, name: String) {
        let str_output = String::from_utf8(self.output().lock().unwrap().clone()).unwrap();
        let mut settings = insta::Settings::clone_current();
        #[allow(clippy::double_parens)]
        settings._private_inner_mut().filters(
            (vec![
                (r"id:\s?(\d+)", "[ID]"),
                (r"#(\d+)\s?", "#[ID] "),
                (
                    r#"Group deleted: \\"([^\\"]+)\\""#,
                    "group deleted: redacted",
                ),
            ]),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix(name);
        settings.set_snapshot_path("z_ignore_test_snapshots");
        settings.bind(|| {
            insta::assert_yaml_snapshot!(str_output);
        });
    }
}

#[cfg(feature = "flecs_nightly_tests")]
impl Drop for OutputCapture {
    fn drop(&mut self) {
        if !self.restored {
            self.restore();
        }
    }
}
