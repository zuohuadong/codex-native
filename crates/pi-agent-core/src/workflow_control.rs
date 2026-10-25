//! Runtime-only controls. Registration lifetime is owned by the workflow future.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

#[derive(Clone, Default)]
pub struct WorkflowControls(Arc<Mutex<HashMap<String, watch::Sender<bool>>>>);

impl WorkflowControls {
    /// None means this executor no longer owns an active workflow with this ID.
    pub fn stopping(&self, id: &str) -> Option<bool> {
        self.0
            .lock()
            .unwrap()
            .get(id)
            .map(|sender| *sender.borrow())
    }

    pub fn stop(&self, id: &str) -> bool {
        let controls = self.0.lock().unwrap();
        let Some(sender) = controls.get(id) else {
            return false;
        };
        sender.send_if_modified(|stopping| {
            if *stopping {
                return false;
            }
            *stopping = true;
            true
        })
    }

    pub(crate) fn register(&self, id: String) -> (Registration, watch::Receiver<bool>) {
        let (sender, receiver) = watch::channel(false);
        self.0.lock().unwrap().insert(id.clone(), sender);
        (
            Registration {
                id,
                controls: self.clone(),
            },
            receiver,
        )
    }
}

pub(crate) struct Registration {
    id: String,
    controls: WorkflowControls,
}
impl Drop for Registration {
    fn drop(&mut self) {
        self.controls.0.lock().unwrap().remove(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stops_are_scoped_idempotent_and_removed_on_drop() {
        let controls = WorkflowControls::default();
        let (a, rx) = controls.register("a".into());
        let (_b, other) = controls.register("b".into());
        assert!(controls.stop("a"));
        assert!(!controls.stop("a"));
        assert!(*rx.borrow());
        assert!(!*other.borrow());
        drop(a);
        assert_eq!(controls.stopping("a"), None);
        assert!(!controls.stop("a"));
    }
}
