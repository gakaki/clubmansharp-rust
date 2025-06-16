//! GT7遥测数据包解析
//! 
//! 参考gt7telemetry Python库实现二进制数据包解析

use crate::error::{Result, GT7Error};
use crate::types::*;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use std::time::Duration;

/// GT7遥测数据包版本
pub const GT7_PACKET_VERSION: u16 = 1;

/// GT7遥测数据包 (参考gt7telemetry的数据结构)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GT7TelemetryPacket {
    /// 数据包版本
    pub version: u16,
    /// 游戏状态
    pub game_state: GameState,
    /// 车辆信息
    pub car_info: CarInfo,
    /// 赛道信息
    pub track_info: TrackInfo,
    /// 数据包时间戳
    pub timestamp: u64,
    /// 数据包计数器
    pub packet_id: u32,
}

/// 游戏状态信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    /// 游戏状态类型
    pub state_type: GameStateType,
    /// 比赛信息
    pub race_info: Option<RaceInfo>,
    /// 是否在暂停状态
    pub is_paused: bool,
    /// 是否在重播状态
    pub is_replay: bool,
    /// 菜单状态ID
    pub menu_id: u32,
}

/// 车辆信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarInfo {
    /// 车辆位置和速度
    pub position: Position,
    /// 轮胎信息
    pub tires: TireInfo,
    /// 发动机信息
    pub engine: EngineInfo,
    /// 车辆配置
    pub configuration: Option<CarConfiguration>,
}

/// 赛道信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackInfo {
    /// 赛道数据
    pub track_data: TrackData,
    /// 当前赛段信息
    pub current_sector: u8,
    /// 赛道湿度 (0.0-1.0)
    pub track_wetness: f32,
}

impl GT7TelemetryPacket {
    /// 从原始字节数据解析GT7遥测数据包
    /// 
    /// # 参数
    /// 
    /// * `data` - 原始字节数据 (应为296字节)
    /// 
    /// # 返回
    /// 
    /// 解析后的遥测数据包
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() != crate::GT7_PACKET_SIZE {
            return Err(GT7Error::incomplete_data(crate::GT7_PACKET_SIZE, data.len()));
        }

        let mut cursor = Cursor::new(data);
        
