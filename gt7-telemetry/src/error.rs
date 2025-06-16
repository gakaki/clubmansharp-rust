//! GT7遥测错误类型定义
//! 
//! 使用thiserror进行结构化错误定义，提供详细的错误信息

use thiserror::Error;
use std::net::AddrParseError;

/// GT7遥测库的Result类型
pub type Result<T> = std::result::Result<T, GT7Error>;

/// GT7遥测错误类型
/// 
/// 这是库层面的结构化错误，使用thiserror定义
/// 可以转换为anyhow::Error供应用层使用
#[derive(Error, Debug)]
pub enum GT7Error {
    /// 网络连接错误
    #[error("网络连接到 {address} 失败: {reason}")]
    NetworkError { address: String, reason: String },

    /// UDP套接字错误  
    #[error("UDP套接字错误")]
    SocketError(#[from] std::io::Error),

    /// 地址解析错误
    #[error("IP地址解析错误")]
    AddressParseError(#[from] AddrParseError),

    /// 数据包解析错误
    #[error("数据包解析错误: {message} (偏移: {offset}, 长度: {length})")]
    PacketParseError {
        message: String,
        offset: usize,
        length: usize,
    },

    /// 数据包版本不匹配
    #[error("数据包版本不匹配: 期望 {expected}，实际 {actual}")]
    PacketVersionMismatch { expected: u16, actual: u16 },

    /// 无效的IP地址
    #[error("无效的IP地址: {ip} (必须是有效的IPv4地址)")]
    InvalidIPAddress { ip: String },

    /// 无效的端口
    #[error("无效的端口: {port} (有效范围: 1-65535)")]
    InvalidPort { port: u16 },

    /// 超时错误
    #[error("操作 '{operation}' 超时 ({timeout_ms}ms)")]
    TimeoutError { operation: String, timeout_ms: u64 },

    /// 游戏未连接
    #[error("游戏未连接或数据不可用 (最后心跳: {last_heartbeat})")]
    GameNotConnected { last_heartbeat: String },

    /// 数据不完整
    #[error("接收到的数据不完整: 期望 {expected} 字节，实际 {actual} 字节")]
    IncompleteData { expected: usize, actual: usize },

    /// 校验和错误
    #[error("数据包校验和错误: 计算值 0x{calculated:04X}，期望值 0x{expected:04X}")]
    ChecksumError { calculated: u16, expected: u16 },

    /// 数据包格式错误
    #[error("数据包格式错误: {field} 字段无效")]
    InvalidPacketFormat { field: String },

    /// 游戏状态错误
    #[error("游戏状态错误: 当前状态 {current_state} 不支持操作 {operation}")]
    InvalidGameState {
        current_state: String,
        operation: String,
    },

    /// 配置错误
    #[error("配置错误: {field} = {value} (原因: {reason})")]
    ConfigError {
        field: String,
        value: String,
        reason: String,
    },

    /// 文件I/O错误
    #[error("文件操作错误: {operation} 失败")]
    FileError { operation: String },

    /// 序列化错误
    #[error("序列化错误")]
    SerializationError(#[from] serde_json::Error),

    /// 多客户端错误
    #[error("多客户端管理错误: {message}")]
    MultiClientError { message: String },
}

impl GT7Error {
    /// 创建网络错误
    pub fn network_error(address: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::NetworkError {
            address: address.into(),
            reason: reason.into(),
        }
    }

    /// 创建数据包解析错误
    pub fn packet_parse_error(
        message: impl Into<String>,
        offset: usize,
        length: usize,
    ) -> Self {
        Self::PacketParseError {
            message: message.into(),
            offset,
            length,
        }
    }

    /// 创建数据包版本错误
    pub fn packet_version_mismatch(expected: u16, actual: u16) -> Self {
        Self::PacketVersionMismatch { expected, actual }
    }

    /// 创建无效IP地址错误
    pub fn invalid_ip(ip: impl Into<String>) -> Self {
        Self::InvalidIPAddress { ip: ip.into() }
    }

    /// 创建无效端口错误
    pub fn invalid_port(port: u16) -> Self {
        Self::InvalidPort { port }
    }

    /// 创建超时错误
    pub fn timeout_error(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// 创建游戏未连接错误
    pub fn game_not_connected(last_heartbeat: impl Into<String>) -> Self {
        Self::GameNotConnected {
            last_heartbeat: last_heartbeat.into(),
        }
    }

    /// 创建数据不完整错误
    pub fn incomplete_data(expected: usize, actual: usize) -> Self {
        Self::IncompleteData { expected, actual }
    }

    /// 创建校验和错误
    pub fn checksum_error(calculated: u16, expected: u16) -> Self {
        Self::ChecksumError {
            calculated,
            expected,
        }
    }

    /// 创建数据包格式错误
    pub fn invalid_packet_format(field: impl Into<String>) -> Self {
        Self::InvalidPacketFormat {
            field: field.into(),
        }
    }

    /// 创建游戏状态错误
    pub fn invalid_game_state(
        current_state: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::InvalidGameState {
            current_state: current_state.into(),
            operation: operation.into(),
        }
    }

    /// 创建配置错误
    pub fn config_error(
        field: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ConfigError {
            field: field.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// 创建文件错误
    pub fn file_error(operation: impl Into<String>) -> Self {
        Self::FileError {
            operation: operation.into(),
        }
    }

    /// 创建多客户端错误
    pub fn multi_client_error(message: impl Into<String>) -> Self {
        Self::MultiClientError {
            message: message.into(),
        }
    }

    /// 检查是否为网络相关错误
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            Self::NetworkError { .. } | Self::SocketError(_) | Self::AddressParseError(_)
        )
    }

    /// 检查是否为数据包相关错误
    pub fn is_packet_error(&self) -> bool {
        matches!(
            self,
            Self::PacketParseError { .. }
                | Self::PacketVersionMismatch { .. }
                | Self::ChecksumError { .. }
                | Self::InvalidPacketFormat { .. }
                | Self::IncompleteData { .. }
        )
    }

    /// 检查是否为可恢复的错误
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::TimeoutError { .. }
                | Self::GameNotConnected { .. }
                | Self::IncompleteData { .. }
                | Self::NetworkError { .. }
        )
    }

    /// 检查是否为配置错误
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            Self::ConfigError { .. }
                | Self::InvalidIPAddress { .. }
                | Self::InvalidPort { .. }
        )
    }
}