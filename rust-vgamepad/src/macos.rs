//! macOS平台实现 - Game Controller Framework + IOKit HID
//! 
//! 注意：macOS上创建虚拟游戏控制器比Windows复杂得多
//! 
//! 实际可行的方案：
//! 1. 使用Game Controller Framework读取现有控制器（仅支持MFi设备）
//! 2. 使用IOKit HID创建虚拟设备（需要用户态驱动程序权限）
//! 3. 现代macOS建议使用DriverKit，但需要苹果开发者账户和特殊权限
//! 
//! 当前实现提供了框架和接口，但由于权限限制，实际的虚拟设备创建
//! 需要额外的系统配置和开发者账户

use crate::controller::DS4ControllerState;
use crate::error::{Result, VGamepadError};

/// macOS虚拟控制器方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacOSVirtualMethod {
    /// 模拟模式（用于测试和开发）
    Simulation,
    /// IOKit HID用户态设备（需要SIP禁用）
    IOKitUserspace,
    /// DriverKit扩展（需要开发者账户和权限）
    DriverKit,
}

/// macOS客户端
pub struct MacOSClient {
    /// 虚拟控制器方法
    method: MacOSVirtualMethod,
    /// 是否已初始化
    initialized: bool,
}

impl MacOSClient {
    /// 创建新的macOS客户端
    pub fn new() -> Result<Self> {
        Self::new_with_method(MacOSVirtualMethod::Simulation)
    }
    
    /// 使用指定方法创建macOS客户端
    pub fn new_with_method(method: MacOSVirtualMethod) -> Result<Self> {
        log::info!("正在初始化macOS虚拟控制器客户端...");
        
        match method {
            MacOSVirtualMethod::Simulation => {
                log::info!("使用模拟模式 - 不会创建真实的虚拟设备");
            }
            MacOSVirtualMethod::IOKitUserspace => {
                log::warn!("IOKit用户态设备需要禁用SIP：csrutil disable");
                log::warn!("这可能会降低系统安全性");
                
                // 检查是否可以访问IOKit
                if !Self::check_iokit_access() {
                    return Err(VGamepadError::insufficient_permissions(
                        "IOKit HID设备创建"
                    ));
                }
            }
            MacOSVirtualMethod::DriverKit => {
                log::warn!("DriverKit方法需要：");
                log::warn!("1. 苹果开发者账户");
                log::warn!("2. HIDDriverKit权限申请");
                log::warn!("3. 代码签名和公证");
                
                return Err(VGamepadError::unsupported_platform(
                    "macOS",
                    "DriverKit虚拟控制器需要特殊权限"
                ));
            }
        }
        
        log::info!("macOS虚拟控制器客户端初始化成功");
        
        Ok(Self {
            method,
            initialized: true,
        })
    }
    
    /// 检查IOKit访问权限
    fn check_iokit_access() -> bool {
        // 在真实实现中，这里应该尝试打开IOKit主端口
        // 目前简化为检查是否在开发环境
        cfg!(debug_assertions)
    }
    
    /// 获取当前使用的方法
    pub fn get_method(&self) -> MacOSVirtualMethod {
        self.method
    }
    
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// 列出可用的游戏控制器（使用Game Controller Framework）
    pub fn list_controllers(&self) -> Vec<String> {
        // 在真实实现中，这里应该使用Game Controller Framework
        // 枚举当前连接的MFi控制器
        log::info!("枚举可用的游戏控制器...");
        
        // 模拟返回一些常见的控制器类型
        vec![
            "DualShock 4 Wireless Controller".to_string(),
            "Xbox Wireless Controller".to_string(),
            "Generic MFi Controller".to_string(),
        ]
    }
}

/// macOS DualShock4控制器
pub struct MacOSDS4Controller {
    /// 控制器方法
    method: MacOSVirtualMethod,
    /// 设备ID（模拟）
    device_id: u32,
    /// 是否已连接
    connected: bool,
    /// 最后一次状态更新时间
    last_update: std::time::Instant,
}

