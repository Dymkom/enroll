use crate::app::{
    Message,
    error::AppError,
    fprint::{enroll_fingerprint_process, verify_finger_process},
};
use cosmic::iced::{Subscription, futures::channel::mpsc::Sender};
use futures_util::SinkExt;

#[derive(Clone)]
pub(crate) struct VerifyData {
    device_path: std::sync::Arc<zbus::zvariant::OwnedObjectPath>,
    connection: zbus::Connection,
    username: std::sync::Arc<String>,
    finger: String,
}

impl VerifyData {
    pub(crate) fn new(
        device_path: std::sync::Arc<zbus::zvariant::OwnedObjectPath>,
        connection: zbus::Connection,
        username: std::sync::Arc<String>,
        finger: String,
    ) -> Self {
        Self {
            device_path,
            connection,
            username,
            finger,
        }
    }
}

impl std::hash::Hash for VerifyData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.username.hash(state);
        self.finger.hash(state);
    }
}

#[derive(Clone)]
pub(crate) struct EnrollData {
    finger_name: std::sync::Arc<String>,
    device_path: std::sync::Arc<zbus::zvariant::OwnedObjectPath>,
    connection: zbus::Connection,
    username: std::sync::Arc<String>,
}

impl EnrollData {
    pub(crate) fn new(
        finger_name: std::sync::Arc<String>,
        device_path: std::sync::Arc<zbus::zvariant::OwnedObjectPath>,
        connection: zbus::Connection,
        username: std::sync::Arc<String>,
    ) -> Self {
        Self {
            finger_name,
            device_path,
            connection,
            username,
        }
    }
}

impl std::hash::Hash for EnrollData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.finger_name.hash(state);
        self.username.hash(state);
    }
}

/// ***Returns*** a subscription to an ongoing enroll process
pub(crate) fn enroll_subscription(data: EnrollData) -> Subscription<Message> {
    Subscription::run_with(data, |data| {
        let data = data.clone();
        cosmic::iced::stream::channel(100, move |mut output: Sender<Message>| async move {
            // Implement enrollment stream here
            match enroll_fingerprint_process(
                data.connection,
                &data.device_path,
                &data.finger_name,
                &data.username,
                &mut output,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    let _ = output
                        .send(Message::OperationError(AppError::from(e)))
                        .await;
                }
            }
            futures_util::future::pending().await
        })
    })
}

/// ***Returns*** a subscription to an ongoing verify process
pub(crate) fn verify_subscription(data: VerifyData) -> Subscription<Message> {
    Subscription::run_with(data, |data| {
        let data = data.clone();
        cosmic::iced::stream::channel(100, move |mut output: Sender<Message>| async move {
            let path = (*data.device_path).clone();
            let username = (*data.username).clone();

            match verify_finger_process(&data.connection, path, data.finger, username, &mut output)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    let _ = output
                        .send(Message::OperationError(AppError::from(e)))
                        .await;
                }
            }
            futures_util::future::pending().await
        })
    })
}
