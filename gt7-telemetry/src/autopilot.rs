//! 自动驾驶模块
//!
//! 根据录制的赛道数据，控制虚拟手柄实现自动驾驶

use crate::recorder::RecordedTrack;
use crate::types::Position;
use rust_vgamepad::{VGamepadClient, DualShock4Controller};

const LOOK_AHEAD_DISTANCE: f32 = 5.0; // (m)
const STEERING_P_GAIN: f32 = 1.5;
const MAX_SPEED_ON_STRAIGHT: f32 = 200.0 / 3.6; // (m/s)
const MAX_SPEED_IN_CORNER: f32 = 80.0 / 3.6; // (m/s)

/// 自动驾驶系统
pub struct Autopilot {
    track: RecordedTrack,
    vgamepad: DualShock4Controller,
    current_target_index: usize,
}

impl Autopilot {
    /// 创建新的自动驾驶实例
    pub fn new(track: RecordedTrack) -> Result<Self, rust_vgamepad::VGamepadError> {
        let client = VGamepadClient::new()?;
        let vgamepad = client.create_dualshock4()?;
        Ok(Self {
            track,
            vgamepad,
            current_target_index: 0,
        })
    }

    /// 根据当前车辆位置和目标路径，更新控制器状态
    pub fn update(&mut self, current_position: &Position, current_speed: f32) {
        if self.track.racing_line.is_empty() {
            return;
        }

        // 1. 找到目标点 (look-ahead)
        self.update_target_index(current_position);
        let target_point = &self.track.racing_line[self.current_target_index];

        // 2. 计算转向
        let steering = self.calculate_steering(current_position, target_point);
        self.vgamepad.set_left_joystick(steering, 0.0).unwrap();

        // 3. 计算油门和刹车
        let (throttle, brake) = self.calculate_throttle_brake(current_speed);
        self.vgamepad.set_right_trigger(throttle).unwrap();
        self.vgamepad.set_left_trigger(brake).unwrap();

        // 4. 更新手柄
        self.vgamepad.update().unwrap();
    }

    /// 更新目标点的索引
    fn update_target_index(&mut self, current_position: &Position) {
        let mut closest_dist_sq = f32::MAX;
        let mut closest_index = self.current_target_index;

        // 从上一个目标点开始搜索，找到最近的点
        for i in self.current_target_index..self.track.racing_line.len() {
            let dist_sq = Self::dist_sq(&current_position.world, &self.track.racing_line[i].world);
            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest_index = i;
            }
        }

        // 从最近的点开始，向前找到第一个超过look-ahead距离的点作为新目标
        let mut target_index = closest_index;
        for i in closest_index..self.track.racing_line.len() {
            let dist_sq = Self::dist_sq(&current_position.world, &self.track.racing_line[i].world);
            if dist_sq > LOOK_AHEAD_DISTANCE * LOOK_AHEAD_DISTANCE {
                target_index = i;
                break;
            }
        }
        self.current_target_index = target_index;
    }

    /// 计算转向值 (-1.0 to 1.0)
    fn calculate_steering(&self, current_position: &Position, target_point: &Position) -> f32 {
        // 车辆朝向向量 (2D)
        let car_heading = (current_position.rotation.y.sin(), current_position.rotation.y.cos());

        // 目标方向向量 (2D)
        let target_dir = (
            target_point.world.x - current_position.world.x,
            target_point.world.z - current_position.world.z
        );

        // 计算航向误差角度
        let angle_error = car_heading.0 * target_dir.1 - car_heading.1 * target_dir.0;

        // P-Controller
        let steering = (angle_error * STEERING_P_GAIN).max(-1.0).min(1.0);
        steering
    }

    /// 计算油门和刹车值 (0.0 to 1.0)
    fn calculate_throttle_brake(&self, current_speed: f32) -> (f32, f32) {
        let curvature = self.get_path_curvature();
        let target_speed = if curvature > 0.1 {
            MAX_SPEED_IN_CORNER
        } else {
            MAX_SPEED_ON_STRAIGHT
        };

        if current_speed < target_speed {
            (1.0, 0.0) // 全油门
        } else {
            (0.0, 1.0) // 全刹车
        }
    }

    /// 获取当前目标点附近的路径曲率
    fn get_path_curvature(&self) -> f32 {
        if self.current_target_index < 1 || self.current_target_index >= self.track.racing_line.len() - 1 {
            return 0.0;
        }

        let p1 = &self.track.racing_line[self.current_target_index - 1].world;
        let p2 = &self.track.racing_line[self.current_target_index].world;
        let p3 = &self.track.racing_line[self.current_target_index + 1].world;

        let v1 = (p2.x - p1.x, p2.z - p1.z);
        let v2 = (p3.x - p2.x, p3.z - p2.z);

        // 用向量夹角的sin值来近似曲率
        let cross_product = v1.0 * v2.1 - v1.1 * v2.0;
        let dot_product = v1.0 * v2.0 + v1.1 * v2.1;

        if dot_product.abs() < 1e-6 { return 1.0; } // 90度角

        (cross_product / dot_product).abs()
    }

    /// 计算两个点之间的距离平方
    fn dist_sq(p1: &crate::types::Vector3, p2: &crate::types::Vector3) -> f32 {
        (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2) + (p1.z - p2.z).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recorder::RecordedTrack;
    use crate::types::{Position, Vector3};

    #[test]
    #[cfg(any(windows, target_os = "macos"))]
    fn test_autopilot_steering() {
        let track = RecordedTrack {
            racing_line: vec![
                Position { world: Vector3::new(0.0, 0.0, 0.0), rotation: Vector3::new(0.0, 0.0, 0.0), velocity: Vector3::new(0.0,0.0,0.0), angular_velocity: Vector3::new(0.0,0.0,0.0) },
                Position { world: Vector3::new(0.0, 0.0, 10.0), rotation: Vector3::new(0.0, 0.0, 0.0), velocity: Vector3::new(0.0,0.0,0.0), angular_velocity: Vector3::new(0.0,0.0,0.0) },
                Position { world: Vector3::new(5.0, 0.0, 20.0), rotation: Vector3::new(0.0, 0.0, 0.0), velocity: Vector3::new(0.0,0.0,0.0), angular_velocity: Vector3::new(0.0,0.0,0.0) },
            ],
            ..Default::default()
        };

        let mut autopilot = Autopilot::new(track).unwrap();

        // Car is at origin, heading straight
        let current_position = Position {
            world: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 10.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
        };

        autopilot.update(&current_position, 10.0);

        let report = autopilot.vgamepad.get_state().report;
        // Should be steering right to get to the next point
        assert!(report.left_thumb_x > 128);
    }
}
