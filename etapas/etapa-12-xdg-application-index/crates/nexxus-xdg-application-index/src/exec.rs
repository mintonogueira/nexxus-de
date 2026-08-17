//! Safe parsing and field-code expansion for the Desktop Entry `Exec` key.
//!
//! The module intentionally returns argv data. It never invokes a shell and it
//! never executes a process, keeping launch policy outside the application index.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// One parsed argument from an XDG `Exec` command line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecArgument {
    Literal(String),
    File,
    Files,
    Url,
    Urls,
    Icon,
    Name,
    DesktopFile,
}

/// Validated command template retained by an application record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecTemplate {
    raw: String,
    program: String,
    arguments: Vec<ExecArgument>,
}

/// Dynamic data used when a launcher later expands a validated template.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LaunchContext {
    pub files: Vec<PathBuf>,
    pub urls: Vec<String>,
}

/// Shell-free executable plus argument vector ready for `std::process::Command`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchCommand {
    pub program: String,
    pub arguments: Vec<String>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ExecError {
    #[error("Exec is empty")]
    Empty,
    #[error("Exec has an unterminated quoted argument")]
    UnterminatedQuote,
    #[error("Exec has a trailing escape")]
    TrailingEscape,
    #[error("the executable name may not contain '='")]
    InvalidProgram,
    #[error("field code %{0} is not recognized by Desktop Entry 1.5")]
    UnknownFieldCode(char),
    #[error("field codes may not occur inside quoted arguments")]
    FieldCodeInsideQuotedArgument,
    #[error("field code %{0} must be a standalone argument")]
    FieldCodeMustBeStandalone(char),
    #[error("Exec contains more than one file/URL field code")]
    TooManyFileFields,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    value: String,
    was_quoted: bool,
}

impl ExecTemplate {
    /// Parses Desktop Entry quoting and validates all field codes before an
    /// entry is admitted to the index.
    pub fn parse(raw: &str) -> Result<Self, ExecError> {
        let tokens = tokenize(raw)?;
        let Some(program_token) = tokens.first() else {
            return Err(ExecError::Empty);
        };
        if program_token.value.is_empty() || program_token.value.contains('=') {
            return Err(ExecError::InvalidProgram);
        }
        if contains_field_code(&program_token.value) {
            return Err(ExecError::InvalidProgram);
        }

        let mut file_fields = 0usize;
        let mut arguments = Vec::new();
        for token in tokens.iter().skip(1) {
            arguments.extend(parse_argument(token, &mut file_fields)?);
        }
        if file_fields > 1 {
            return Err(ExecError::TooManyFileFields);
        }

        Ok(Self {
            raw: raw.to_owned(),
            program: unescape_literal_percent(&program_token.value)?,
            arguments,
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn arguments(&self) -> &[ExecArgument] {
        &self.arguments
    }

    /// Expands field codes without re-tokenizing replacements. Each replacement
    /// remains one argv element unless `%F`, `%U` or `%i` explicitly requires more.
    pub fn expand(
        &self,
        context: &LaunchContext,
        name: &str,
        icon: Option<&str>,
        desktop_file: &Path,
    ) -> LaunchCommand {
        let mut arguments = Vec::new();
        for argument in &self.arguments {
            match argument {
                ExecArgument::Literal(value) => arguments.push(value.clone()),
                ExecArgument::File => {
                    if let Some(value) = context.files.first() {
                        arguments.push(value.to_string_lossy().into_owned());
                    }
                }
                ExecArgument::Files => arguments.extend(
                    context
                        .files
                        .iter()
                        .map(|value| value.to_string_lossy().into_owned()),
                ),
                ExecArgument::Url => {
                    if let Some(value) = context.urls.first() {
                        arguments.push(value.clone());
                    }
                }
                ExecArgument::Urls => arguments.extend(context.urls.iter().cloned()),
                ExecArgument::Icon => {
                    if let Some(value) = icon.filter(|value| !value.is_empty()) {
                        arguments.push("--icon".to_owned());
                        arguments.push(value.to_owned());
                    }
                }
                ExecArgument::Name => arguments.push(name.to_owned()),
                ExecArgument::DesktopFile => {
                    arguments.push(desktop_file.to_string_lossy().into_owned())
                }
            }
        }
        LaunchCommand {
            program: self.program.clone(),
            arguments,
        }
    }
}

/// Desktop Entry quoting is intentionally not POSIX shell quoting. Single quotes
/// are ordinary characters; only double quotes group an argument.
fn tokenize(raw: &str) -> Result<Vec<Token>, ExecError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut was_quoted = false;
    let mut escaped = false;
    let mut active = false;

    for character in raw.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            active = true;
            continue;
        }
        match character {
            '\\' => {
                escaped = true;
                active = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                was_quoted = true;
                active = true;
            }
            character if character.is_ascii_whitespace() && !in_quotes => {
                if active {
                    tokens.push(Token {
                        value: std::mem::take(&mut current),
                        was_quoted,
                    });
                    was_quoted = false;
                    active = false;
                }
            }
            _ => {
                current.push(character);
                active = true;
            }
        }
    }