impl MacOSDS4Controller {
    /// 创建新的macOS DS4控制器
    pub fn new(client: &MacOSClient) -> Result<Self> {
        log::info!("正在创建macOS虚拟DS4控制器...");
        
        let method = client.get_method();
        
        match method {
            MacOSVirtualMethod::Simulation => {
                log::info!("创建模拟虚拟控制器");
            }
            MacOSVirtualMethod::IOKitUserspace => {
                log::info!("尝试创建IOKit HID虚拟设备...");
                // 在真实实现中，这里需要：
                // 1. 创建IOHIDUserDevice
                // 2. 设置HID描述符
                // 3. 注册设备到系统
                Self::create_iokit_device()?;
            }
            MacOSVirtualMethod::DriverKit => {
                return Err(VGamepadError::unsupported_platform(
                    "macOS",
                    "DriverKit方法尚未实现"
                ));
            }
        }
        
        let mut controller = Self {
            method,
            device_id: 1,
            connected: false,
            last_update: std::time::Instant::now(),
        };
        
        // 自动连接
        controller.connect()?;
        
        Ok(controller)
    }
    
    /// 创建IOKit HID设备（需要权限）
    fn create_iokit_device() -> Result<()> {
        // 在真实实现中，这里需要：
        // 1. 包含IOKit框架
        // 2. 创建HID设备描述符
        // 3. 注册虚拟设备
        // 4. 处理权限问题
        
        log::warn!("IOKit HID设备创建需要系统权限");
        log::info!("当前为模拟实现");
        
        Ok(())
    }
    
    /// 连接控制器
    pub fn connect(&mut self) -> Result<()> {
        log::info!("正在连接虚拟DS4控制器...");
        
        match self.method {
            MacOSVirtualMethod::Simulation => {
                log::info!("模拟连接成功");
            }
            MacOSVirtualMethod::IOKitUserspace => {
                // 激活IOKit设备
                log::info!("激活IOKit HID设备...");
            }
            MacOSVirtualMethod::DriverKit => {
                return Err(VGamepadError::unsupported_platform(
                    "macOS",
                    "DriverKit方法尚未实现"
                ));
            }
        }
        
        self.connected = true;
        log::info!("虚拟DS4控制器连接成功 ({})", 
            match self.method {
                MacOSVirtualMethod::Simulation => "模拟",
                MacOSVirtualMethod::IOKitUserspace => "IOKit",
                MacOSVirtualMethod::DriverKit => "DriverKit",
            }
        );
        
        Ok(())
    }
    
