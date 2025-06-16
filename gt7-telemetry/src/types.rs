//! GT7遥测数据类型定义
//! 
//! 参考gt7telemetry Python库的数据结构

use serde::{Deserialize, Serialize};

/// 3D向量
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

/// 车辆位置和方向
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// 世界坐标位置
    pub world: Vector3,
    /// 速度向量 (m/s)
    pub velocity: Vector3,
    /// 角速度 (rad/s)
    pub angular_velocity: Vector3,
    /// 朝向角度 (弧度)
    pub rotation: Vector3,
}

/// 车辆轮胎信息
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TireInfo {
    /// 前左轮
    pub front_left: TireData,
    /// 前右轮
    pub front_right: TireData,
    /// 后左轮
    pub rear_left: TireData,
    /// 后右轮
    pub rear_right: TireData,
}

/// 单个轮胎数据
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TireData {
    /// 轮胎温度 (摄氏度)
    pub temperature: f32,
    /// 轮胎磨损 (0.0-1.0)
    pub wear: f32,
    /// 悬挂行程 (米)
    pub suspension_travel: f32,
    /// 轮速 (rad/s)
    pub wheel_speed: f32,
    /// 轮胎半径 (米)
    pub radius: f32,
}

/// 车辆发动机信息
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EngineInfo {
    /// 发动机转速 (RPM)
    pub rpm: f32,
    /// 最大转速 (RPM)
    pub max_rpm: f32,
    /// 油门开度 (0.0-1.0)
    pub throttle: f32,
    /// 刹车力度 (0.0-1.0)
    pub brake: f32,
    /// 离合器状态 (0.0-1.0)
    pub clutch: f32,
    /// 当前档位 (0=倒档, 1-8=前进档)
    pub gear: i8,
    /// 建议档位
    pub suggested_gear: i8,
    /// 燃油剩余 (升)
    pub fuel_remaining: f32,
    /// 燃油消耗率 (升/圈)
    pub fuel_consumption: f32,
    /// 燃油箱容量 (升)
    pub fuel_capacity: f32,
    /// 当前燃油等级 (0.0-1.0)
    pub fuel_level: f32,
}

/// 比赛信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceInfo {
    /// 当前圈数
    pub current_lap: u16,
    /// 总圈数
    pub total_laps: u16,
    /// 当前位置
    pub position: u8,
    /// 总参赛者数
    pub total_participants: u8,
    /// 最快圈速 (毫秒)
    pub best_lap_time: Option<u32>,
    /// 上一圈时间 (毫秒)
    pub last_lap_time: Option<u32>,
    /// 当前圈时间 (毫秒)
    pub current_lap_time: u32,
    /// 赛道完成进度 (0.0-1.0)
    pub track_progress: f32,
}

/// 游戏状态枚举 (参考gt7telemetry)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStateType {
    /// 菜单中
    InMenu = 0,
    /// 比赛中
    InRace = 1,
    /// 暂停
    Paused = 2,
    /// 重播
    Replay = 3,
    /// 车库
    Garage = 4,
    /// 加载中
    Loading = 5,
    /// 未知状态
    Unknown = 255,
}

impl From<u8> for GameStateType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::InMenu,
            1 => Self::InRace,
            2 => Self::Paused,
            3 => Self::Replay,
            4 => Self::Garage,
            5 => Self::Loading,
            _ => Self::Unknown,
        }
    }
}

/// 赛道信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackData {
    /// 赛道ID
    pub track_id: u32,
    /// 赛道名称
    pub track_name: String,
    /// 赛道长度 (米)
    pub track_length: f32,
    /// 海拔高度 (米)
    pub altitude: f32,
    /// 天气状况
    pub weather: WeatherCondition,
    /// 路面温度 (摄氏度)
    pub road_temperature: f32,
    /// 环境温度 (摄氏度)
    pub air_temperature: f32,
}

/// 天气状况
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    /// 晴朗
    Clear = 0,
    /// 多云
    Cloudy = 1,
    /// 小雨
    LightRain = 2,
    /// 大雨
    HeavyRain = 3,
    /// 雾
    Fog = 4,
    /// 雪
    Snow = 5,
    /// 未知
    Unknown = 255,
}

impl From<u8> for WeatherCondition {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Clear,
            1 => Self::Cloudy,
            2 => Self::LightRain,
            3 => Self::HeavyRain,
            4 => Self::Fog,
            5 => Self::Snow,
            _ => Self::Unknown,
        }
    }
}

/// 车辆配置信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarConfiguration {
    /// 车辆ID
    pub car_id: u32,
    /// 车辆名称
    pub car_name: String,
    /// 车辆类别
    pub car_category: String,
    /// 车重 (千克)
    pub weight: f32,
    /// 功率 (马力)
    pub power: f32,
    /// 扭矩 (牛·米)
    pub torque: f32,
    /// 驱动方式 (FF/FR/MR/RR/4WD)
    pub drivetrain: String,
    /// 轮胎类型
    pub tire_type: String,
}

/// 遥测客户端配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// PS4/PS5 IP地址
    pub console_ip: String,
    /// 遥测端口
    pub port: u16,
    /// 连接超时时间 (秒)
    pub timeout: u64,
    /// 心跳间隔 (毫秒)
    pub heartbeat_interval: u64,
    /// 是否启用数据记录
    pub enable_logging: bool,
    /// 日志文件路径
    pub log_file_path: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            console_ip: "192.168.1.30".to_string(),
            port: crate::GT7_TELEMETRY_PORT,
            timeout: 5,
            heartbeat_interval: 100,
            enable_logging: false,
            log_file_path: None,
        }
    }
}