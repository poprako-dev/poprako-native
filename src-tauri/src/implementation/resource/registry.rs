use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

use crate::implementation::resource::cache::DisplayRequest;
use crate::result::{AppError, AppResult};

#[derive(Clone)]
pub struct ImageGrant {
    pub comic_id: Uuid,
    pub page_id: Uuid,
    pub reference: String,
    pub request: DisplayRequest,
}

#[derive(Default)]
pub struct ResourceRegistry {
    grants: Mutex<HashMap<String, ImageGrant>>,
}

impl ResourceRegistry {
    /// # Errors
    /// Rejects exhausted consumer leases instead of growing without a bound.
    pub fn acquire(&self, grant: ImageGrant) -> AppResult<String> {
        let mut grants = self.grants.lock().map_err(|_| AppError::RecoveryRequired)?;

        if grants.len() >= 512 {
            return Err(AppError::Busy);
        }

        let handle = Uuid::new_v4().to_string();

        grants.insert(handle.clone(), grant);

        Ok(handle)
    }

    /// # Errors
    /// Only opaque leases allocated by this process can be resolved.
    pub fn get(&self, handle: &str) -> AppResult<ImageGrant> {
        self.grants
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?
            .get(handle)
            .cloned()
            .ok_or(AppError::NotFound)
    }

    /// # Errors
    /// Rejects oversized release requests.
    pub fn release(&self, handles: &[String]) -> AppResult<()> {
        if handles.len() > 512 {
            return Err(AppError::InvalidInput);
        }

        let mut grants = self.grants.lock().map_err(|_| AppError::RecoveryRequired)?;

        for handle in handles {
            grants.remove(handle);
        }

        Ok(())
    }

    /// # Errors
    /// A poisoned registry cannot prove that an image is unused.
    pub fn protects(&self, reference: &str) -> AppResult<bool> {
        Ok(self
            .grants
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?
            .values()
            .any(|grant| grant.reference == reference))
    }
}
