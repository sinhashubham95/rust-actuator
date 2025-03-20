mod env;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use crate::env::{build_stamp, cpu, git_branch, git_commit_id, git_commit_stamp, os, rustc_version};

pub enum Endpoint {
    Ping,
    Info,
    Health,
    Env,
    Metrics,
    Shutdown,
    ThreadDump,
}

pub type HealthCheckFn = dyn Fn() -> Box<dyn Future<Output = Result<(), Err>> + Send> + Send;

#[derive(Debug, Clone)]
pub struct HealthChecker {
    key: String,
    is_mandatory: bool,
    func: HealthCheckFn,
}

#[derive(Debug, Clone)]
pub struct HealthConfig {
    cache_duration: Duration,
    timeout: Duration,
    checkers: [HealthChecker],
}

#[derive(Debug, Clone)]
pub struct Config {
    endpoints: [Endpoint],
    env: String,
    name: String,
    port: u16,
    version: String,
    health: HealthConfig,
}

#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    name: String,
    env: String,
    version: String,
    startup_stamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct GITInfo {
    build_stamp: String,
    commit_id: String,
    commit_stamp: String,
    primary_branch: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeInfo {
    arch: String,
    os: String,
    port: u16,
    version: String,
}

#[derive(Debug, Clone)]
pub struct Info {
    application: ApplicationInfo,
    git: GITInfo,
    runtime: RuntimeInfo,
}

#[derive(Debug,Clone)]
pub struct HealthInfo {
    key: String,
    is_mandatory: bool,
    success: bool,
    error: String,
}

#[derive(Debug,Clone)]
pub struct Health {
    last_check_stamp: SystemTime,
    data: HashMap<String, HealthInfo>,
}

#[derive(Debug,Clone)]
struct InnerHealth(Arc<Mutex<Health>>);

#[derive(Debug,Clone)]
struct Inner {
    cfg: *Config,
    health: InnerHealth,
    info: Info,
}

#[derive(Debug)]
pub struct Actuator(Inner);

impl Info {
    fn new(cfg: &Config) -> Info {
        Info{
            application: ApplicationInfo{
                name: cfg.name.clone(),
                env: cfg.env.clone(),
                version: cfg.version.clone(),
                startup_stamp: SystemTime::now(),
            },
            git: GITInfo{
                build_stamp: build_stamp(),
                commit_id: git_commit_id(),
                commit_stamp: git_commit_stamp(),
                primary_branch: git_branch(),
            },
            runtime: RuntimeInfo{
                arch: cpu(),
                os: os(),
                port: cfg.port,
                version: rustc_version(),
            },
        }
    }
}

impl InnerHealth {
    fn new() -> InnerHealth {
        InnerHealth(Arc::new(Mutex::new(Health{
            last_check_stamp: SystemTime::UNIX_EPOCH,
            data: HashMap::new(),
        })))
    }
}

impl Actuator {
    pub fn new(cfg: &Config) -> Actuator {
        Actuator(Inner{
            cfg,
            health: InnerHealth::new(),
            info: Info::new(cfg),
        })
    }

    pub fn info(self) -> Info {
        self.0.info
    }

    pub fn health(self) -> HashMap<String, HealthInfo> {
        HashMap::new()
    }
}
