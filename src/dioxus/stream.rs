use core::pin::pin;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use dioxus::prelude::*;
use futures::{Future, Stream, StreamExt};

use tokio::sync::{oneshot, watch};

use super::periodic::SendOnDrop;

pub async fn cancellable<T>(
    kill: oneshot::Receiver<()>,
    task: impl Future<Output = T>,
) -> anyhow::Result<T> {
    tokio::select! {
        output = task => Ok(output),
        _ = kill => anyhow::bail!("Task was cancelled"),
    }
}

pub fn use_stream<'a, T, F, S, P>(
    cx: &'a Scope<P>,
    max_values: usize,
    stream_creator: impl Send + Sync + 'static + Fn() -> F,
) -> RwLockReadGuard<'a, Option<Vec<T>>>
where
    T: Send + Sync + 'static,
    F: Send + Future<Output = S>,
    S: Send + Stream<Item = T>,
{
    let (value, sender_max_vals, receiver_max_vals, _) = cx.use_hook(|| {
        let value = Arc::new(RwLock::new(None));

        let update = cx.schedule_update();

        let (sender_cancel, receiver_cancel) = oneshot::channel();
        let (sender_max_vals, receiver_max_vals) = watch::channel(max_values);

        {
            let mut receiver_max_vals = receiver_max_vals.clone();
            let value = value.clone();
            tokio::spawn(cancellable(receiver_cancel, async move {
                let mut stream = pin!(stream_creator().await);
                let mut cnt = 0;

                *value.write().unwrap() = Some(Vec::new());

                loop {
                    while cnt >= *receiver_max_vals.borrow() {
                        let Ok(_) = receiver_max_vals.changed().await else {
                            return;
                        };
                    }

                    let Some(val) = stream.next().await else {
                        return;
                    };

                    let mut opt = value.write().unwrap();
                    let vec = opt.as_mut().unwrap();
                    vec.push(val);
                    cnt += 1;

                    update();
                }
            }));
        }

        (
            value,
            sender_max_vals,
            receiver_max_vals,
            SendOnDrop::new(sender_cancel),
        )
    });

    if *receiver_max_vals.borrow() != max_values {
        sender_max_vals.send(max_values).unwrap();
    }

    value.read().unwrap()
}

pub fn use_last_stream_value<'a, Arg, T, F, S, P>(
    cx: &'a Scope<P>,
    arg: Arg,
    stream_creator: impl Send + Sync + 'static + Fn(Arg) -> F,
) -> RwLockReadGuard<'a, Option<T>>
where
    T: Send + Sync + 'static,
    F: Send + Future<Output = S>,
    S: Send + Stream<Item = T>,
    Arg: std::fmt::Debug + Eq + Clone + Send + Sync + 'static,
{
    let thearg = arg.clone();

    let (value, sender_args, receiver_args, _) = cx.use_hook(|| {
        let value = Arc::new(RwLock::new(None));

        let update = cx.schedule_update();

        let (sender_cancel, receiver_cancel) = oneshot::channel();
        let (sender_args, receiver_args) = watch::channel(arg);

        {
            let mut receiver_args = receiver_args.clone();
            let value = value.clone();

            tokio::spawn(cancellable(receiver_cancel, async move {
                loop {
                    let current_arg = receiver_args.borrow().clone();
                    let mut stream = pin!(stream_creator(current_arg.clone()).await);

                    loop {
                        tokio::select! {
                            _ = receiver_args.changed() => {
                                break;
                            }
                            val = stream.next() => {
                                match val {
                                    None => {
                                        break;
                                    }
                                    Some(val) => {
                                        *value.write().unwrap() = Some(val);
                                        update();
                                    }
                                }
                            }
                        }
                    }

                    if current_arg == *receiver_args.borrow()
                        && receiver_args.changed().await.is_err()
                    {
                        return;
                    }
                }
            }));
        }

        (
            value,
            sender_args,
            receiver_args,
            SendOnDrop::new(sender_cancel),
        )
    });

    if *receiver_args.borrow() != thearg {
        sender_args.send(thearg).unwrap();
    }

    value.read().unwrap()
}
