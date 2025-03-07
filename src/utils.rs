use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};
use std::time::Duration;

use serde::de::Visitor;
use serde::Deserializer;
use tokio::task::{JoinError, JoinHandle};

#[derive(Debug, Default)]
pub struct SyncMutex<T>(std::sync::Mutex<T>);
impl<T> SyncMutex<T> {
    pub fn new(t: T) -> Self {
        Self(std::sync::Mutex::new(t))
    }
    pub fn mutate<F: FnOnce(&mut T) -> U, U>(&self, f: F) -> U {
        f(&mut *self.0.lock().unwrap())
    }
    pub fn peek<F: FnOnce(&T) -> U, U>(&self, f: F) -> U {
        f(&*self.0.lock().unwrap())
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct NonDetachingJoinHandle<T>(#[pin] JoinHandle<T>);
impl<T> NonDetachingJoinHandle<T> {
    pub async fn wait_for_abort(self) -> Result<T, JoinError> {
        self.abort();
        self.await
    }
}
impl<T> From<JoinHandle<T>> for NonDetachingJoinHandle<T> {
    fn from(t: JoinHandle<T>) -> Self {
        NonDetachingJoinHandle(t)
    }
}

impl<T> Deref for NonDetachingJoinHandle<T> {
    type Target = JoinHandle<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for NonDetachingJoinHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[pin_project::pinned_drop]
impl<T> PinnedDrop for NonDetachingJoinHandle<T> {
    fn drop(self: std::pin::Pin<&mut Self>) {
        let this = self.project();
        this.0.into_ref().get_ref().abort()
    }
}
impl<T> Future for NonDetachingJoinHandle<T> {
    type Output = Result<T, JoinError>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.0.poll(cx)
    }
}

pub struct TimedResource<T: 'static + Send + Sync> {
    handle: NonDetachingJoinHandle<()>,
    resource: Weak<T>,
}
impl<T: 'static + Send + Sync> TimedResource<T> {
    pub fn new(resource: T, timer: Duration) -> Self {
        let resource = Arc::new(resource);
        let weak = Arc::downgrade(&resource);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(timer).await;
            drop(resource);
        });
        Self {
            handle: handle.into(),
            resource: weak,
        }
    }

    pub fn get(&self) -> Option<Arc<T>> {
        self.resource.upgrade()
    }

    pub fn is_timed_out(&self) -> bool {
        self.handle.is_finished()
    }
}

pub fn deserialize_bigint<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct MyVisitor;

    impl<'de> Visitor<'de> for MyVisitor {
        type Value = u64;

        fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt.write_str("integer or string")
        }

        fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(val)
        }

        fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match val.parse::<u64>() {
                Ok(val) => self.visit_u64(val),
                Err(_) => Err(E::custom("failed to parse integer")),
            }
        }
    }

    deserializer.deserialize_any(MyVisitor)
}
