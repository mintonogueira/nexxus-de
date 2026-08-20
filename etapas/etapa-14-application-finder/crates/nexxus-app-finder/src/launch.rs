//! Shell-free launch planning for Finder selections.

use std::collections::HashMap;
use std::process::{Child, Command};

use nexxus_xdg_application_index::{ApplicationRecord, LaunchContext};
use thiserror::Error;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedValue;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaunchPlan {
    Exec {
        program: String,
        arguments: Vec<String>,
    },
    DbusActivate {
        destination: String,
        object_path: String,
    },
}

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("application '{0}' has no usable Exec command and is not D-Bus activatable")]
    Unlaunchable(String),
    #[error("application D-Bus identifier '{0}' is invalid")]
    InvalidDbusDesktopId(String),
    #[error("failed to spawn '{program}': {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("D-Bus application activation failed: {0}")]
    Dbus(#[from] zbus::Error),
}

/// Converts the validated Stage 12 record into an auditable launch plan.
pub fn plan_application_launch(
    record: &ApplicationRecord,
    context: &LaunchContext,
) -> Result<LaunchPlan, LaunchError> {
    if record.dbus_activatable {
        return dbus_identity(record.id.as_str()).map(|(destination, object_path)| {
            LaunchPlan::DbusActivate {
                destination,
                object_path,
            }
        });
    }

    let template = record
        .exec
        .as_ref()
        .ok_or_else(|| LaunchError::Unlaunchable(record.id.as_str().to_owned()))?;
    let command = template.expand(
        context,
        &record.name,
        record.icon.exec_icon_value(),
        &record.desktop_file,
    );
    Ok(LaunchPlan::Exec {
        program: command.program,
        arguments: command.arguments,
    })
}

/// Executes argv literally. No command string is ever passed to a shell.
pub fn execute_launch_plan(plan: &LaunchPlan) -> Result<Option<Child>, LaunchError> {
    match plan {
        LaunchPlan::Exec { program, arguments } => Command::new(program)
            .args(arguments)
            .spawn()
            .map(Some)
            .map_err(|source| LaunchError::Spawn {
                program: program.clone(),
                source,
            }),
        LaunchPlan::DbusActivate {
            destination,
            object_path,
        } => {
            let connection = Connection::session()?;
            let platform_data: HashMap<String, OwnedValue> = HashMap::new();
            connection.call_method(
                Some(destination.as_str()),
                object_path.as_str(),
                Some("org.freedesktop.Application"),
                "Activate",
                &platform_data,
            )?;
            Ok(None)
        }
    }
}

fn dbus_identity(desktop_id: &str) -> Result<(String, String), LaunchError> {
    let stem = desktop_id
        .strip_suffix(".desktop")
        .ok_or_else(|| LaunchError::InvalidDbusDesktopId(desktop_id.to_owned()))?;
    if stem.is_empty()
        || stem.contains('/')
        || stem.starts_with('.')
        || stem.ends_with('.')
        || !stem.contains('.')
        || stem.split('.').any(|part| {
            part.is_empty()
                || part
                    .bytes()
                    .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'))
        })
    {
        return Err(LaunchError::InvalidDbusDesktopId(desktop_id.to_owned()));
    }

    Ok((
        stem.to_owned(),
        format!("/{}", stem.replace('.', "/").replace('-', "_")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_standard_dbus_identity() {
        assert_eq!(
            dbus_identity("org.example.Foo-Bar.desktop").unwrap(),
            (
                "org.example.Foo-Bar".to_owned(),
                "/org/example/Foo_Bar".to_owned()
            )
        );
    }

    #[test]
    fn rejects_non_bus_desktop_id() {
        assert!(matches!(
            dbus_identity("nested-demo.desktop"),
            Err(LaunchError::InvalidDbusDesktopId(_))
        ));
    }

    #[test]
    fn exec_plan_data_is_program_plus_argv_not_shell_text() {
        let plan = LaunchPlan::Exec {
            program: "printf".to_owned(),
            arguments: vec!["%s".to_owned(), "a; touch /tmp/never".to_owned()],
        };
        match plan {
            LaunchPlan::Exec { program, arguments } => {
                assert_eq!(program, "printf");
                assert_eq!(arguments[1], "a; touch /tmp/never");
            }
            LaunchPlan::DbusActivate { .. } => panic!("unexpected plan"),
        }
    }
}
