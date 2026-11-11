use std::sync::Arc;

use crate::effector::Effector;
use crate::intent::Intent;

#[derive(Default, Clone)]
pub struct EffectorRegistry {
    items: Vec<Arc<dyn Effector>>,
}

impl EffectorRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn register<E: Effector + 'static>(&mut self, effector: E) {
        self.items.push(Arc::new(effector));
    }

    pub fn register_arc(&mut self, effector: Arc<dyn Effector>) {
        self.items.push(effector);
    }

    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }

    pub fn find(&self, intent: &Intent) -> Option<Arc<dyn Effector>> {
        self.items.iter().find(|effector| effector.handles(intent)).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Effector>> {
        self.items.iter()
    }
}
