//! # rust-vgamepad
//! 
//! 虚拟游戏手柄库，参考Python vgamepad实现
//! 支持Windows (ViGEm) 和 macOS (IOKit HID)
//! 专为DualShock4控制器设计

pub mod error;
pub mod controller;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

pub use error::{VGamepadError, Result};
pub use controller::{
    DualShock4Controller, 
    DS4Button, 
    DS4DPad,
    DS4ControllerState,
    DS4Report
};

/// 虚拟游戏手柄客户端
/// 
/// 负责管理底层平台接口，创建和管理虚拟控制器
pub struct VGamepadClient {
    #[cfg(windows)]
    inner: windows::WindowsClient,
    
    #[cfg(target_os = "macos")]
    inner: macos::MacOSClient,
}

impl VGamepadClient {
    /// 创建新的虚拟游戏手柄客户端
    /// 
    /// # 错误
    /// 
    /// 如果无法初始化底层驱动程序（如ViGEm）则返回错误
    pub fn new() -> Result<Self> {
        log::info!("正在初始化虚拟游戏手柄客户端...");
        
        Ok(Self {
            #[cfg(windows)]
            inner: windows::WindowsClient::new()?,
            
            #[cfg(target_os = "macos")]
            inner: macos::MacOSClient::new()?,
        })
    }
    
    /// 创建新的DualShock4虚拟控制器
    /// 
    /// # 返回
    /// 
    /// 返回一个新的DualShock4控制器实例
    pub fn create_dualshock4(&self) -> Result<DualShock4Controller> {
        log::info!("正在创建DualShock4虚拟控制器...");
        DualShock4Controller::new(&self.inner)
    }
}

impl Default for VGamepadClient {
    fn default() -> Self {
        Self::new().expect("无法创建虚拟游戏手柄客户端")
    }
}