    if escaped {
        return Err(ExecError::TrailingEscape);
    }
    if in_quotes {
        return Err(ExecError::UnterminatedQuote);
    }
    if active {
        tokens.push(Token {
            value: current,
            was_quoted,
        });
    }
    Ok(tokens)
}

fn parse_argument(token: &Token, file_fields: &mut usize) -> Result<Vec<ExecArgument>, ExecError> {
    if token.was_quoted && contains_field_code(&token.value) {
        return Err(ExecError::FieldCodeInsideQuotedArgument);
    }

    if token.value.len() == 2 && token.value.starts_with('%') {
        let code = token
            .value
            .chars()
            .nth(1)
            .expect("two-character field code");
        let value = match code {
            'f' => {
                *file_fields += 1;
                Some(ExecArgument::File)
            }
            'F' => {
                *file_fields += 1;
                Some(ExecArgument::Files)
            }
            'u' => {
                *file_fields += 1;
                Some(ExecArgument::Url)
            }
            'U' => {
                *file_fields += 1;
                Some(ExecArgument::Urls)
            }
            'i' => Some(ExecArgument::Icon),
            'c' => Some(ExecArgument::Name),
            'k' => Some(ExecArgument::DesktopFile),
            'd' | 'D' | 'n' | 'N' | 'v' | 'm' => None,
            '%' => Some(ExecArgument::Literal("%".to_owned())),
            other => return Err(ExecError::UnknownFieldCode(other)),
        };
        return Ok(value.into_iter().collect());
    }

    for pair in percent_codes(&token.value)? {
        match pair {
            'F' | 'U' | 'i' => return Err(ExecError::FieldCodeMustBeStandalone(pair)),
            'f' | 'u' | 'c' | 'k' => return Err(ExecError::FieldCodeMustBeStandalone(pair)),
            'd' | 'D' | 'n' | 'N' | 'v' | 'm' => {
                return Err(ExecError::FieldCodeMustBeStandalone(pair));
            }
            '%' => {}
            other => return Err(ExecError::UnknownFieldCode(other)),
        }
    }
    Ok(vec![ExecArgument::Literal(unescape_literal_percent(
        &token.value,
    )?)])
}

fn contains_field_code(value: &str) -> bool {
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '%' && chars.peek().is_some() {
            return true;
        }
    }
    false
}

fn percent_codes(value: &str) -> Result<Vec<char>, ExecError> {
    let mut result = Vec::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '%' {
            continue;
        }
        let Some(code) = chars.next() else {
            return Err(ExecError::UnknownFieldCode('%'));
        };
        result.push(code);
    }
    Ok(result)
}

fn unescape_literal_percent(value: &str) -> Result<String, ExecError> {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '%' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('%') => output.push('%'),
            Some(code) => return Err(ExecError::UnknownFieldCode(code)),
            None => return Err(ExecError::UnknownFieldCode('%')),
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semicolon_is_plain_argv_data_not_shell_syntax() {
        let template = ExecTemplate::parse("printf %%s a;touch-pwned").unwrap();
        let command = template.expand(
            &LaunchContext::default(),
            "Demo",
            None,
            Path::new("x.desktop"),
        );
        assert_eq!(command.program, "printf");
        assert_eq!(command.arguments, ["%s", "a;touch-pwned"]);
    }

    #[test]
    fn list_field_expands_to_distinct_arguments() {
        let template = ExecTemplate::parse("viewer %F").unwrap();
        let context = LaunchContext {
            files: vec![PathBuf::from("/tmp/a b"), PathBuf::from("/tmp/c")],
            urls: Vec::new(),
        };
        let command = template.expand(&context, "Viewer", None, Path::new("viewer.desktop"));
        assert_eq!(command.arguments, ["/tmp/a b", "/tmp/c"]);
    }

    #[test]
    fn unknown_field_code_invalidates_template() {
        assert_eq!(
            ExecTemplate::parse("demo %Z").unwrap_err(),
            ExecError::UnknownFieldCode('Z')
        );
    }

    #[test]
    fn field_code_inside_quotes_is_rejected() {
        assert_eq!(
            ExecTemplate::parse("demo \"%u\"").unwrap_err(),
            ExecError::FieldCodeInsideQuotedArgument
        );
    }

    #[test]
    fn literal_percent_is_preserved() {
        let template = ExecTemplate::parse("demo 100%%").unwrap();
        assert_eq!(
            template.arguments(),
            &[ExecArgument::Literal("100%".into())]
        );
    }
}
