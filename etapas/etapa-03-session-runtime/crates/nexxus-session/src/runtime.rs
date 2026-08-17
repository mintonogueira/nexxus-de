//! Deterministic Session Runtime orchestration.
//!
//! Backend-specific mechanics are injected through the already established
//! `NexxusModule` lifecycle contract. This module owns only selection,
//! dependency wiring, bootstrap, diagnostics and reverse-order shutdown.

use crate::{SessionControlRequest, SessionControlResponse, SessionStatus};
use nexxus_backend_api::BackendKind;
use nexxus_core::{
    ApiVersion, CORE_API_VERSION, CapabilityId, CapabilitySelections, Dependency, EventBus,
    IsolationMode, LifecycleError, LifecycleManager, ModuleContext, ModuleDescriptor,
    ModuleFailure, ModuleId, ModuleRegistry, NexxusModule,
};
use nexxus_protocol::{
    Message, MessageKind, PROTOCOL_VERSION, ProtocolError, UnixConnection, UnixEndpoint,
};
use nexxus_wm::WindowManager;
use std::collections::BTreeMap;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use thiserror::Error;

const GRAPHICS_CAPABILITY: &str = "graphics.backend";

#[derive(Debug, Error)]
pub enum SessionRuntimeError {
    #[error("selected backend {backend:?} is unavailable in this session integration")]
    BackendUnavailable { backend: BackendKind },
    #[error("backend module '{module}' does not match selected backend {backend:?}")]
    BackendModuleMismatch {
        backend: BackendKind,
        module: ModuleId,
    },
    #[error("backend module declares {declared:?} but session selected {selected:?}")]
    BackendKindMismatch {
        selected: BackendKind,
        declared: BackendKind,
    },
    #[error("backend module '{module}' does not provide '{capability}'")]
    MissingBackendCapability {
        module: ModuleId,
        capability: &'static str,
    },
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error("session shutdown completed with {0} module cleanup error(s)")]
    ShutdownFailures(usize),
}

/// Backend implementation passed to the Session Runtime by an integration
/// stage. Etapa 03 deliberately does not create X11 or Wayland backends.
pub struct BackendModule {
    pub kind: BackendKind,
    pub module: Box<dyn NexxusModule>,
}

/// Runtime state after preflight has succeeded and before module startup.
pub type SessionEnvironment = BTreeMap<String, String>;

pub struct SessionRuntime {
    backend: BackendKind,
    environment: SessionEnvironment,
    backend_module_id: ModuleId,
    lifecycle: LifecycleManager,
    startup_order: Vec<ModuleId>,
    control_endpoint: UnixEndpoint,
    control_socket: PathBuf,
}

impl SessionRuntime {
    /// Builds the validated module graph and reserves the private IPC endpoint.
    /// No module receives control until all descriptors and implementations are
    /// installed successfully.
    pub fn prepare(
        backend: BackendKind,
        runtime_dir: &Path,
        backend_module: Option<BackendModule>,
    ) -> Result<Self, SessionRuntimeError> {
        let backend_module =
            backend_module.ok_or(SessionRuntimeError::BackendUnavailable { backend })?;
        validate_backend_module(backend, &backend_module)?;

        let backend_descriptor = backend_module.module.descriptor().clone();
        let backend_module_id = backend_descriptor.id.clone();
        let wm_descriptor = wm_descriptor();

        let mut registry = ModuleRegistry::new(CORE_API_VERSION);
        registry.register(backend_descriptor)?;
        registry.register(wm_descriptor.clone())?;

        let context = ModuleContext::new(runtime_dir, EventBus::new());
        let mut lifecycle = LifecycleManager::new(registry, context);
        lifecycle.install(backend_module.module)?;
        lifecycle.install(Box::new(WmLifecycleModule::new(wm_descriptor)))?;

        let control_socket = runtime_dir.join("session.sock");
        let control_endpoint = UnixEndpoint::bind_private(&control_socket)?;

        Ok(Self {
            backend,
            environment: prepared_environment(backend),
            backend_module_id,
            lifecycle,
            startup_order: Vec::new(),
            control_endpoint,
            control_socket,
        })
    }

