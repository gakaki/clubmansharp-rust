//! DualShock4控制器实现
//! 
//! 参考Python vgamepad库和ViGEmBus的DualShock4Controller实现

use crate::error::{Result, VGamepadError};

/// DualShock4按键位掩码 (参考ViGEmBus DS4_BUTTONS定义)
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DS4Button {
    /// 三角按键
    Triangle = 0x0080,
    /// 圆圈按键  
    Circle = 0x0020,
    /// X按键
    Cross = 0x0010,
    /// 方块按键
    Square = 0x0040,
    /// 左肩键L1
    L1 = 0x0001,
    /// 右肩键R1
    R1 = 0x0002,
    /// 左扳机键L2
    L2 = 0x0004,
    /// 右扳机键R2
    R2 = 0x0008,
    /// 分享按键
    Share = 0x1000,
    /// 选项按键
    Options = 0x2000,
    /// 左摇杆按下
    ThumbLeft = 0x0400,
    /// 右摇杆按下
    ThumbRight = 0x0800,
    /// PlayStation按键
    PlayStation = 0x0100,
    /// 触摸板按下
    TouchPad = 0x0200,
}

/// DualShock4方向键 (参考ViGEmBus DS4_DPAD定义)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DS4DPad {
    /// 中性位置
    None = 0x8,
    /// 向北 (上)
    North = 0x0,
    /// 向东北
    NorthEast = 0x1,
    /// 向东 (右)
    East = 0x2,
    /// 向东南
    SouthEast = 0x3,
    /// 向南 (下)
    South = 0x4,
    /// 向西南
    SouthWest = 0x5,
    /// 向西 (左)
    West = 0x6,
    /// 向西北
    NorthWest = 0x7,
}

/// DualShock4报告结构 (参考ViGEmBus DS4_REPORT定义)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DS4Report {
    /// 报告ID (固定为0x01)
    pub report_id: u8,
    /// 左摇杆X轴 (0-255, 128为中心)
    pub left_thumb_x: u8,
    /// 左摇杆Y轴 (0-255, 128为中心) 
    pub left_thumb_y: u8,
    /// 右摇杆X轴 (0-255, 128为中心)
    pub right_thumb_x: u8,
    /// 右摇杆Y轴 (0-255, 128为中心)
    pub right_thumb_y: u8,
    /// 按键状态位掩码
    pub buttons: u16,
    /// 方向键状态
    pub dpad: u8,
    /// 左扳机L2 (0-255)
    pub left_trigger: u8,
    /// 右扳机R2 (0-255)  
    pub right_trigger: u8,
    /// 时间戳
    pub timestamp: u16,
    /// 电池状态
    pub battery: u8,
    /// 陀螺仪X轴
    pub gyro_x: i16,
    /// 陀螺仪Y轴
    pub gyro_y: i16,
    /// 陀螺仪Z轴
    pub gyro_z: i16,
    /// 加速度计X轴
    pub accel_x: i16,
    /// 加速度计Y轴
    pub accel_y: i16,
    /// 加速度计Z轴
    pub accel_z: i16,
    /// 保留字段
    pub reserved: [u8; 5],
    /// 扩展数据
    pub extension: [u8; 12],
}

impl Default for DS4Report {
    fn default() -> Self {
        Self {
            report_id: 0x01,
            left_thumb_x: 128,
            left_thumb_y: 128, 
            right_thumb_x: 128,
            right_thumb_y: 128,
            buttons: 0,
            dpad: DS4DPad::None as u8,
            left_trigger: 0,
            right_trigger: 0,
            timestamp: 0,
            battery: 0,
            gyro_x: 0,
            gyro_y: 0,
            gyro_z: 0,
            accel_x: 0,
            accel_y: 0,
            accel_z: 0,
            reserved: [0; 5],
            extension: [0; 12],
        }
    }
}

/// DualShock4控制器状态 (参考vgamepad的API设计)
#[derive(Debug, Clone)]
pub struct DS4ControllerState {
    /// 内部报告结构
    pub(crate) report: DS4Report,
    /// LED颜色 (R, G, B)
    pub led_color: (u8, u8, u8),
    /// 左震动强度 (0-255)
    pub left_rumble: u8,
    /// 右震动强度 (0-255)
    pub right_rumble: u8,
}

impl Default for DS4ControllerState {
    fn default() -> Self {
        Self {
            report: DS4Report::default(),
            led_color: (0, 0, 255), // 默认蓝色LED
            left_rumble: 0,
            right_rumble: 0,
        }
    }
}

/// DualShock4虚拟控制器 (参考vgamepad的DualShock4Controller)
pub struct DualShock4Controller {
    /// 控制器当前状态
    state: DS4ControllerState,
    
