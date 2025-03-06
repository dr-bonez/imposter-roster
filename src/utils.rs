use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};
use std::time::Duration;

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
