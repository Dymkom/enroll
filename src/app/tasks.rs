// SPDX-License-Identifier: MPL-2.0

use crate::app::error::AppError;
use crate::app::fprint::*;
use crate::app::message::Message;
use crate::fprint_dbus::*;
use cosmic::Task;

pub fn task_delete_prints(
    path: zbus::zvariant::OwnedObjectPath,
    username: String,
    conn: zbus::Connection,
) -> Task<cosmic::Action<Message>> {
    Task::perform(
        async move {
            match delete_fingers(&conn, path, username).await {
                Ok(_) => Message::DeleteComplete,
                Err(e) => Message::OperationError(AppError::from(e)),
            }
        },
        cosmic::Action::App,
    )
}

pub fn task_delete_print(
    path: zbus::zvariant::OwnedObjectPath,
    username: String,
    finger_name: String,
    conn: zbus::Connection,
) -> Task<cosmic::Action<Message>> {
    Task::perform(
        async move {
            match delete_fingerprint_dbus(&conn, path, finger_name, username).await {
                Ok(_) => Message::DeleteComplete,
                Err(e) => Message::OperationError(AppError::from(e)),
            }
        },
        cosmic::Action::App,
    )
}

pub fn task_enroll_stop(
    path: zbus::zvariant::OwnedObjectPath,
    conn: zbus::Connection,
) -> Task<cosmic::Action<Message>> {
    Task::perform(
        async move {
            let device = DeviceProxy::builder(&conn).path(path)?.build().await?;
            let _ = device.enroll_stop().await;
            device.release().await?;
            Ok::<(), zbus::Error>(())
        },
        |res| match res {
            Ok(_) => {
                cosmic::Action::App(Message::EnrollStatus("enroll-cancelled".to_string(), true))
            }
            Err(e) => cosmic::Action::App(Message::OperationError(AppError::from(e))),
        },
    )
}

pub fn task_verify_finger(
    path: zbus::zvariant::OwnedObjectPath,
    username: String,
    finger: String,
    conn: zbus::Connection,
) -> Task<cosmic::Action<Message>> {
    Task::perform(
        async move { verify_finger_dbus(&conn, path, finger, username).await },
        |res| match res {
            Ok(()) => cosmic::Action::App(Message::Success),
            Err(e) => cosmic::Action::App(Message::OperationError(AppError::from(e))),
        },
    )
}

pub fn task_clear_device(
    path: zbus::zvariant::OwnedObjectPath,
    usernames: Vec<String>,
    conn: zbus::Connection,
) -> Task<cosmic::Action<Message>> {
    Task::perform(
        async move {
            match clear_all_fingers_dbus(&conn, path, usernames).await {
                Ok(_) => Message::ClearComplete(Ok(())),
                Err(e) => Message::ClearComplete(Err(AppError::from(e))),
            }
        },
        cosmic::Action::App,
    )
}

pub fn task_find_device(conn_clone: zbus::Connection) -> Task<cosmic::Action<Message>> {
    Task::perform(
        async move {
            match find_device(&conn_clone).await {
                Ok((path, proxy)) => Message::DeviceFound(Some((path, proxy))),
                Err(e) => {
                    let error = AppError::from(e);
                    if matches!(error, AppError::Unknown(_)) {
                        Message::OperationError(AppError::DeviceNotFound)
                    } else {
                        Message::OperationError(error)
                    }
                }
            }
        },
        cosmic::Action::App,
    )
}
