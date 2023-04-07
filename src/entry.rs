use std::path::PathBuf;
use std::sync::{Condvar, Mutex};

fn get_image_bytes(path: &PathBuf) -> Vec<u8> {
    image::open(path).unwrap().as_bytes().to_vec()
}

pub struct Entry {
    pub path: Box<PathBuf>,
    pub ready: Mutex<bool>,
    pub cvar: Condvar,
    pub data: Mutex<Option<Box<Vec<u8>>>>,
}

impl Entry {
    pub fn wait_ready(self: &Entry) -> &Entry {
        let mut opt = self.ready.lock().unwrap();

        while !*opt {
            opt = self.cvar.wait(opt).unwrap();
        }

        self
    }

    pub fn get_data(self: &Entry) -> Option<Vec<u8>> {
        let locked_value = self.data.lock().unwrap();

        match &*locked_value {
            Some(boxed_vec) => Some((**boxed_vec).clone()),
            None => None,
        }
    }

    pub fn load(self: &Entry) {
        let mut file = self.data.lock().unwrap();
        *file = Some(Box::new(get_image_bytes(&self.path)));

        let mut opt = self.ready.lock().unwrap();
        *opt = true;

        self.cvar.notify_all();
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            path: Box::new(PathBuf::default()),
            ready: Mutex::new(false),
            cvar: Condvar::new(),
            data: Mutex::new(None),
        }
    }
}