    pub fn backend(&self) -> BackendKind {
        self.backend
    }

    pub fn control_socket(&self) -> &Path {
        &self.control_socket
    }

    /// Returns XDG session variables prepared for processes launched by later
    /// integration stages. The runtime does not mutate global process state.
    pub fn environment(&self) -> &SessionEnvironment {
        &self.environment
    }

    /// Starts the selected backend before the WM through the Core dependency
    /// graph. A complete order is retained for deterministic reverse shutdown.
    pub fn start(&mut self) -> Result<(), SessionRuntimeError> {
        let mut selections = CapabilitySelections::default();
        selections.select(graphics_capability(), self.backend_module_id.clone());
        self.startup_order = self.lifecycle.start_all(&selections)?;
        Ok(())
    }

    /// Starts the graph, serves control requests and always attempts orderly
    /// cleanup before returning.
    pub fn run(mut self) -> Result<(), SessionRuntimeError> {
        self.start()?;
        let control_result = self.run_control_loop();
        let shutdown_result = self.shutdown();
        control_result?;
        shutdown_result
    }

    pub fn run_control_loop(&mut self) -> Result<(), SessionRuntimeError> {
        loop {
            let mut connection = self.control_endpoint.accept()?;
            let request = match connection.receive::<SessionControlRequest>() {
                Ok(message) => message,
                Err(error) => {
                    tracing::warn!(%error, "invalid session control request");
                    continue;
                }
            };
            PROTOCOL_VERSION.negotiate(request.protocol)?;

            match request.payload {
                SessionControlRequest::Status => {
                    let response = Message {
                        protocol: PROTOCOL_VERSION,
                        request_id: request.request_id,
                        kind: MessageKind::Response,
                        payload: SessionControlResponse::Status(self.status()),
                    };
                    connection.send(&response)?;
                }
                SessionControlRequest::Shutdown { reason } => {
                    tracing::info!(%reason, "orderly Nexxus session shutdown requested");
                    let response = Message {
                        protocol: PROTOCOL_VERSION,
                        request_id: request.request_id,
                        kind: MessageKind::Response,
                        payload: SessionControlResponse::Accepted,
                    };
                    connection.send(&response)?;
                    return Ok(());
                }
            }
        }
    }

    pub fn status(&self) -> SessionStatus {
        let modules = self
            .startup_order
            .iter()
            .map(|id| (id.as_str().to_owned(), self.lifecycle.state(id)))
            .collect();
        SessionStatus {
            backend: self.backend,
            control_socket: self.control_socket.clone(),
            modules,
        }
    }

    /// Stops every started module in the exact reverse dependency order.
    /// Cleanup continues after individual failures and the aggregate failure is
    /// reported only after all remaining modules have received `stop`.
    pub fn shutdown(&mut self) -> Result<(), SessionRuntimeError> {
        let errors = self.lifecycle.stop_all(&self.startup_order);
        self.startup_order.clear();
        if errors.is_empty() {
            Ok(())
        } else {
            for error in &errors {
                tracing::error!(%error, "module shutdown failed");
            }
            Err(SessionRuntimeError::ShutdownFailures(errors.len()))
        }
    }
}

/// Queries a running Session Runtime over its private Unix socket.
pub fn query_status(socket: &Path) -> Result<SessionStatus, SessionRuntimeError> {
    match send_control(socket, SessionControlRequest::Status)? {
        SessionControlResponse::Status(status) => Ok(status),
        SessionControlResponse::Error(message) => Err(SessionRuntimeError::Protocol(
            ProtocolError::Io(std::io::Error::other(message)),
        )),
        SessionControlResponse::Accepted => Err(SessionRuntimeError::Protocol(ProtocolError::Io(
            std::io::Error::other("unexpected shutdown acknowledgement for status request"),
        ))),
    }
}