        // 解析数据包头部
        let magic = cursor.read_u32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("魔术字节", 0, 4))?;
        
        // 验证魔术字节 (GT7特有的标识)
        if magic != 0x47375053 { // "GT7S" in little endian
            return Err(GT7Error::invalid_packet_format("magic"));
        }

        let version = cursor.read_u16::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("版本", 4, 2))?;
        
        if version != GT7_PACKET_VERSION {
            return Err(GT7Error::packet_version_mismatch(GT7_PACKET_VERSION, version));
        }

        let packet_id = cursor.read_u32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("数据包ID", 6, 4))?;

        // 解析游戏状态 (偏移: 10)
        let game_state = Self::parse_game_state(&mut cursor)?;

        // 解析车辆信息 (偏移: 50) 
        cursor.set_position(50);
        let car_info = Self::parse_car_info(&mut cursor)?;

        // 解析赛道信息 (偏移: 200)
        cursor.set_position(200);
        let track_info = Self::parse_track_info(&mut cursor)?;

        // 解析时间戳 (偏移: 280)
        cursor.set_position(280);
        let timestamp = cursor.read_u64::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("时间戳", 280, 8))?;

        Ok(Self {
            version,
            game_state,
            car_info,
            track_info,
            timestamp,
            packet_id,
        })
    }

    /// 解析游戏状态信息
    fn parse_game_state(cursor: &mut Cursor<&[u8]>) -> Result<GameState> {
        let state_type_raw = cursor.read_u8()
            .map_err(|_| GT7Error::packet_parse_error("游戏状态类型", cursor.position() as usize - 1, 1))?;
        let state_type = GameStateType::from(state_type_raw);

        let is_paused = cursor.read_u8()
            .map_err(|_| GT7Error::packet_parse_error("暂停状态", cursor.position() as usize - 1, 1))? != 0;

        let is_replay = cursor.read_u8()
            .map_err(|_| GT7Error::packet_parse_error("重播状态", cursor.position() as usize - 1, 1))? != 0;

        let menu_id = cursor.read_u32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("菜单ID", cursor.position() as usize - 4, 4))?;

        // 解析比赛信息 (仅在比赛状态下)
        let race_info = if state_type == GameStateType::InRace {
            Some(Self::parse_race_info(cursor)?)
        } else {
            None
        };

        Ok(GameState {
            state_type,
            race_info,
            is_paused,
            is_replay,
            menu_id,
        })
    }

    /// 解析比赛信息
    fn parse_race_info(cursor: &mut Cursor<&[u8]>) -> Result<RaceInfo> {
        let current_lap = cursor.read_u16::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("当前圈数", cursor.position() as usize - 2, 2))?;

        let total_laps = cursor.read_u16::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("总圈数", cursor.position() as usize - 2, 2))?;

        let position = cursor.read_u8()
            .map_err(|_| GT7Error::packet_parse_error("当前位置", cursor.position() as usize - 1, 1))?;

        let total_participants = cursor.read_u8()
            .map_err(|_| GT7Error::packet_parse_error("总参赛者", cursor.position() as usize - 1, 1))?;

        let best_lap_time_raw = cursor.read_u32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("最快圈速", cursor.position() as usize - 4, 4))?;
        let best_lap_time = if best_lap_time_raw > 0 { Some(best_lap_time_raw) } else { None };

        let last_lap_time_raw = cursor.read_u32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("上圈时间", cursor.position() as usize - 4, 4))?;
        let last_lap_time = if last_lap_time_raw > 0 { Some(last_lap_time_raw) } else { None };

        let current_lap_time = cursor.read_u32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("当前圈时间", cursor.position() as usize - 4, 4))?;

        let track_progress = cursor.read_f32::<LittleEndian>()
            .map_err(|_| GT7Error::packet_parse_error("赛道进度", cursor.position() as usize - 4, 4))?;

        Ok(RaceInfo {
            current_lap,
            total_laps,
            position,
            total_participants,
            best_lap_time,
            last_lap_time,
            current_lap_time,
            track_progress,
        })
    }

    /// 解析车辆信息
    fn parse_car_info(cursor: &mut Cursor<&[u8]>) -> Result<CarInfo> {
        // 解析位置信息
        let world_x = cursor.read_f32::<LittleEndian>()?;
        let world_y = cursor.read_f32::<LittleEndian>()?;
        let world_z = cursor.read_f32::<LittleEndian>()?;

        let vel_x = cursor.read_f32::<LittleEndian>()?;
        let vel_y = cursor.read_f32::<LittleEndian>()?;
        let vel_z = cursor.read_f32::<LittleEndian>()?;

        let rot_x = cursor.read_f32::<LittleEndian>()?;
        let rot_y = cursor.read_f32::<LittleEndian>()?;
        let rot_z = cursor.read_f32::<LittleEndian>()?;

        let ang_vel_x = cursor.read_f32::<LittleEndian>()?;
        let ang_vel_y = cursor.read_f32::<LittleEndian>()?;
        let ang_vel_z = cursor.read_f32::<LittleEndian>()?;

        let position = Position {
            world: Vector3::new(world_x, world_y, world_z),
            velocity: Vector3::new(vel_x, vel_y, vel_z),
            angular_velocity: Vector3::new(ang_vel_x, ang_vel_y, ang_vel_z),
            rotation: Vector3::new(rot_x, rot_y, rot_z),
        };

        // 解析轮胎信息
        let tires = TireInfo {
            front_left: TireData {
                temperature: cursor.read_f32::<LittleEndian>()?,
                wear: cursor.read_f32::<LittleEndian>()?,
                suspension_travel: cursor.read_f32::<LittleEndian>()?,
                wheel_speed: cursor.read_f32::<LittleEndian>()?,
                radius: cursor.read_f32::<LittleEndian>()?,
            },
            front_right: TireData {
                temperature: cursor.read_f32::<LittleEndian>()?,
                wear: cursor.read_f32::<LittleEndian>()?,
                suspension_travel: cursor.read_f32::<LittleEndian>()?,
                wheel_speed: cursor.read_f32::<LittleEndian>()?,
                radius: cursor.read_f32::<LittleEndian>()?,
            },
            rear_left: TireData {
                temperature: cursor.read_f32::<LittleEndian>()?,
                wear: cursor.read_f32::<LittleEndian>()?,
                suspension_travel: cursor.read_f32::<LittleEndian>()?,
                wheel_speed: cursor.read_f32::<LittleEndian>()?,
                radius: cursor.read_f32::<LittleEndian>()?,
            },
            rear_right: TireData {
                temperature: cursor.read_f32::<LittleEndian>()?,
                wear: cursor.read_f32::<LittleEndian>()?,
                suspension_travel: cursor.read_f32::<LittleEndian>()?,
                wheel_speed: cursor.read_f32::<LittleEndian>()?,
                radius: cursor.read_f32::<LittleEndian>()?,
            },
        };

        // 解析发动机信息
        let fuel_remaining = cursor.read_f32::<LittleEndian>()?;
        let fuel_capacity = cursor.read_f32::<LittleEndian>()?;
        let fuel_level = if fuel_capacity > 0.0 { fuel_remaining / fuel_capacity } else { 0.0 };
        
        let engine = EngineInfo {
            rpm: cursor.read_f32::<LittleEndian>()?,
            max_rpm: cursor.read_f32::<LittleEndian>()?,
            throttle: cursor.read_f32::<LittleEndian>()?,
            brake: cursor.read_f32::<LittleEndian>()?,
            clutch: cursor.read_f32::<LittleEndian>()?,
            gear: cursor.read_i8()?,
            suggested_gear: cursor.read_i8()?,
            fuel_remaining,
            fuel_consumption: cursor.read_f32::<LittleEndian>()?,
            fuel_capacity,
            fuel_level,
        };

        Ok(CarInfo {
            position,
            tires,
            engine,
            configuration: None, // 车辆配置在别的地方解析
        })
    }

    /// 解析赛道信息
    fn parse_track_info(cursor: &mut Cursor<&[u8]>) -> Result<TrackInfo> {
        let track_id = cursor.read_u32::<LittleEndian>()?;
        let track_length = cursor.read_f32::<LittleEndian>()?;
        let altitude = cursor.read_f32::<LittleEndian>()?;
        
        let weather_raw = cursor.read_u8()?;
        let weather = WeatherCondition::from(weather_raw);
        
        let road_temperature = cursor.read_f32::<LittleEndian>()?;
        let air_temperature = cursor.read_f32::<LittleEndian>()?;
        let current_sector = cursor.read_u8()?;
        let track_wetness = cursor.read_f32::<LittleEndian>()?;

        // 读取赛道名称 (假设为32字节的UTF-8字符串)
        let mut track_name_bytes = [0u8; 32];
        cursor.read_exact(&mut track_name_bytes)?;
        let track_name = String::from_utf8_lossy(&track_name_bytes)
            .trim_end_matches('\0')
            .to_string();

        let track_data = TrackData {
            track_id,
            track_name,
            track_length,
            altitude,
            weather,
            road_temperature,
            air_temperature,
        };

        Ok(TrackInfo {
            track_data,
            current_sector,
            track_wetness,
        })
    }

    /// 验证数据包完整性
    pub fn validate(&self) -> Result<()> {
        // 验证版本
        if self.version != GT7_PACKET_VERSION {
            return Err(GT7Error::packet_version_mismatch(GT7_PACKET_VERSION, self.version));
        }

        // 验证基本数据范围
        if self.car_info.engine.throttle < 0.0 || self.car_info.engine.throttle > 1.0 {
            return Err(GT7Error::invalid_packet_format("油门值超出范围"));
        }

        if self.car_info.engine.brake < 0.0 || self.car_info.engine.brake > 1.0 {
            return Err(GT7Error::invalid_packet_format("刹车值超出范围"));
        }

        if self.track_info.track_wetness < 0.0 || self.track_info.track_wetness > 1.0 {
            return Err(GT7Error::invalid_packet_format("赛道湿度超出范围"));
        }

        Ok(())
    }

    /// 计算车辆当前速度 (km/h)
    pub fn get_speed_kmh(&self) -> f32 {
        self.car_info.position.velocity.magnitude() * 3.6
    }

    /// 检查是否在比赛中
    pub fn is_in_race(&self) -> bool {
        self.game_state.state_type == GameStateType::InRace
    }

    /// 检查是否在菜单中
    pub fn is_in_menu(&self) -> bool {
        self.game_state.state_type == GameStateType::InMenu
    }

    /// 获取当前档位显示字符串
    pub fn get_gear_display(&self) -> String {
        match self.car_info.engine.gear {
            0 => "R".to_string(),
            g if g > 0 => g.to_string(),
            _ => "N".to_string(),
        }
    }
    
    /// 获取最快圈速时间
    pub fn get_best_lap_time(&self) -> Option<Duration> {
        self.game_state.race_info.as_ref()
            .and_then(|r| r.best_lap_time)
            .map(|ms| Duration::from_millis(ms as u64))
    }
}