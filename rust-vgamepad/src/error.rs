//! 虚拟游戏手柄错误类型定义
//! 
//! 使用thiserror进行结构化错误定义，便于库用户处理特定错误类型

use thiserror::Error;

/// 虚拟游戏手柄库的Result类型
pub type Result<T> = std::result::Result<T, VGamepadError>;

/// 虚拟游戏手柄错误类型
/// 
/// 这是库层面的结构化错误，使用thiserror定义
/// 可以转换为anyhow::Error供应用层使用
#[derive(Error, Debug)]
pub enum VGamepadError {
    /// ViGEm驱动程序错误 (Windows)
    #[error("ViGEm驱动程序错误: {message} (错误代码: 0x{code:08X})")]
    ViGEmError { message: String, code: u32 },

    /// ViGEm库加载失败
    #[error("无法加载ViGEmClient.dll: {reason}")]
    ViGEmLibraryError { reason: String },

    /// ViGEm函数调用失败
    #[error("ViGEm函数 '{function}' 调用失败: {reason}")]
    ViGEmFunctionError { function: String, reason: String },

    /// IOKit HID错误 (macOS)
    #[error("IOKit HID错误: {message} (状态码: {status})")]
    IOKitError { message: String, status: i32 },

    /// 控制器初始化失败
    #[error("控制器初始化失败: {reason}")]
    ControllerInitError { reason: String },

    /// 控制器连接失败
    #[error("控制器连接失败: {reason}")]
    ControllerConnectionError { reason: String },

    /// 控制器已断开连接
    #[error("控制器已断开连接")]
    ControllerDisconnected,

    /// 控制器状态更新失败
    #[error("控制器状态更新失败: {reason}")]
    ControllerUpdateError { reason: String },

    /// 无效的输入值
    #[error("无效的输入值 {field}: 期望 {expected}，实际 {actual}")]
    InvalidInput {
        field: String,
        expected: String,
        actual: String,
    },

    /// 平台不支持
    #[error("当前平台 '{platform}' 不支持功能: {feature}")]
    UnsupportedPlatform { platform: String, feature: String },

    /// 驱动程序未安装
    #[error("所需驱动程序未安装: {driver} (请访问 {download_url})")]
    DriverNotInstalled { driver: String, download_url: String },

    /// 权限不足
    #[error("权限不足: {operation} 需要管理员权限")]
    InsufficientPermissions { operation: String },

    /// 系统错误
    #[error("系统错误")]
    SystemError(#[from] std::io::Error),
}

impl VGamepadError {
    /// 创建ViGEm错误
    pub fn vigem_error(message: impl Into<String>, code: u32) -> Self {
        Self::ViGEmError {
            message: message.into(),
            code,
        }
    }

    /// 创建ViGEm库加载错误
    pub fn vigem_library_error(reason: impl Into<String>) -> Self {
        Self::ViGEmLibraryError {
            reason: reason.into(),
        }
    }

    /// 创建ViGEm函数调用错误
    pub fn vigem_function_error(function: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ViGEmFunctionError {
            function: function.into(),
            reason: reason.into(),
        }
    }

    /// 创建IOKit错误
    pub fn iokit_error(message: impl Into<String>, status: i32) -> Self {
        Self::IOKitError {
            message: message.into(),
            status,
        }
    }

    /// 创建控制器初始化错误
    pub fn controller_init_error(reason: impl Into<String>) -> Self {
        Self::ControllerInitError {
            reason: reason.into(),
        }
    }

    /// 创建控制器连接错误
    pub fn controller_connection_error(reason: impl Into<String>) -> Self {
        Self::ControllerConnectionError {
            reason: reason.into(),
        }
    }

    /// 创建控制器更新错误
    pub fn controller_update_error(reason: impl Into<String>) -> Self {
        Self::ControllerUpdateError {
            reason: reason.into(),
        }
    }

    /// 创建无效输入错误
    pub fn invalid_input(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::InvalidInput {
            field: field.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// 创建不支持平台错误
    pub fn unsupported_platform(platform: impl Into<String>, feature: impl Into<String>) -> Self {
        Self::UnsupportedPlatform {
            platform: platform.into(),
            feature: feature.into(),
        }
    }

    /// 创建驱动程序未安装错误
    pub fn driver_not_installed(driver: impl Into<String>, download_url: impl Into<String>) -> Self {
        Self::DriverNotInstalled {
            driver: driver.into(),
            download_url: download_url.into(),
        }
    }

    /// 创建权限不足错误
    pub fn insufficient_permissions(operation: impl Into<String>) -> Self {
        Self::InsufficientPermissions {
            operation: operation.into(),
        }
    }

    /// 检查是否为ViGEm相关错误
    pub fn is_vigem_error(&self) -> bool {
        matches!(
            self,
            Self::ViGEmError { .. }
                | Self::ViGEmLibraryError { .. }
                | Self::ViGEmFunctionError { .. }
        )
    }

    /// 检查是否为IOKit相关错误
    pub fn is_iokit_error(&self) -> bool {
        matches!(self, Self::IOKitError { .. })
    }

    /// 检查是否为可恢复的错误
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ControllerUpdateError { .. } | Self::InvalidInput { .. }
        )
    }
}