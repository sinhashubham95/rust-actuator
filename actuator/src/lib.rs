mod env;

use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use crate::env::{build_stamp, cpu, envs, git_branch, git_commit_id,
                 git_commit_stamp, os, rustc_version};
use sysinfo::{System};

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
struct Health {
    last_check_stamp: SystemTime,
    data: HashMap<String, HealthInfo>,
}

#[derive(Debug,Clone)]
struct InnerHealth(Arc<Mutex<Health>>);

#[derive(Debug,Clone)]
pub struct Metrics {
    total_memory: u64,
    used_memory: u64,
    available_memory: u64,
    free_memory: u64,
    total_swap: u64,
    used_swap: u64,
    free_swap: u64,
    global_cpu_usage: f32,
}

#[derive(Debug,Clone)]
struct Inner {
    cfg: *Config,
    health: InnerHealth,
    info: Rc<Info>,
    envs: Rc<HashMap<String, String>>,
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
            info: Rc::new(Info::new(cfg)),
            envs: envs(),
        })
    }

    pub fn ping(self) -> bool {
        true
    }

    pub fn info(self) -> Rc<Info> {
        self.0.info
    }

    pub fn health(self) -> Rc<HashMap<String, HealthInfo>> {
        Rc::new(HashMap::new())
    }

    pub fn env(self) -> Rc<HashMap<String, String>> {
        self.0.envs
    }

    pub fn metrics(self) -> Rc<Metrics> {
        let mut sys = System::new_all();
        sys.refresh_all();
        Rc::new(Metrics{
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            available_memory: sys.available_memory(),
            free_memory: sys.free_memory(),
            total_swap: sys.total_swap(),
            used_swap: sys.used_swap(),
            free_swap: sys.free_swap(),
            global_cpu_usage: sys.global_cpu_usage(),
        })
    }

    pub fn shutdown(self) {}

    pub fn thread_dump(self) {}
}