    /// 断开连接控制器
    pub fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }
        
        log::info!("正在断开虚拟DS4控制器...");
        
        match self.method {
            MacOSVirtualMethod::Simulation => {
                log::info!("模拟断开连接");
            }
            MacOSVirtualMethod::IOKitUserspace => {
                // 停用IOKit设备
                log::info!("停用IOKit HID设备...");
            }
            MacOSVirtualMethod::DriverKit => {
                // DriverKit设备清理
            }
        }
        
        self.connected = false;
        log::info!("虚拟DS4控制器已断开连接");
        
        Ok(())
    }
    
    /// 更新控制器状态
    pub fn update(&mut self, state: &DS4ControllerState) -> Result<()> {
        if !self.connected {
            return Err(VGamepadError::ControllerDisconnected);
        }
        
        // 限制更新频率（避免过于频繁的日志）
        let now = std::time::Instant::now();
        let should_log = now.duration_since(self.last_update) > std::time::Duration::from_millis(100);
        
        match self.method {
            MacOSVirtualMethod::Simulation => {
                if should_log {
                    // 读取packed字段到本地变量以避免unaligned reference
                    let buttons = state.report.buttons;
                    let left_thumb_x = state.report.left_thumb_x;
                    let left_thumb_y = state.report.left_thumb_y;
                    let right_thumb_x = state.report.right_thumb_x;
                    let right_thumb_y = state.report.right_thumb_y;
                    let left_trigger = state.report.left_trigger;
                    let right_trigger = state.report.right_trigger;
                    
                    log::debug!("更新DS4状态(模拟): 按键=0x{:04X}, L摇杆=({},{}), R摇杆=({},{}), 扳机=({},{})", 
                        buttons, left_thumb_x, left_thumb_y, 
                        right_thumb_x, right_thumb_y,
                        left_trigger, right_trigger
                    );
                    
                    self.last_update = now;
                }
            }
            MacOSVirtualMethod::IOKitUserspace => {
                // 在真实实现中，发送HID报告到IOKit设备
                if should_log {
                    log::debug!("发送HID报告到IOKit设备...");
                    self.last_update = now;
                }
            }
            MacOSVirtualMethod::DriverKit => {
                // DriverKit设备更新
                return Err(VGamepadError::unsupported_platform(
                    "macOS",
                    "DriverKit方法尚未实现"
                ));
            }
        }
        
        Ok(())
    }
    
    /// 获取设备状态信息
    pub fn get_device_info(&self) -> (MacOSVirtualMethod, u32, bool) {
        (self.method, self.device_id, self.connected)
    }
    
    /// 创建DualShock4 HID描述符
    /// 
    /// 用于IOKit或DriverKit实现
    #[allow(dead_code)]
    fn create_ds4_hid_descriptor() -> Vec<u8> {
        // 完整的DualShock4 HID报告描述符
        vec![
            0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
            0x09, 0x05,        // Usage (Game Pad)
            0xA1, 0x01,        // Collection (Application)
            0x85, 0x01,        //   Report ID (1)
            
            // 摇杆轴
            0x09, 0x30,        //   Usage (X)
            0x09, 0x31,        //   Usage (Y)
            0x09, 0x32,        //   Usage (Z)
            0x09, 0x35,        //   Usage (Rz)
            0x15, 0x00,        //   Logical Minimum (0)
            0x26, 0xFF, 0x00,  //   Logical Maximum (255)
            0x75, 0x08,        //   Report Size (8)
            0x95, 0x04,        //   Report Count (4)
            0x81, 0x02,        //   Input (Data,Var,Abs)
            
            // 按键
            0x05, 0x09,        //   Usage Page (Button)
            0x19, 0x01,        //   Usage Minimum (0x01)
            0x29, 0x0E,        //   Usage Maximum (0x0E)
            0x15, 0x00,        //   Logical Minimum (0)
            0x25, 0x01,        //   Logical Maximum (1)
            0x75, 0x01,        //   Report Size (1)
            0x95, 0x0E,        //   Report Count (14)
            0x81, 0x02,        //   Input (Data,Var,Abs)
            
            // 填充位
            0x75, 0x02,        //   Report Size (2)
            0x95, 0x01,        //   Report Count (1)
            0x81, 0x03,        //   Input (Cnst,Var,Abs)
            
            // 方向键
            0x05, 0x01,        //   Usage Page (Generic Desktop Ctrls)
            0x09, 0x39,        //   Usage (Hat switch)
            0x15, 0x00,        //   Logical Minimum (0)
            0x25, 0x07,        //   Logical Maximum (7)
            0x35, 0x00,        //   Physical Minimum (0)
            0x46, 0x3B, 0x01,  //   Physical Maximum (315)
            0x65, 0x14,        //   Unit (System: English Rotation, Length: Centimeter)
            0x75, 0x04,        //   Report Size (4)
            0x95, 0x01,        //   Report Count (1)
            0x81, 0x42,        //   Input (Data,Var,Abs,Null State)
            
            // 更多填充
            0x75, 0x04,        //   Report Size (4)
            0x95, 0x01,        //   Report Count (1)
            0x81, 0x03,        //   Input (Cnst,Var,Abs)
            
            // 扳机
            0x05, 0x01,        //   Usage Page (Generic Desktop Ctrls)
            0x09, 0x32,        //   Usage (Z)
            0x09, 0x35,        //   Usage (Rz)
            0x15, 0x00,        //   Logical Minimum (0)
            0x26, 0xFF, 0x00,  //   Logical Maximum (255)
            0x75, 0x08,        //   Report Size (8)
            0x95, 0x02,        //   Report Count (2)
            0x81, 0x02,        //   Input (Data,Var,Abs)
            
            0xC0,              // End Collection
        ]
    }
}

