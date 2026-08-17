//! Background worker and cloneable control handle for the X11 runtime.

use crate::runtime::X11Runtime;
use crate::{X11BackendError, operation_error};
use nexxus_backend_api::OutputInfo;
use nexxus_wm::{Window, WindowId};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

enum Action {
    Snapshot,
    Focus(WindowId),
    Move(WindowId, i32, i32),
    Resize(WindowId, u32, u32),
    Maximize(WindowId),
    Restore(WindowId),
    Fullscreen(WindowId, bool),
    Close(WindowId),
    Stop,
}

enum Response {
    Unit,
    Windows(Vec<Window>),
}

struct Control {
    action: Action,
    reply: Sender<Result<Response, String>>,
}

/// Thread-safe command endpoint. All X11 access still occurs exclusively in
/// the worker thread, preserving request/event ordering.
#[derive(Clone)]
pub struct X11Controller {
    sender: Sender<Control>,
}

impl X11Controller {
    pub fn windows(&self) -> Result<Vec<Window>, X11BackendError> {
        match self.call(Action::Snapshot)? {
            Response::Windows(windows) => Ok(windows),
            Response::Unit => Err(X11BackendError::UnexpectedResponse),
        }
    }

    pub fn focus(&self, id: WindowId) -> Result<(), X11BackendError> { self.unit(Action::Focus(id)) }
    pub fn move_window(&self, id: WindowId, x: i32, y: i32) -> Result<(), X11BackendError> { self.unit(Action::Move(id, x, y)) }
    pub fn resize_window(&self, id: WindowId, width: u32, height: u32) -> Result<(), X11BackendError> { self.unit(Action::Resize(id, width, height)) }
    pub fn maximize(&self, id: WindowId) -> Result<(), X11BackendError> { self.unit(Action::Maximize(id)) }
    pub fn restore(&self, id: WindowId) -> Result<(), X11BackendError> { self.unit(Action::Restore(id)) }
    pub fn fullscreen(&self, id: WindowId, enabled: bool) -> Result<(), X11BackendError> { self.unit(Action::Fullscreen(id, enabled)) }
    pub fn close(&self, id: WindowId) -> Result<(), X11BackendError> { self.unit(Action::Close(id)) }

    fn unit(&self, action: Action) -> Result<(), X11BackendError> {
        match self.call(action)? {
            Response::Unit => Ok(()),
            Response::Windows(_) => Err(X11BackendError::UnexpectedResponse),
        }
    }

    fn call(&self, action: Action) -> Result<Response, X11BackendError> {
        let (reply, receiver) = mpsc::channel();
        self.sender.send(Control { action, reply }).map_err(|_| X11BackendError::WorkerStopped)?;
        receiver.recv().map_err(|_| X11BackendError::WorkerStopped)?
            .map_err(X11BackendError::Operation)
    }
}

/// Owns the X11 worker lifecycle and exposes only backend-neutral state to
/// callers. Dropping the service performs an orderly stop and joins the worker.
pub struct X11Service {
    controller: X11Controller,
    worker: Option<JoinHandle<()>>,
    output: OutputInfo,
}

impl X11Service {
    pub fn start(display: Option<String>) -> Result<Self, X11BackendError> {
        let (control_tx, control_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("nexxus-x11-backend".into())
            .spawn(move || worker_main(display, control_rx, ready_tx))
            .map_err(operation_error)?;

        let output = match ready_rx.recv().map_err(|_| X11BackendError::WorkerStopped)? {
            Ok(output) => output,
            Err(message) => {
                let _ = worker.join();
                return Err(X11BackendError::Unavailable(message));
            }
        };
        Ok(Self {
            controller: X11Controller { sender: control_tx },
            worker: Some(worker),
            output,
        })
    }

    pub fn controller(&self) -> X11Controller { self.controller.clone() }
    pub fn output(&self) -> OutputInfo { self.output.clone() }

    pub fn stop(&mut self) -> Result<(), X11BackendError> {
        if self.worker.is_none() {
            return Ok(());
        }
        self.controller.unit(Action::Stop)?;
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| X11BackendError::Operation("X11 worker panicked".into()))?;
        }
        Ok(())
    }
}

impl Drop for X11Service {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn worker_main(
    display: Option<String>,
    controls: Receiver<Control>,
    ready: mpsc::SyncSender<Result<OutputInfo, String>>,
) {
    let mut runtime = match X11Runtime::connect_and_claim(display.as_deref()) {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = ready.send(Err(error.to_string()));
            return;
        }
    };
    let output = match runtime.output() {
        Ok(output) => output,
        Err(error) => {
            let _ = ready.send(Err(error.to_string()));
            return;
        }
    };
    if ready.send(Ok(output)).is_err() {
        let _ = runtime.shutdown();
        return;
    }

    let mut running = true;
    while running {
        if let Err(error) = runtime.drain_events() {
            eprintln!("nexxus-backend-x11: event loop failed: {error}");
            break;
        }
        while let Ok(control) = controls.try_recv() {
            let (result, stop) = execute_control(&mut runtime, control.action);
            let _ = control.reply.send(result.map_err(|error| error.to_string()));
            if stop {
                running = false;
                break;
            }
        }
        if running {
            thread::sleep(Duration::from_millis(4));
        }
    }
    let _ = runtime.shutdown();
}

fn execute_control(runtime: &mut X11Runtime, action: Action) -> (Result<Response, X11BackendError>, bool) {
    let result = match action {
        Action::Snapshot => Ok(Response::Windows(runtime.windows())),
        Action::Focus(id) => runtime.focus(id).map(|_| Response::Unit),
        Action::Move(id, x, y) => runtime.move_window(id, x, y).map(|_| Response::Unit),
        Action::Resize(id, width, height) => runtime.resize_window(id, width, height).map(|_| Response::Unit),
        Action::Maximize(id) => runtime.maximize(id).map(|_| Response::Unit),
        Action::Restore(id) => runtime.restore(id).map(|_| Response::Unit),
        Action::Fullscreen(id, enabled) => runtime.fullscreen(id, enabled).map(|_| Response::Unit),
        Action::Close(id) => runtime.close(id).map(|_| Response::Unit),
        Action::Stop => return (Ok(Response::Unit), true),
    };
    (result, false)
}
