use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

use color_eyre::eyre::{self, ensure, eyre};

#[derive(serde::Deserialize)]
struct ExecutionParams {
    binary: PathBuf,
    environment: HashMap<String, String>,
}

fn get_sysroot(rustc: &str) -> eyre::Result<String> {
    let res = Command::new(rustc)
        .arg("--print=sysroot")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    ensure!(res.status.success(), "failed to determine sysroot");
    Ok(String::from_utf8(res.stdout)?)
}

fn main() -> eyre::Result<ExitCode> {
    color_eyre::install()?;

    let linter_dir = env::var("CARGO_NAG_LINTER_DIR")
        .map_err(|_| eyre!("CARGO_NAG_LINTER_DIR must be set for `cargo nag` to work"))?;
    let linter_manifest_path = PathBuf::from(linter_dir).join("Cargo.toml");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());

    let cargo_args: Vec<_> = env::args().skip(2).collect();
    let sysroot = get_sysroot(&rustc)?;

    #[rustfmt::skip]
    let res = Command::new(&cargo)
        .args(["run", "--quiet", "--manifest-path"]).arg(&linter_manifest_path)
        .env("CARGO_NAG_DUMP_EXECUTION_PARAMS", "please")
        .env("RUSTC_BOOTSTRAP", "1")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if !res.status.success() {
        return Ok(ExitCode::FAILURE);
    }

    let params: ExecutionParams = serde_json::from_slice(&res.stdout)?;

    let mut command = Command::new(cargo);
    command
        .args(["check"])
        .args(cargo_args)
        .env("RUSTC_WORKSPACE_WRAPPER", params.binary)
        .env("CARGO_NAG_SYSROOT", sysroot.trim());

    // TODO: something else?
    for env in ["LD_LIBRARY_PATH"] {
        if let Some(val) = params.environment.get(env) {
            command.env(env, val);
        }
    }

    let status = command.spawn()?.wait()?;

    if status.success() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}
