use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::implementation::image::error::ImageError;

pub struct OriginalRead {
    pub file: File,
    reference: String,
    readers: Arc<Mutex<HashMap<String, usize>>>,
}

impl Drop for OriginalRead {
    fn drop(&mut self) {
        // Poisoning keeps protection in place; dropping a lease must never panic.
        let Ok(mut readers) = self.readers.lock() else {
            return;
        };

        let Some(count) = readers.get_mut(&self.reference) else {
            return;
        };

        *count = count.saturating_sub(1);

        if *count == 0 {
            readers.remove(&self.reference);
        }
    }
}

#[derive(Default)]
pub struct OriginalLeaseRegistry {
    readers: Arc<Mutex<HashMap<String, usize>>>,
}

impl OriginalLeaseRegistry {
    /// # Errors
    /// Returns an error if the resource protection state is unavailable.
    pub fn acquire(
        self: &Arc<Self>,
        reference: &str,
        open: impl FnOnce() -> Result<File, ImageError>,
    ) -> Result<OriginalRead, ImageError> {
        let mut readers = self.readers.lock().map_err(|_| ImageError::Unavailable)?;

        let file = open()?;

        let count = readers.entry(reference.to_owned()).or_default();

        *count = count.checked_add(1).ok_or(ImageError::Unavailable)?;

        Ok(OriginalRead {
            file,
            reference: reference.to_owned(),
            readers: Arc::clone(&self.readers),
        })
    }

    /// Executes deletion while preventing any new reader lease from being registered.
    /// # Errors
    /// Active original readers protect their resources until their leases are dropped.
    pub fn remove(
        &self,
        reference: &str,
        remove: impl FnOnce() -> Result<(), ImageError>,
    ) -> Result<(), ImageError> {
        let readers = self.readers.lock().map_err(|_| ImageError::Unavailable)?;

        if readers.contains_key(reference) {
            return Err(ImageError::Unavailable);
        }

        remove()
    }
}
