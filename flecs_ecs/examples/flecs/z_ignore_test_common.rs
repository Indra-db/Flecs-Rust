use std::{
    io::{set_output_capture, Error},
    sync::{Arc, Mutex},
};

pub use capture_stdio::*;

/// Wrap `std::io::set_output_capture`
pub struct OutputCapture {
    local_stream: Arc<Mutex<Vec<u8>>>,
    original: Option<Arc<Mutex<Vec<u8>>>>,
    restored: bool,
}

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
        settings
            ._private_inner_mut()
            .filters((vec![(r"id: (\d+)\s", "[ID] ")]));
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix(name);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(str_output);
        });
    }
}

impl Drop for OutputCapture {
    fn drop(&mut self) {
        if !self.restored {
            self.restore();
        }
    }
}
