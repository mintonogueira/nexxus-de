//! Public-contract tests for Etapa 03.

use nexxus_backend_api::BackendKind;
use nexxus_session::{SessionConfig, SessionConfigError, parse_backend};

#[test]
fn explicit_backend_contract_accepts_only_supported_names() {
    assert_eq!(parse_backend("x11").unwrap(), BackendKind::X11);
    assert_eq!(parse_backend("wayland").unwrap(), BackendKind::Wayland);
    assert!(matches!(
        parse_backend("auto"),
        Err(SessionConfigError::InvalidBackend(_))
    ));
}

#[test]
fn configuration_never_invents_a_backend() {
    assert!(matches!(
        SessionConfig::default().resolve_backend(None),
        Err(SessionConfigError::BackendRequired)
    ));
}
