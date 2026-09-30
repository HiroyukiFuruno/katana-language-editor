use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use serde_json::Value;

use super::super::operational_input::SourceClosureInput;
use super::super::operational_loader::SourceClosureInputLoaderVerifier;
use super::operational_fixture::Fixture;

pub(super) type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub(super) fn test_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(io::Error::other(message.into()))
}

pub(super) fn string_result<T>(result: Result<T, String>) -> TestResult<T> {
    result.map_err(test_error)
}

pub(super) fn option_result<T>(value: Option<T>, message: impl Into<String>) -> TestResult<T> {
    value.ok_or_else(|| test_error(message.into()))
}

pub(super) fn cleanup_dir(path: impl AsRef<Path>) -> TestResult {
    fs::remove_dir_all(path.as_ref())?;
    Ok(())
}

pub(super) fn mutate_input(
    fixture: &Fixture,
    mutator: impl FnOnce(&mut Value) -> TestResult,
) -> TestResult {
    let mut value: Value = serde_json::from_slice(&fs::read(&fixture.input_json)?)?;
    mutator(&mut value)?;
    let bytes = match serde_json::from_value::<SourceClosureInput>(value.clone()) {
        Ok(input) => string_result(input.canonical_json_bytes())?,
        Err(_) => {
            let mut bytes = serde_json::to_vec_pretty(&value)?;
            bytes.push(b'\n');
            bytes
        }
    };
    fs::write(&fixture.input_json, bytes)?;
    Ok(())
}

pub(super) fn reject(fixture: Fixture) -> TestResult {
    let result = SourceClosureInputLoaderVerifier::load(&fixture.input_json);
    cleanup_dir(&fixture.root)?;
    match result {
        Ok(_) => Err(test_error("invalid operational input was accepted")),
        Err(_) => Ok(()),
    }
}
