use self::CPUWorkersConfig::{Count, Single};
use super::errors::ZeroPoolSizeError;

#[derive(Debug, Clone, Copy)]
enum CPUWorkersConfig {
    Single,
    Count(u64),
    Auto,
}

pub struct CPU {
    workers_config: CPUWorkersConfig
    // pool_manager: Option<...>
}

// TODO: refactor dyn errors
impl CPU {
    // TODO: check runtime options
    pub fn with_worker_pool(num: u64) -> Result<CPU, ZeroPoolSizeError> {
        if num == 0 {
            Err(ZeroPoolSizeError{})
        } else {
            Ok(CPU { workers_config: Count(num) })
        }
    }

    pub fn with_auto() -> Result<CPU, ZeroPoolSizeError> {
        todo!()
    }

    pub fn with_single_thread() -> CPU { CPU{workers_config: Single } }
}

pub struct GPU {}