/// Requests an orderly shutdown. The runtime acknowledges before beginning the
/// reverse dependency cleanup, so the caller knows the request was accepted.
pub fn request_shutdown(
    socket: &Path,
    reason: impl Into<String>,
) -> Result<(), SessionRuntimeError> {
    match send_control(
        socket,
        SessionControlRequest::Shutdown {
            reason: reason.into(),
        },
    )? {
        SessionControlResponse::Accepted => Ok(()),
        SessionControlResponse::Error(message) => Err(SessionRuntimeError::Protocol(
            ProtocolError::Io(std::io::Error::other(message)),
        )),
        SessionControlResponse::Status(_) => Err(SessionRuntimeError::Protocol(ProtocolError::Io(
            std::io::Error::other("unexpected status response for shutdown request"),
        ))),
    }
}

fn send_control(
    socket: &Path,
    payload: SessionControlRequest,
) -> Result<SessionControlResponse, SessionRuntimeError> {
    let stream = UnixStream::connect(socket).map_err(ProtocolError::Io)?;
    let mut connection = UnixConnection::new(stream);
    let request = Message {
        protocol: PROTOCOL_VERSION,
        request_id: 1,
        kind: MessageKind::Request,
        payload,
    };
    connection.send(&request)?;
    let response = connection.receive::<SessionControlResponse>()?;
    PROTOCOL_VERSION.negotiate(response.protocol)?;
    Ok(response.payload)
}

fn prepared_environment(backend: BackendKind) -> SessionEnvironment {
    let mut environment = BTreeMap::new();
    environment.insert("XDG_CURRENT_DESKTOP".into(), "Nexxus".into());
    environment.insert("XDG_SESSION_DESKTOP".into(), "nexxus".into());
    environment.insert(
        "XDG_SESSION_TYPE".into(),
        match backend {
            BackendKind::X11 => "x11",
            BackendKind::Wayland => "wayland",
        }
        .into(),
    );
    environment
}

fn graphics_capability() -> CapabilityId {
    CapabilityId::new(GRAPHICS_CAPABILITY).expect("static capability id is valid")
}

fn expected_backend_module_id(backend: BackendKind) -> ModuleId {
    let value = match backend {
        BackendKind::X11 => "nexxus-backend-x11",
        BackendKind::Wayland => "nexxus-backend-wayland",
    };
    ModuleId::new(value).expect("static backend module id is valid")
}

fn validate_backend_module(
    backend: BackendKind,
    candidate: &BackendModule,
) -> Result<(), SessionRuntimeError> {
    if candidate.kind != backend {
        return Err(SessionRuntimeError::BackendKindMismatch {
            selected: backend,
            declared: candidate.kind,
        });
    }
    let descriptor = candidate.module.descriptor();
    let expected = expected_backend_module_id(backend);
    if descriptor.id != expected {
        return Err(SessionRuntimeError::BackendModuleMismatch {
            backend,
            module: descriptor.id.clone(),
        });
    }
    let capability = graphics_capability();
    if !descriptor.provides.contains(&capability) {
        return Err(SessionRuntimeError::MissingBackendCapability {
            module: descriptor.id.clone(),
            capability: GRAPHICS_CAPABILITY,
        });
    }
    Ok(())
}

fn wm_descriptor() -> ModuleDescriptor {
    ModuleDescriptor {
        id: nexxus_wm::module_id(),
        name: "Nexxus Window Manager Core".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        required_core_api: ApiVersion::new(1, 0),
        provides: Vec::new(),
        requires: vec![Dependency::Capability(graphics_capability())],
        optional: false,
        isolation: IsolationMode::InProcess,
    }
}

/// Lifecycle adapter around the already-implemented backend-neutral WM engine.
/// No window-management behavior is reproduced in Session Runtime.
struct WmLifecycleModule {
    descriptor: ModuleDescriptor,
    manager: Option<WindowManager>,
}

impl WmLifecycleModule {
    fn new(descriptor: ModuleDescriptor) -> Self {
        Self {
            descriptor,
            manager: None,
        }
    }
}

