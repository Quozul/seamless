use std::path::PathBuf;
use std::sync::{Condvar, Mutex};

pub(crate) fn get_image_bytes(path: PathBuf) -> Vec<u8> {
    image::open(path).unwrap().as_bytes().to_vec()
}

pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) ready: Mutex<bool>,
    pub(crate) cvar: Condvar,
    pub(crate) data: Mutex<Option<Vec<u8>>>,
}

impl Entry {
    pub(crate) fn wait_ready(self: &Entry) -> &Entry {
        let mut opt = self.ready.lock().unwrap();

        while !*opt {
            opt = self.cvar.wait(opt).unwrap();
        }

        self
    }

    pub(crate) fn get_data(self: &Entry) -> Option<Vec<u8>> {
        self.data.lock().ok().and_then(|x| x.clone())
    }

    pub(crate) fn load(self: &Entry) {
        let mut file = self.data.lock().unwrap();
        *file = Some(get_image_bytes(self.path.clone()));

        let mut opt = self.ready.lock().unwrap();
        *opt = true;

        self.cvar.notify_all();
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            path: PathBuf::default(),
            ready: Mutex::new(false),
            cvar: Condvar::new(),
            data: Mutex::new(None),
        }
    }
}