    /// 平台特定的内部实现
    #[cfg(windows)]
    inner: crate::windows::WindowsDS4Controller,
    
    #[cfg(target_os = "macos")]
    inner: crate::macos::MacOSDS4Controller,
}

impl DualShock4Controller {
    /// 创建新的DualShock4控制器
    #[cfg(windows)]
    pub fn new(client: &crate::windows::WindowsClient) -> Result<Self> {
        let inner = crate::windows::WindowsDS4Controller::new(client)?;
        Ok(Self {
            state: DS4ControllerState::default(),
            inner,
        })
    }
    
    /// 创建新的DualShock4控制器  
    #[cfg(target_os = "macos")]
    pub fn new(client: &crate::macos::MacOSClient) -> Result<Self> {
        let inner = crate::macos::MacOSDS4Controller::new(client)?;
        Ok(Self {
            state: DS4ControllerState::default(),
            inner,
        })
    }
    
    /// 按下按键 (参考vgamepad的press_button)
    pub fn press_button(&mut self, button: DS4Button) -> Result<()> {
        log::debug!("按下按键: {:?}", button);
        self.state.report.buttons |= button as u16;
        self.update()
    }
    
    /// 释放按键 (参考vgamepad的release_button)
    pub fn release_button(&mut self, button: DS4Button) -> Result<()> {
        log::debug!("释放按键: {:?}", button);
        self.state.report.buttons &= !(button as u16);
        self.update()
    }
    
    /// 设置方向键 (参考vgamepad的directional_pad)
    pub fn set_dpad(&mut self, direction: DS4DPad) -> Result<()> {
        log::debug!("设置方向键: {:?}", direction);
        self.state.report.dpad = direction as u8;
        self.update()
    }
    
    /// 设置左摇杆 (参考vgamepad的left_joystick)
    /// x, y 范围: -1.0 到 1.0
    pub fn set_left_joystick(&mut self, x: f32, y: f32) -> Result<()> {
        if !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y) {
            return Err(VGamepadError::invalid_input(
                "left_joystick",
                "-1.0 到 1.0",
                format!("({}, {})", x, y)
            ));
        }
        
        log::debug!("设置左摇杆: x={}, y={}", x, y);
        self.state.report.left_thumb_x = ((x + 1.0) * 127.5) as u8;
        self.state.report.left_thumb_y = ((y + 1.0) * 127.5) as u8;
        self.update()
    }
    
    /// 设置右摇杆 (参考vgamepad的right_joystick)
    /// x, y 范围: -1.0 到 1.0
    pub fn set_right_joystick(&mut self, x: f32, y: f32) -> Result<()> {
        if !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y) {
            return Err(VGamepadError::invalid_input(
                "right_joystick",
                "-1.0 到 1.0",
                format!("({}, {})", x, y)
            ));
        }
        
        log::debug!("设置右摇杆: x={}, y={}", x, y);
        self.state.report.right_thumb_x = ((x + 1.0) * 127.5) as u8;
        self.state.report.right_thumb_y = ((y + 1.0) * 127.5) as u8;
        self.update()
    }
    
    /// 设置左扳机 (参考vgamepad的left_trigger)
    /// value 范围: 0.0 到 1.0
    pub fn set_left_trigger(&mut self, value: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&value) {
            return Err(VGamepadError::invalid_input(
                "left_trigger",
                "0.0 到 1.0",
                value.to_string()
            ));
        }
        
        log::debug!("设置左扳机: {}", value);
        self.state.report.left_trigger = (value * 255.0) as u8;
        self.update()
    }
    
    /// 设置右扳机 (参考vgamepad的right_trigger)
    /// value 范围: 0.0 到 1.0
    pub fn set_right_trigger(&mut self, value: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&value) {
            return Err(VGamepadError::invalid_input(
                "right_trigger",
                "0.0 到 1.0",
                value.to_string()
            ));
        }
        
        log::debug!("设置右扳机: {}", value);
        self.state.report.right_trigger = (value * 255.0) as u8;
        self.update()
    }
    
    /// 重置控制器到默认状态 (参考vgamepad的reset)
    pub fn reset(&mut self) -> Result<()> {
        log::info!("重置控制器状态");
        self.state = DS4ControllerState::default();
        self.update()
    }
    
    /// 获取当前状态
    pub fn get_state(&self) -> &DS4ControllerState {
        &self.state
    }
    
    /// 更新控制器状态到系统 (参考vgamepad的update)
    pub fn update(&mut self) -> Result<()> {
        self.inner.update(&self.state)
    }
}