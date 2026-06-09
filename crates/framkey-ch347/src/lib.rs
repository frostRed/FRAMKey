mod config;
mod device;
mod flashrom;
mod temp;

pub use config::{Ch347Config, Ch347SpiSpeed};
pub use device::{Ch347Device, Ch347WriteVerifyReport};

#[cfg(test)]
mod tests;
