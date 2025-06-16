//! GT7遥测网络客户端
//! 
//! 参考gt7telemetry Python库实现UDP客户端和多IP支持

use crate::error::{Result, GT7Error};
use crate::packet::GT7TelemetryPacket;
use crate::types::TelemetryConfig;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;

/// GT7遥测客户端
/// 
/// 支持多IP连接，可同时监控多个PS4/PS5设备
pub struct GT7TelemetryClient {
    /// 客户端配置
    config: TelemetryConfig,
    /// 活动连接管理
    connections: Arc<Mutex<HashMap<String, ClientConnection>>>,
    /// 数据包广播发送器
    packet_sender: broadcast::Sender<(String, GT7TelemetryPacket)>,
    /// 客户端状态
    is_running: Arc<Mutex<bool>>,
}

/// 单个客户端连接信息
#[derive(Debug, Clone)]
struct ClientConnection {
    /// 目标地址
    address: SocketAddr,
    /// UDP套接字
    socket: Arc<UdpSocket>,
    /// 最后接收数据包时间
    last_received: Instant,
    /// 最后发送心跳时间
    last_heartbeat: Instant,
    /// 连接状态
    is_connected: bool,
    /// 接收到的数据包计数
    packet_count: u64,
}

impl GT7TelemetryClient {
    /// 创建新的GT7遥测客户端
    /// 
    /// # 参数
    /// 
    /// * `config` - 客户端配置
    /// 
    /// # 返回
    /// 
    /// 新的客户端实例和数据包接收器
    pub fn new(config: TelemetryConfig) -> Result<(Self, broadcast::Receiver<(String, GT7TelemetryPacket)>)> {
        let (packet_sender, packet_receiver) = broadcast::channel(1000);
        
        let client = Self {
            config,
            connections: Arc::new(Mutex::new(HashMap::new())),
            packet_sender,
            is_running: Arc::new(Mutex::new(false)),
        };

        Ok((client, packet_receiver))
    }

    /// 添加GT7设备连接
    /// 
    /// # 参数
    /// 
    /// * `ip` - GT7设备IP地址
    /// * `port` - 可选端口（默认使用配置中的端口）
    pub async fn add_connection(&self, ip: String, port: Option<u16>) -> Result<()> {
        // 验证IP地址格式
        if !crate::is_valid_gt7_ip(&ip) {
            return Err(GT7Error::invalid_ip(ip));
        }

        let port = port.unwrap_or(self.config.port);
        if port == 0 {
            return Err(GT7Error::invalid_port(port));
        }

        let address: SocketAddr = format!("{}:{}", ip, port).parse()
            .map_err(|_| GT7Error::invalid_ip(&ip))?;

        // 创建UDP套接字
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| GT7Error::network_error(&address.to_string(), e.to_string()))?;

