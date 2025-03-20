use std::collections::HashMap;
use std::env;

pub(crate) fn build_stamp() -> String {
    env::var("VERGEN_BUILD_TIMESTAMP")
        .unwrap_or_else(|_| String::from(""))
}

pub(crate) fn git_commit_stamp() -> String {
    env::var("VERGEN_GIT_COMMIT_TIMESTAMP").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn git_commit_id() -> String {
    env::var("VERGEN_GIT_SHA").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn git_branch() -> String {
    env::var("VERGEN_GIT_BRANCH").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn rustc_version() -> String {
    env::var("VERGEN_RUSTC_SEMVER").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn cargo_profile() -> String {
    env::var("VERGEN_CARGO_PROFILE").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn os() -> String {
    env::var("VERGEN_SYSINFO_OS_VERSION").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn cpu() -> String {
    env::var("VERGEN_SYSINFO_CPU_BRAND").unwrap_or_else(|_| String::from(""))
}

pub(crate) fn envs() -> HashMap<String, String> {
    env::vars().collect()
}
