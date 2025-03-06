use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::{Arc, Weak};

use axum::body::Bytes;

struct WeakBytes(Weak<Bytes>);
impl Hash for WeakBytes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(b) = self.0.upgrade() {
            b.hash(state);
        } else {
            Bytes::new().hash(state);
        }
    }
}
impl PartialEq for WeakBytes {
    fn eq(&self, other: &Self) -> bool {
        self.0.upgrade() == other.0.upgrade()
    }
}
impl Eq for WeakBytes {}

#[derive(Default)]
pub struct CharacterPackCache(HashSet<WeakBytes>);
impl CharacterPackCache {
    pub fn size(&mut self) -> usize {
        let mut res = 0;
        self.0.retain(|w| {
            if let Some(b) = w.0.upgrade() {
                res += b.len();
                true
            } else {
                false
            }
        });
        res
    }

    pub fn cache(&mut self, pack: Bytes) -> CharacterPack {
        self.0.retain(|w| w.0.strong_count() > 0);
        let pack = Arc::new(pack);
        self.0.insert(WeakBytes(Arc::downgrade(&pack)));
        eprintln!("cache has {} items", self.0.len());
        CharacterPack(pack)
    }
}

#[derive(Clone)]
pub struct CharacterPack(Arc<Bytes>);
impl AsRef<[u8]> for CharacterPack {
    fn as_ref(&self) -> &[u8] {
        self.0.deref().as_ref()
    }
}
