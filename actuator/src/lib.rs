mod env;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::pin::Pin;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use crate::env::{build_stamp, cargo_profile, cpu, envs, git_branch, git_commit_id, git_commit_stamp, os, rustc_version};
use sysinfo::{System};
use backtrace::Backtrace;
use futures::future::join_all;

#[derive(Debug)]
struct ActuatorError {
    details: String,
}

#[derive(Debug, Clone)]
pub enum Endpoint {
    Ping,
    Info,
    Health,
    Env,
    Metrics,
    Shutdown,
    ThreadDump,
}

pub type HealthCheckFn<E> = fn() -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>>;

#[derive(Debug, Clone)]
pub struct HealthChecker {
    key: String,
    is_mandatory: bool,
    func: HealthCheckFn<ActuatorError>,
}

#[derive(Debug, Clone)]
pub struct HealthConfig {
    cache_duration: Duration,
    timeout: Duration,
    checkers: Box<[HealthChecker]>,
}

#[derive(Debug, Clone)]
pub struct Config {
    endpoints: Box<[Endpoint]>,
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
    cargo_version: String,
}

#[derive(Debug, Clone)]
pub struct Info {
    application: ApplicationInfo,
    git: GITInfo,
    runtime: RuntimeInfo,
}

#[derive(Debug, Clone)]
pub struct HealthInfo {
    key: String,
    is_mandatory: bool,
    success: bool,
    error: String,
}

#[derive(Debug, Clone)]
struct Health {
    last_check_stamp: SystemTime,
    data: HashMap<String, HealthInfo>,
}

#[derive(Debug, Clone)]
struct InnerHealth{
    cfg: HealthConfig,
    health: Arc<RwLock<Health>>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct Inner {
    cfg: Rc<Config>,
    health: InnerHealth,
    info: Rc<Info>,
    envs: Rc<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct Actuator(Inner);

impl Display for ActuatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActuatorError: {}", self.details)
    }
}

impl Default for ActuatorError {
    fn default() -> Self {
        Self {
            details: "actuator error".into(),
        }
    }
}

impl Error for ActuatorError {}

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
                cargo_version: cargo_profile(),
            },
        }
    }
}

impl InnerHealth {
    fn new(cfg: &Config) -> InnerHealth {
        InnerHealth{
            cfg: cfg.health.clone(),
            health: Arc::new(RwLock::new(Health{
                last_check_stamp: SystemTime::UNIX_EPOCH,
                data: HashMap::new(),
            }))
        }
    }

    fn get_from_cache(&self) -> Option<Rc<HashMap<String, HealthInfo>>> {
        let health = self.health.read().unwrap();
        if SystemTime::now().duration_since(health.last_check_stamp).
            unwrap_or_else(|_| Duration::MAX) <= self.cfg.cache_duration {
            Some(Rc::new(health.data.clone()))
        } else {
            None
        }
    }

    async fn get_health_and_cache_if_success(&self) -> (Rc<HashMap<String, HealthInfo>>, bool) {
        let mut tasks = vec![];
        for checker in self.cfg.checkers.iter() {
            let key = checker.key.clone();
            let is_mandatory = checker.is_mandatory;
            let fut = (checker.func)();
            tasks.push(async move {
                let result = fut.await;
                HealthInfo {
                    key,
                    is_mandatory,
                    success: result.is_ok(),
                    error: result.err().unwrap().details,
                }
            });
        }
        let results: Vec<HealthInfo> = join_all(tasks).await;
        let new_data: HashMap<String, HealthInfo> = results.into_iter()
            .map(|info| (info.key.clone(), info))
            .collect();
        let ok = new_data
            .values()
            .filter(|info| info.is_mandatory)
            .all(|info| info.success);
        (Rc::new(new_data), ok)
    }

    async fn get(&self) -> (Rc<HashMap<String, HealthInfo>>, bool) {
        match self.get_from_cache() {
            Some(health) => (health, true),
            None => self.get_health_and_cache_if_success().await,
        }
    }
}

impl Actuator {
    pub fn new(cfg: &Config) -> Actuator {
        Actuator(Inner{
            cfg: Rc::new(cfg.clone()),
            health: InnerHealth::new(cfg),
            info: Rc::new(Info::new(cfg)),
            envs: envs(),
        })
    }

    pub fn ping(&self) -> bool {
        true
    }

    pub fn info(&self) -> Rc<Info> {
        self.0.info.clone()
    }

    pub async fn health(&self) -> (Rc<HashMap<String, HealthInfo>>, bool) {
        self.0.health.get().await
    }

    pub fn env(&self) -> Rc<HashMap<String, String>> {
        self.0.envs.clone()
    }

    pub fn metrics(&self) -> Rc<Metrics> {
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

    pub fn shutdown(&self) {
        process::exit(0)
    }

    pub fn thread_dump(&self) -> String {
        format!("{:?}", Backtrace::new())
    }
}