        socket.set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| GT7Error::network_error(&address.to_string(), e.to_string()))?;

        let connection = ClientConnection {
            address,
            socket: Arc::new(socket),
            last_received: Instant::now(),
            last_heartbeat: Instant::now(),
            is_connected: false,
            packet_count: 0,
        };

        let mut connections = self.connections.lock().unwrap();
        connections.insert(ip.clone(), connection);
        
        log::info!("添加GT7连接: {} -> {}", ip, address);
        Ok(())
    }

    /// 移除GT7设备连接
    /// 
    /// # 参数
    /// 
    /// * `ip` - 要移除的GT7设备IP地址
    pub async fn remove_connection(&self, ip: &str) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(_) = connections.remove(ip) {
            log::info!("移除GT7连接: {}", ip);
            Ok(())
        } else {
            Err(GT7Error::network_error(ip, "连接不存在".to_string()))
        }
    }

    /// 获取所有连接状态
    pub fn get_connection_status(&self) -> HashMap<String, bool> {
        let connections = self.connections.lock().unwrap();
        connections.iter()
            .map(|(ip, conn)| (ip.clone(), conn.is_connected))
            .collect()
    }

    /// 启动客户端
    /// 
    /// 开始监听所有连接的数据包和发送心跳
    pub async fn start(&self) -> Result<()> {
        {
            let mut is_running = self.is_running.lock().unwrap();
            if *is_running {
                return Err(GT7Error::config_error(
                    "client_state", 
                    "running", 
                    "客户端已在运行"
                ));
            }
            *is_running = true;
        }

        log::info!("启动GT7遥测客户端...");

        // 启动数据包接收任务
        let connections_clone = Arc::clone(&self.connections);
        let packet_sender_clone = self.packet_sender.clone();
        let is_running_clone = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            Self::packet_receiver_task(connections_clone, packet_sender_clone, is_running_clone).await;
        });

        // 启动心跳发送任务
        let connections_clone = Arc::clone(&self.connections);
        let heartbeat_interval = self.config.heartbeat_interval;
        let is_running_clone = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            Self::heartbeat_sender_task(connections_clone, heartbeat_interval, is_running_clone).await;
        });

        // 启动连接监控任务
        let connections_clone = Arc::clone(&self.connections);
        let timeout_duration = Duration::from_secs(self.config.timeout);
        let is_running_clone = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            Self::connection_monitor_task(connections_clone, timeout_duration, is_running_clone).await;
        });

        Ok(())
    }

    /// 停止客户端
    pub async fn stop(&self) {
        {
            let mut is_running = self.is_running.lock().unwrap();
            *is_running = false;
        }
        
        log::info!("停止GT7遥测客户端");
    }

    /// 数据包接收任务
    async fn packet_receiver_task(
        connections: Arc<Mutex<HashMap<String, ClientConnection>>>,
        packet_sender: broadcast::Sender<(String, GT7TelemetryPacket)>,
        is_running: Arc<Mutex<bool>>,
    ) {
        let mut buffer = [0u8; crate::GT7_PACKET_SIZE * 2]; // 留点余量
        
        while *is_running.lock().unwrap() {
            let connections_map = {
                let guard = connections.lock().unwrap();
                guard.clone()
            };

            for (ip, mut connection) in connections_map {
                match connection.socket.recv(&mut buffer) {
                    Ok(size) => {
                        if size >= crate::GT7_PACKET_SIZE {
                            match GT7TelemetryPacket::from_bytes(&buffer[..crate::GT7_PACKET_SIZE]) {
                                Ok(packet) => {
                                    // 验证数据包
                                    if packet.validate().is_ok() {
                                        connection.last_received = Instant::now();
                                        connection.is_connected = true;
                                        connection.packet_count += 1;

                                        // 更新连接状态
                                        {
                                            let mut connections_guard = connections.lock().unwrap();
                                            connections_guard.insert(ip.clone(), connection.clone());
                                        }

                                        // 广播数据包
                                        if packet_sender.send((ip.clone(), packet)).is_err() {
                                            log::warn!("数据包广播队列已满，跳过数据包");
                                        }

                                        log::debug!("接收到来自 {} 的数据包 #{}", ip, connection.packet_count);
                                    } else {
                                        log::warn!("来自 {} 的数据包验证失败", ip);
                                    }
                                }
                                Err(e) => {
                                    log::warn!("解析来自 {} 的数据包失败: {}", ip, e);
                                }
                            }
                        } else {
                            log::warn!("来自 {} 的数据包大小不正确: {} 字节", ip, size);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // 超时，继续尝试其他连接
                        continue;
                    }
                    Err(e) => {
                        log::warn!("从 {} 接收数据包时出错: {}", ip, e);
                    }
                }
            }

            // 短暂休眠避免过度占用CPU
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    /// 心跳发送任务
    async fn heartbeat_sender_task(
        connections: Arc<Mutex<HashMap<String, ClientConnection>>>,
        heartbeat_interval_ms: u64,
        is_running: Arc<Mutex<bool>>,
    ) {
        let mut interval = interval(Duration::from_millis(heartbeat_interval_ms));
        
        while *is_running.lock().unwrap() {
            interval.tick().await;

            let connections_map = {
                let guard = connections.lock().unwrap();
                guard.clone()
            };

            for (ip, mut connection) in connections_map {
                let now = Instant::now();
                if now.duration_since(connection.last_heartbeat) >= Duration::from_millis(heartbeat_interval_ms) {
                    match connection.socket.send_to(crate::GT7_HEARTBEAT, connection.address) {
                        Ok(_) => {
                            connection.last_heartbeat = now;
                            
                            // 更新连接状态
                            {
                                let mut connections_guard = connections.lock().unwrap();
                                connections_guard.insert(ip.clone(), connection);
                            }
                            
                            log::debug!("发送心跳到 {}", ip);
                        }
                        Err(e) => {
                            log::warn!("发送心跳到 {} 失败: {}", ip, e);
                        }
                    }
                }
            }
        }
    }

    /// 连接监控任务
    async fn connection_monitor_task(
        connections: Arc<Mutex<HashMap<String, ClientConnection>>>,
        timeout_duration: Duration,
        is_running: Arc<Mutex<bool>>,
    ) {
        let mut interval = interval(Duration::from_secs(1));
        
        while *is_running.lock().unwrap() {
            interval.tick().await;

            let mut connections_guard = connections.lock().unwrap();
            let now = Instant::now();
            
            for (ip, connection) in connections_guard.iter_mut() {
                let time_since_last = now.duration_since(connection.last_received);
                if time_since_last > timeout_duration && connection.is_connected {
                    connection.is_connected = false;
                    log::warn!("GT7设备 {} 连接超时 ({}秒)", ip, time_since_last.as_secs());
                }
            }
        }
    }
}

/// 简化的单IP客户端
/// 
/// 用于只需要连接一个GT7设备的场景
pub struct SimpleGT7Client {
    client: GT7TelemetryClient,
    ip: String,
}

impl SimpleGT7Client {
    /// 创建简单的单IP客户端
    pub async fn new(ip: String, port: Option<u16>) -> Result<(Self, broadcast::Receiver<(String, GT7TelemetryPacket)>)> {
        let config = TelemetryConfig {
            console_ip: ip.clone(),
            port: port.unwrap_or(crate::GT7_TELEMETRY_PORT),
            ..Default::default()
        };

        let (client, receiver) = GT7TelemetryClient::new(config)?;
        client.add_connection(ip.clone(), port).await?;

        Ok((Self { client, ip }, receiver))
    }

    /// 启动客户端
    pub async fn start(&self) -> Result<()> {
        self.client.start().await
    }

    /// 停止客户端
    pub async fn stop(&self) {
        self.client.stop().await
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.client.get_connection_status()
            .get(&self.ip)
            .copied()
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = TelemetryConfig::default();
        let result = GT7TelemetryClient::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_connection() {
        let config = TelemetryConfig::default();
        let (client, _) = GT7TelemetryClient::new(config).unwrap();
        
        let result = client.add_connection("192.168.1.30".to_string(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_ip() {
        let config = TelemetryConfig::default();
        let (client, _) = GT7TelemetryClient::new(config).unwrap();
        
        let result = client.add_connection("invalid.ip".to_string(), None).await;
        assert!(result.is_err());
    }
}