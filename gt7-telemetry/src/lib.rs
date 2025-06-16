//! # GT7 遥测数据库
//! 
//! 参考gt7telemetry Python库实现，用于解析Gran Turismo 7的遥测数据
//! 支持实时监控游戏状态、车辆信息、赛道情况等

pub mod error;
pub mod packet;
pub mod client;
pub mod types;

pub use error::{GT7Error, Result};
pub use packet::{GT7TelemetryPacket, GameState, CarInfo, TrackInfo};
pub use client::GT7TelemetryClient;
pub use types::*;

/// GT7默认遥测端口 (参考gt7telemetry)
pub const GT7_TELEMETRY_PORT: u16 = 33740;

/// GT7遥测数据包大小 (参考gt7telemetry)
pub const GT7_PACKET_SIZE: usize = 296;

/// GT7发送心跳包的魔术字节 (参考gt7telemetry)
pub const GT7_HEARTBEAT: &[u8] = b"A";

/// GT7 IP地址范围验证
pub fn is_valid_gt7_ip(ip: &str) -> bool {
    // GT7通常在局域网内运行
    ip.starts_with("192.168.") || 
    ip.starts_with("10.") || 
    ip.starts_with("172.") ||
    ip == "127.0.0.1"
}