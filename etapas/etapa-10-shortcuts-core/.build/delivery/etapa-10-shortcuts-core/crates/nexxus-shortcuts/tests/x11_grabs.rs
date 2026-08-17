//! Real X11 grab validation. The CI wrapper runs this test under Xvfb.

use nexxus_shortcuts::{ShortcutRegistry, X11ShortcutGrabs};

#[test]
#[ignore = "requires a real X11 server; CI executes it under Xvfb"]
fn installs_and_releases_default_grabs() {
    let registry = ShortcutRegistry::with_defaults();
    let mut grabs = X11ShortcutGrabs::install(None, &registry)
        .expect("default shortcut grabs must install on an isolated Xvfb server");

    assert!(!grabs.specs().is_empty());
    assert!(grabs.specs().iter().all(|spec| !spec.trigger.is_bare_f11()));

    grabs.uninstall().expect("default grabs must be releasable");
    assert!(grabs.specs().is_empty());
}