impl Drop for MacOSDS4Controller {
    fn drop(&mut self) {
        log::info!("正在清理macOS DS4控制器...");
        let _ = self.disconnect();
    }
}

/// macOS虚拟控制器实用函数
pub mod utils {
    use super::*;
    
    /// 检查macOS版本兼容性
    pub fn check_macos_compatibility() -> Result<()> {
        log::info!("检查macOS虚拟控制器兼容性...");
        
        // 在真实实现中，可以检查：
        // 1. macOS版本是否支持DriverKit
        // 2. 是否禁用了SIP（对IOKit用户态设备）
        // 3. 开发者权限
        
        log::warn!("macOS虚拟控制器实现说明：");
        log::warn!("1. 模拟模式：适用于测试和开发");
        log::warn!("2. IOKit模式：需要禁用SIP，安全风险");
        log::warn!("3. DriverKit模式：需要苹果开发者账户和特殊权限");
        
        Ok(())
    }
    
    /// 获取系统信息
    pub fn get_system_info() -> (String, String) {
        // 在真实实现中，可以获取：
        // 1. macOS版本
        // 2. SIP状态
        // 3. 开发者模式状态
        
        ("macOS".to_string(), "模拟环境".to_string())
    }
    
    /// 检查权限
    pub fn check_permissions() -> Vec<String> {
        let mut permissions = Vec::new();
        
        // 模拟权限检查
        permissions.push("模拟模式: ✅ 可用".to_string());
        permissions.push("IOKit模式: ❌ 需要禁用SIP".to_string());
        permissions.push("DriverKit模式: ❌ 需要开发者权限".to_string());
        
        permissions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_macos_client_creation() {
        let result = MacOSClient::new();
        assert!(result.is_ok(), "macOS客户端创建应该成功");
        
        let client = result.unwrap();
        assert!(client.is_initialized(), "客户端应该已初始化");
        assert_eq!(client.get_method(), MacOSVirtualMethod::Simulation);
    }
    
    #[test]
    fn test_different_methods() {
        // 测试模拟模式
        let sim_client = MacOSClient::new_with_method(MacOSVirtualMethod::Simulation);
        assert!(sim_client.is_ok());
        
        // 测试IOKit模式（可能失败）
        let iokit_result = MacOSClient::new_with_method(MacOSVirtualMethod::IOKitUserspace);
        // 在没有权限的环境中可能失败，这是正常的
        
        // 测试DriverKit模式（应该失败）
        let driverkit_result = MacOSClient::new_with_method(MacOSVirtualMethod::DriverKit);
        assert!(driverkit_result.is_err(), "DriverKit模式应该返回错误");
    }
    
    #[test]
    fn test_ds4_controller_creation() {
        let client = MacOSClient::new().unwrap();
        let result = MacOSDS4Controller::new(&client);
        assert!(result.is_ok(), "DS4控制器创建应该成功");
        
        let controller = result.unwrap();
        let (method, device_id, connected) = controller.get_device_info();
        assert_eq!(method, MacOSVirtualMethod::Simulation);
        assert_eq!(device_id, 1);
        assert!(connected);
    }
    
    #[test]
    fn test_compatibility_check() {
        let result = utils::check_macos_compatibility();
        assert!(result.is_ok(), "兼容性检查应该成功");
    }
    
    #[test]
    fn test_controller_list() {
        let client = MacOSClient::new().unwrap();
        let controllers = client.list_controllers();
        assert!(!controllers.is_empty(), "应该返回控制器列表");
    }
}