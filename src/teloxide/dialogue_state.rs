use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use async_lock::{Semaphore, SemaphoreGuardArc};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::fmt::Debug;
use teloxide::{
    dispatching::dialogue::{serializer::Json, GetChatId, SqliteStorage},
    prelude::Dialogue,
    types::{ChatId, Update},
};

static EXCLUSIONS: Lazy<DashMap<ChatId, Arc<async_lock::Semaphore>>> = Lazy::new(DashMap::new);

type TeloDialogue<T> = Dialogue<T, SqliteStorage<Json>>;

#[derive(Clone)]
pub struct MutDialogueState<T> {
    chat_id: ChatId,
    telodial: TeloDialogue,
    state: ManuallyDrop<Arc<(SemaphoreGuardArc, Mutex<T>)>>,
}

// unsafe impl Send for MutDialogueState {}
// unsafe impl Sync for MutDialogueState {}

impl<T> MutDialogueState<T> {
    pub async fn new(update: Update, telodial: TeloDialogue) -> Option<Self> {
        let chat_id = update.chat_id()?;

        let excl = EXCLUSIONS
            .entry(chat_id)
            .or_insert_with(|| Arc::new(Semaphore::new(1)))
            // .clone() // doesn't deadlock?
            ;

        // println!("Will wait...");

        let guard = excl.acquire_arc().await;

        // println!("Got...");

        let state = telodial
            .get()
            .await
            .inspect_err(|e| {
                eprintln!("Error getting state: {}", e);
            })
            .ok()?
            .unwrap_or_default();

        Some(Self {
            chat_id,
            telodial,
            state: ManuallyDrop::new(Arc::new((guard, Mutex::new(state)))),
        })
    }

    pub fn chat_id(&self) -> ChatId {
        self.chat_id
    }

    pub fn get(&self) -> impl Deref<Target = T> + Debug + Send + Sync + '_ {
        self.state.1.lock()
    }

    pub fn as_mut(&self) -> impl DerefMut<Target = T> + Send + Sync + '_ {
        self.state.1.lock()
    }
}

impl<T> Drop for MutDialogueState<T> {
    fn drop(&mut self) {
        let arc = unsafe { ManuallyDrop::take(&mut self.state) };
        let Some((guard, mutex)) = Arc::into_inner(arc) else {
            // println!("Not dropping MutDialogueState for chat {}", self.chat_id);
            return;
        };
        let state = mutex.into_inner();
        let telodial = self.telodial.clone();

        tokio::spawn(async move {
            // println!("Saving state...");

            // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let _ = telodial.update(state).await.inspect_err(|e| {
                eprintln!("Error setting state: {}", e);
            });

            drop(guard);
            // println!("Released...");
        });
    }
}
