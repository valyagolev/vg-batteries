use std::{mem::ManuallyDrop, sync::Arc, time::Duration};

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};

use dioxus::prelude::Scope;

use futures::Future;
use tokio::sync::oneshot::{self, Sender};

pub struct SendOnDrop {
    sender: ManuallyDrop<Sender<()>>,
}

impl SendOnDrop {
    pub fn new(sender: Sender<()>) -> Self {
        Self {
            sender: ManuallyDrop::new(sender),
        }
    }
}

impl Drop for SendOnDrop {
    fn drop(&mut self) {
        let sender = unsafe { ManuallyDrop::take(&mut self.sender) };
        sender.send(()).unwrap();
    }
}

pub fn use_periodic_update(cx: Scope, interval: Duration) {
    cx.use_hook(|| {
        let update = cx.schedule_update();

        let (sender, mut receiver) = oneshot::channel();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut receiver => break,
                    _ = tokio::time::sleep(interval) => {
                        update();
                    }
                }
            }
        });

        SendOnDrop {
            sender: ManuallyDrop::new(sender),
        }
    });
}

pub fn use_periodic_update_future<'a, T, F>(
    cx: &'a Scope,
    interval: Duration,
    future_fabric: impl Send + Sync + 'static + Fn() -> F,
) -> Option<MappedRwLockReadGuard<'a, T>>
where
    T: Send + Sync + 'static,
    F: Send + Future<Output = T>,
{
    let (value, _) = cx.use_hook(|| {
        let value = Arc::new(RwLock::<Option<T>>::new(None));

        let update = cx.schedule_update();

        let (sender, mut receiver) = oneshot::channel();

        {
            let value = value.clone();
            tokio::spawn(async move {
                loop {
                    let val = Some(future_fabric().await);
                    *value.write() = val;

                    tokio::select! {
                        _ = &mut receiver => break,
                        _ = tokio::time::sleep(interval) => {
                            update();
                        }
                    }
                }
            });
        }

        (
            value,
            SendOnDrop {
                sender: ManuallyDrop::new(sender),
            },
        )
    });

    let opt_v = value.read();
    if opt_v.is_none() {
        None
    } else {
        return Some(RwLockReadGuard::map(opt_v, |v| v.as_ref().unwrap()));
    }
}
