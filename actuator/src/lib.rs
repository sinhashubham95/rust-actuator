use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

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
    data: HashMap<String, Rc<HealthInfo>>,
}

#[derive(Debug,Clone)]
struct InnerHealth(Arc<Mutex<Health>>);

#[derive(Debug,Clone)]
struct Inner {
    cfg: Config,
    health: InnerHealth
}

#[derive(Debug)]
pub struct Actuator(Rc<Inner>);
