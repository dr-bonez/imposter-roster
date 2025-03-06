use std::collections::HashSet;
use std::hash::Hash;
use std::io::{Cursor, Read};
use std::mem::MaybeUninit;
use std::sync::{Arc, Weak};

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::http::{HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use bytes::BytesMut;
use zip::ZipArchive;

use crate::NUM_CHARS;

struct WeakHashable<T>(Weak<T>);
impl<T: Hash + Default> Hash for WeakHashable<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(b) = self.0.upgrade() {
            b.hash(state);
        } else {
            T::default().hash(state);
        }
    }
}
impl<T: PartialEq> PartialEq for WeakHashable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.upgrade() == other.0.upgrade()
    }
}
impl<T: Eq> Eq for WeakHashable<T> {}

#[derive(Default)]
pub struct CharacterCache(HashSet<WeakHashable<Character>>);
impl CharacterCache {
    pub fn size(&mut self) -> usize {
        let mut res = 0;
        self.0.retain(|w| {
            if let Some(b) = w.0.upgrade() {
                res += b.size();
                true
            } else {
                false
            }
        });
        res
    }

    pub fn load(&mut self, pack: Bytes) -> Result<CharacterSet, anyhow::Error> {
        self.0.retain(|w| w.0.strong_count() > 0);
        let mut zip = ZipArchive::new(Cursor::new(pack))?;
        let mut set = [const { MaybeUninit::<Arc<Character>>::uninit() }; NUM_CHARS];
        let mut initialized = 0_usize;
        if let Err(e) = (|| {
            for (idx, char_idx) in
                rand::seq::index::sample_array::<_, NUM_CHARS>(&mut rand::rng(), zip.len())
                    .ok_or_else(|| anyhow!("not enough images in the zip!"))?
                    .into_iter()
                    .enumerate()
            {
                let mut file = zip.by_index(char_idx)?;
                let mut data = BytesMut::zeroed(file.size() as usize);
                file.read_exact(&mut data)?;
                let character = Arc::new(Character {
                    content_type: mime_guess::from_path(file.name())
                        .first()
                        .map(|m| HeaderValue::from_str(&m.to_string()))
                        .transpose()?,
                    data: data.into(),
                });
                self.0.insert(WeakHashable(Arc::downgrade(&character)));
                set[idx].write(character);
                initialized += 1;
            }
            Ok(())
        })() {
            unsafe {
                for i in 0..initialized {
                    set[i].assume_init_drop();
                }
            }
            return Err(e);
        }
        eprintln!("cache has {} items", self.0.len());
        Ok(CharacterSet(unsafe { std::mem::transmute(set) }))
    }
}

pub struct CharacterSet(pub [Arc<Character>; NUM_CHARS]);

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Character {
    content_type: Option<HeaderValue>,
    data: Bytes,
}
impl Character {
    pub fn size(&self) -> usize {
        self.content_type.as_ref().map_or(0, |h| h.len()) + self.data.len()
    }

    pub fn to_response(&self) -> Response<Body> {
        let mut res = StatusCode::OK.into_response();
        *res.body_mut() = Body::from(self.data.clone());
        if let Some(content_type) = self.content_type.clone() {
            res.headers_mut().insert("content-type", content_type);
        }
        res
    }
}
