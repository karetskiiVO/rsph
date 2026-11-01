use thiserror::Error;

#[derive(Debug, Error)]
#[error("Размер пула воркеров не может быть равен нулю")]
pub struct ZeroPoolSizeError;
