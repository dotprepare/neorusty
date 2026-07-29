use crate::jvm::{JvmError, JvmManager};
use jni::objects::JValue;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use tokio::sync::oneshot;

pub enum JvmCommand {
    CallStaticInt {
        class: String,
        method: String,
        sig: String,
        args: Vec<i32>,
        response: oneshot::Sender<Result<i32, JvmError>>,
    },
    Shutdown,
}

pub struct JvmWorker {
    tx: mpsc::Sender<JvmCommand>,
}

impl JvmWorker {
    pub fn spawn(manager: Arc<JvmManager>) -> Self {
        let (tx, rx) = mpsc::channel::<JvmCommand>();

        thread::spawn(move || {
            for cmd in rx {
                match cmd {
                    JvmCommand::CallStaticInt {
                        class,
                        method,
                        sig,
                        args,
                        response,
                    } => {
                        let jargs: Vec<JValue> =
                            args.into_iter().map(JValue::from).collect();
                        let result =
                            manager.call_static_int(&class, &method, &sig, &jargs);
                        let _ = response.send(result);
                    }
                    JvmCommand::Shutdown => break,
                }
            }
        });

        Self { tx }
    }

    pub async fn call_int(
        &self,
        class: &str,
        method: &str,
        sig: &str,
        args: Vec<i32>,
    ) -> Result<i32, JvmError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(JvmCommand::CallStaticInt {
                class: class.to_string(),
                method: method.to_string(),
                sig: sig.to_string(),
                args,
                response: tx,
            })
            .map_err(|e| JvmError::Internal(format!("Worker send failed: {e}")))?;
        rx.await
            .map_err(|e| JvmError::Internal(format!("Worker response failed: {e}")))
            .and_then(std::convert::identity)
    }

    pub fn shutdown_sync(self) {
        let _ = self.tx.send(JvmCommand::Shutdown);
    }
}
