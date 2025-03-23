use std::collections::HashMap;
use std::env;
use std::rc::Rc;

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

pub(crate) fn envs() -> Rc<HashMap<String, String>> {
    Rc::new(env::vars().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper function to set environment variables for testing
    fn set_env_var(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    // Helper function to remove environment variables
    fn remove_env_var(key: &str) {
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_build_stamp_when_env_var_exists() {
        set_env_var("VERGEN_BUILD_TIMESTAMP", "2023-04-15T14:30:45Z");
        assert_eq!(build_stamp(), "2023-04-15T14:30:45Z");
        remove_env_var("VERGEN_BUILD_TIMESTAMP");
    }

    #[test]
    fn test_build_stamp_when_env_var_missing() {
        remove_env_var("VERGEN_BUILD_TIMESTAMP");
        assert_eq!(build_stamp(), "");
    }

    #[test]
    fn test_git_commit_stamp_when_env_var_exists() {
        set_env_var("VERGEN_GIT_COMMIT_TIMESTAMP", "2023-04-14T10:20:30Z");
        assert_eq!(git_commit_stamp(), "2023-04-14T10:20:30Z");
        remove_env_var("VERGEN_GIT_COMMIT_TIMESTAMP");
    }

    #[test]
    fn test_git_commit_stamp_when_env_var_missing() {
        remove_env_var("VERGEN_GIT_COMMIT_TIMESTAMP");
        assert_eq!(git_commit_stamp(), "");
    }

    #[test]
    fn test_git_commit_id_when_env_var_exists() {
        set_env_var("VERGEN_GIT_SHA", "abc123def456");
        assert_eq!(git_commit_id(), "abc123def456");
        remove_env_var("VERGEN_GIT_SHA");
    }

    #[test]
    fn test_git_commit_id_when_env_var_missing() {
        remove_env_var("VERGEN_GIT_SHA");
        assert_eq!(git_commit_id(), "");
    }

    #[test]
    fn test_git_branch_when_env_var_exists() {
        set_env_var("VERGEN_GIT_BRANCH", "main");
        assert_eq!(git_branch(), "main");
        remove_env_var("VERGEN_GIT_BRANCH");
    }

    #[test]
    fn test_git_branch_when_env_var_missing() {
        remove_env_var("VERGEN_GIT_BRANCH");
        assert_eq!(git_branch(), "");
    }

    #[test]
    fn test_rustc_version_when_env_var_exists() {
        set_env_var("VERGEN_RUSTC_SEMVER", "1.68.0");
        assert_eq!(rustc_version(), "1.68.0");
        remove_env_var("VERGEN_RUSTC_SEMVER");
    }

    #[test]
    fn test_rustc_version_when_env_var_missing() {
        remove_env_var("VERGEN_RUSTC_SEMVER");
        assert_eq!(rustc_version(), "");
    }

    #[test]
    fn test_cargo_profile_when_env_var_exists() {
        set_env_var("VERGEN_CARGO_PROFILE", "release");
        assert_eq!(cargo_profile(), "release");
        remove_env_var("VERGEN_CARGO_PROFILE");
    }

    #[test]
    fn test_cargo_profile_when_env_var_missing() {
        remove_env_var("VERGEN_CARGO_PROFILE");
        assert_eq!(cargo_profile(), "");
    }

    #[test]
    fn test_os_when_env_var_exists() {
        set_env_var("VERGEN_SYSINFO_OS_VERSION", "macOS 13.4");
        assert_eq!(os(), "macOS 13.4");
        remove_env_var("VERGEN_SYSINFO_OS_VERSION");
    }

    #[test]
    fn test_os_when_env_var_missing() {
        remove_env_var("VERGEN_SYSINFO_OS_VERSION");
        assert_eq!(os(), "");
    }

    #[test]
    fn test_cpu_when_env_var_exists() {
        set_env_var("VERGEN_SYSINFO_CPU_BRAND", "Intel Core i7-9750H");
        assert_eq!(cpu(), "Intel Core i7-9750H");
        remove_env_var("VERGEN_SYSINFO_CPU_BRAND");
    }

    #[test]
    fn test_cpu_when_env_var_missing() {
        remove_env_var("VERGEN_SYSINFO_CPU_BRAND");
        assert_eq!(cpu(), "");
    }

    #[test]
    fn test_envs_contains_expected_values() {
        set_env_var("TEST_KEY1", "test_value1");
        set_env_var("TEST_KEY2", "test_value2");

        let env_map = envs();

        assert!(env_map.contains_key("TEST_KEY1"));
        assert!(env_map.contains_key("TEST_KEY2"));
        assert_eq!(env_map.get("TEST_KEY1").unwrap(), "test_value1");
        assert_eq!(env_map.get("TEST_KEY2").unwrap(), "test_value2");

        remove_env_var("TEST_KEY1");
        remove_env_var("TEST_KEY2");
    }

    #[test]
    fn test_envs_returns_rc_hashmap() {
        let env_map = envs();
        assert!(Rc::strong_count(&env_map) >= 1);

        // Create a clone to verify Rc works correctly
        let env_map_clone = Rc::clone(&env_map);
        assert!(Rc::strong_count(&env_map) >= 2);
        assert_eq!(env_map.len(), env_map_clone.len());
    }
}
