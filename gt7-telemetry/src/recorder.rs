//! 赛道数据记录和管理
//!
//! 提供用于记录、保存和加载赛道数据的功能

use crate::packet::GT7TelemetryPacket;
use crate::types::{Position, TrackData};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tokio::sync::broadcast;

/// 记录的赛道数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordedTrack {
    /// 赛道元数据
    pub track_data: Option<TrackData>,
    /// 记录的赛车位置点 (驾驶路线)
    pub racing_line: Vec<Position>,
    /// 左边界线
    pub left_border: Vec<Position>,
    /// 右边界线
    pub right_border: Vec<Position>,
}

impl RecordedTrack {
    /// 从JSON文件加载赛道数据
    pub fn from_json_file(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&json_data).map_err(|e| e.to_string())
    }

    /// 将赛道数据保存到JSON文件
    pub fn save_to_json_file(&self, path: &Path) -> Result<(), String> {
        let json_data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(json_data.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 录制模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingMode {
    /// 驾驶路线
    RacingLine,
    /// 左边界
    LeftBorder,
    /// 右边界
    RightBorder,
}

/// 赛道记录器
pub struct TrackRecorder {
    is_recording: bool,
    recording_mode: RecordingMode,
    recorded_track: RecordedTrack,
    packet_receiver: broadcast::Receiver<(String, GT7TelemetryPacket)>,
}

impl TrackRecorder {
    /// 创建新的赛道记录器
    pub fn new(packet_receiver: broadcast::Receiver<(String, GT7TelemetryPacket)>) -> Self {
        Self {
            is_recording: false,
            recording_mode: RecordingMode::RacingLine,
            recorded_track: RecordedTrack::default(),
            packet_receiver,
        }
    }

    /// 开始记录
    pub fn start_recording(&mut self) {
        if self.is_recording {
            log::warn!("录制已经开始");
            return;
        }
        self.is_recording = true;
        self.recorded_track = RecordedTrack::default(); // 清除旧数据
        log::info!("开始录制赛道数据...");
    }

    /// 停止记录
    pub fn stop_recording(&mut self) {
        if !self.is_recording {
            log::warn!("录制尚未开始");
            return;
        }
        self.is_recording = false;
        log::info!("停止录制赛道数据。");
    }

    /// 设置录制模式
    pub fn set_recording_mode(&mut self, mode: RecordingMode) {
        self.recording_mode = mode;
        log::info!("设置录制模式为: {:?}", mode);
    }

    /// 更新记录器状态，处理新的遥测数据包
    pub async fn update(&mut self) {
        if !self.is_recording {
            return;
        }

        match self.packet_receiver.recv().await {
            Ok((_, packet)) => {
                if packet.is_in_race() {
                    let position = packet.car_info.position;

                    if self.recorded_track.track_data.is_none() {
                        self.recorded_track.track_data = Some(packet.track_info.track_data.clone());
                        log::info!("检测到赛道: {}", packet.track_info.track_data.track_name);
                    }

                    match self.recording_mode {
                        RecordingMode::RacingLine => self.recorded_track.racing_line.push(position),
                        RecordingMode::LeftBorder => self.recorded_track.left_border.push(position),
                        RecordingMode::RightBorder => self.recorded_track.right_border.push(position),
                    }
                }
            }
            Err(e) => {
                log::error!("接收遥测数据包失败: {}", e);
            }
        }
    }

    /// 获取记录的赛道数据
    pub fn get_recorded_track(&self) -> &RecordedTrack {
        &self.recorded_track
    }

    /// 检查是否正在录制
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::TrackInfo;
    use crate::types::{EngineInfo, GameStateType, TireData, TireInfo, Vector3, WeatherCondition};
    use tempfile;

    // 创建一个模拟的遥测数据包用于测试
    fn create_mock_packet(track_name: &str, position: Position) -> GT7TelemetryPacket {
        GT7TelemetryPacket {
            version: 1,
            game_state: crate::packet::GameState {
                state_type: GameStateType::InRace,
                race_info: None,
                is_paused: false,
                is_replay: false,
                menu_id: 0,
            },
            car_info: crate::packet::CarInfo {
                position,
                tires: TireInfo {
                    front_left: TireData { temperature: 0.0, wear: 0.0, suspension_travel: 0.0, wheel_speed: 0.0, radius: 0.0 },
                    front_right: TireData { temperature: 0.0, wear: 0.0, suspension_travel: 0.0, wheel_speed: 0.0, radius: 0.0 },
                    rear_left: TireData { temperature: 0.0, wear: 0.0, suspension_travel: 0.0, wheel_speed: 0.0, radius: 0.0 },
                    rear_right: TireData { temperature: 0.0, wear: 0.0, suspension_travel: 0.0, wheel_speed: 0.0, radius: 0.0 },
                },
                engine: EngineInfo {
                    rpm: 0.0,
                    max_rpm: 0.0,
                    throttle: 0.0,
                    brake: 0.0,
                    clutch: 0.0,
                    gear: 0,
                    suggested_gear: 0,
                    fuel_remaining: 0.0,
                    fuel_consumption: 0.0,
                    fuel_capacity: 0.0,
                    fuel_level: 0.0,
                },
                configuration: None,
            },
            track_info: TrackInfo {
                track_data: TrackData {
                    track_id: 1,
                    track_name: track_name.to_string(),
                    track_length: 5000.0,
                    altitude: 100.0,
                    weather: WeatherCondition::Clear,
                    road_temperature: 25.0,
                    air_temperature: 20.0,
                },
                current_sector: 1,
                track_wetness: 0.0,
            },
            timestamp: 0,
            packet_id: 0,
        }
    }

    #[tokio::test]
    async fn test_recorder_workflow() {
        let (sender, receiver) = broadcast::channel(100);
        let mut recorder = TrackRecorder::new(receiver);

        assert!(!recorder.is_recording());
        recorder.start_recording();
        assert!(recorder.is_recording());

        // Test racing line recording
        let pos1 = Position { world: Vector3::new(1.0, 0.0, 0.0), velocity: Vector3::new(10.0, 0.0, 0.0), angular_velocity: Vector3::new(0.0, 0.0, 0.0), rotation: Vector3::new(0.0, 0.0, 0.0) };
        let packet1 = create_mock_packet("Test Track", pos1);
        sender.send(("mock_ip".to_string(), packet1)).unwrap();
        recorder.update().await;

        // Test left border recording
        recorder.set_recording_mode(RecordingMode::LeftBorder);
        let pos2 = Position { world: Vector3::new(2.0, 0.0, 0.0), velocity: Vector3::new(10.0, 0.0, 0.0), angular_velocity: Vector3::new(0.0, 0.0, 0.0), rotation: Vector3::new(0.0, 0.0, 0.0) };
        let packet2 = create_mock_packet("Test Track", pos2);
        sender.send(("mock_ip".to_string(), packet2)).unwrap();
        recorder.update().await;

        // Test right border recording
        recorder.set_recording_mode(RecordingMode::RightBorder);
        let pos3 = Position { world: Vector3::new(3.0, 0.0, 0.0), velocity: Vector3::new(10.0, 0.0, 0.0), angular_velocity: Vector3::new(0.0, 0.0, 0.0), rotation: Vector3::new(0.0, 0.0, 0.0) };
        let packet3 = create_mock_packet("Test Track", pos3);
        sender.send(("mock_ip".to_string(), packet3)).unwrap();
        recorder.update().await;

        recorder.stop_recording();
        assert!(!recorder.is_recording());

        let recorded_track = recorder.get_recorded_track();
        assert_eq!(recorded_track.track_data.as_ref().unwrap().track_name, "Test Track");
        assert_eq!(recorded_track.racing_line.len(), 1);
        assert_eq!(recorded_track.racing_line[0], pos1);
        assert_eq!(recorded_track.left_border.len(), 1);
        assert_eq!(recorded_track.left_border[0], pos2);
        assert_eq!(recorded_track.right_border.len(), 1);
        assert_eq!(recorded_track.right_border[0], pos3);
    }

    #[test]
    fn test_save_and_load_track() {
        let track_data = TrackData {
            track_id: 123,
            track_name: "Test Track".to_string(),
            track_length: 1234.5,
            altitude: 10.0,
            weather: WeatherCondition::Clear,
            road_temperature: 25.0,
            air_temperature: 20.0,
        };

        let pos1 = Position { world: Vector3::new(1.0, 0.0, 0.0), velocity: Vector3::new(10.0, 0.0, 0.0), angular_velocity: Vector3::new(0.0, 0.0, 0.0), rotation: Vector3::new(0.0, 0.0, 0.0) };
        let pos2 = Position { world: Vector3::new(2.0, 0.0, 0.0), velocity: Vector3::new(10.0, 0.0, 0.0), angular_velocity: Vector3::new(0.0, 0.0, 0.0), rotation: Vector3::new(0.0, 0.0, 0.0) };

        let recorded_track = RecordedTrack {
            track_data: Some(track_data),
            racing_line: vec![pos1],
            left_border: vec![pos2],
            right_border: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_track.json");

        recorded_track.save_to_json_file(&file_path).unwrap();
        let loaded_track = RecordedTrack::from_json_file(&file_path).unwrap();

        assert_eq!(recorded_track.track_data, loaded_track.track_data);
        assert_eq!(recorded_track.racing_line, loaded_track.racing_line);
        assert_eq!(recorded_track.left_border, loaded_track.left_border);
        assert_eq!(recorded_track.right_border, loaded_track.right_border);
    }
}