impl NexxusModule for WmLifecycleModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn initialize(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
        self.manager = Some(WindowManager::new());
        Ok(())
    }

    fn start(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
        if self.manager.is_none() {
            return Err(ModuleFailure::new("window manager was not initialized"));
        }
        Ok(())
    }

    fn stop(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
        self.manager = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexxus_core::{CapabilityId, ModuleContext};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeBackend {
        descriptor: ModuleDescriptor,
        calls: Arc<Mutex<Vec<&'static str>>>,
        fail_start: bool,
    }

    impl NexxusModule for FakeBackend {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }
        fn initialize(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
            self.calls.lock().unwrap().push("backend.initialize");
            Ok(())
        }
        fn start(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
            self.calls.lock().unwrap().push("backend.start");
            if self.fail_start {
                return Err(ModuleFailure::new("synthetic backend failure"));
            }
            Ok(())
        }
        fn stop(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
            self.calls.lock().unwrap().push("backend.stop");
            Ok(())
        }
    }

    fn private_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nexxus-session-runtime-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    fn backend(
        kind: BackendKind,
        calls: Arc<Mutex<Vec<&'static str>>>,
        fail_start: bool,
    ) -> BackendModule {
        let id = expected_backend_module_id(kind);
        BackendModule {
            kind,
            module: Box::new(FakeBackend {
                descriptor: ModuleDescriptor {
                    id,
                    name: "fake backend".into(),
                    version: "0.1.0".into(),
                    required_core_api: ApiVersion::new(1, 0),
                    provides: vec![CapabilityId::new(GRAPHICS_CAPABILITY).unwrap()],
                    requires: Vec::new(),
                    optional: false,
                    isolation: IsolationMode::InProcess,
                },
                calls,
                fail_start,
            }),
        }
    }

    #[test]
    fn unavailable_explicit_backend_fails_before_module_start() {
        let dir = private_dir();
        let result = SessionRuntime::prepare(BackendKind::X11, &dir, None);
        assert!(matches!(
            result,
            Err(SessionRuntimeError::BackendUnavailable {
                backend: BackendKind::X11
            })
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn starts_backend_before_wm_and_stops_in_reverse_order() {
        let dir = private_dir();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = SessionRuntime::prepare(
            BackendKind::X11,
            &dir,
            Some(backend(BackendKind::X11, Arc::clone(&calls), false)),
        )
        .unwrap();
        assert_eq!(
            runtime
                .environment()
                .get("XDG_SESSION_TYPE")
                .map(String::as_str),
            Some("x11")
        );
        runtime.start().unwrap();
        assert_eq!(runtime.status().modules.len(), 2);
        runtime.shutdown().unwrap();
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &["backend.initialize", "backend.start", "backend.stop"]
        );
        drop(runtime);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn private_ipc_reports_status_and_requests_orderly_shutdown() {
        let dir = private_dir();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runtime = SessionRuntime::prepare(
            BackendKind::X11,
            &dir,
            Some(backend(BackendKind::X11, Arc::clone(&calls), false)),
        )
        .unwrap();
        let socket = runtime.control_socket().to_path_buf();
        let handle = std::thread::spawn(move || runtime.run());

        let status = query_status(&socket).unwrap();
        assert_eq!(status.backend, BackendKind::X11);
        assert_eq!(status.modules.len(), 2);
        assert!(
            status
                .modules
                .iter()
                .all(|(_, state)| *state == Some(nexxus_core::ModuleState::Running))
        );

        request_shutdown(&socket, "test-complete").unwrap();
        handle.join().unwrap().unwrap();
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &["backend.initialize", "backend.start", "backend.stop"]
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn backend_start_failure_rolls_back_without_starting_wm() {
        let dir = private_dir();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = SessionRuntime::prepare(
            BackendKind::Wayland,
            &dir,
            Some(backend(BackendKind::Wayland, Arc::clone(&calls), true)),
        )
        .unwrap();
        assert!(runtime.start().is_err());
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &["backend.initialize", "backend.start", "backend.stop"]
        );
        drop(runtime);
        fs::remove_dir_all(dir).unwrap();
    }
}
