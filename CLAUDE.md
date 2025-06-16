# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 请完全使用中文回答

## Project Overview

ClubmanSharp是一个Gran Turismo自动驾驶机器人项目，包含C#原版和Rust重写版本。

### C#版本 (clubmanSharp/)
原始的C# WPF应用程序，使用Simulator Interface API自动化GT6/Sport/7比赛。通过网络连接到游戏，接收遥测数据，并控制虚拟DualShock4控制器进行自动驾驶。

### Rust版本 (rust项目/)
现代化的Rust重写版本，具有以下特性：
- **跨平台支持**: Windows + macOS
- **多客户端架构**: 支持同时控制多个GT7实例
- **现代UI**: 使用Slint框架的现代化界面
- **类型安全**: Rust的类型系统确保安全性
- **模块化设计**: 分离的库和清晰的架构

## Architecture

### Core Components

- **Bot.cs**: Main automation logic that handles game state detection, controller input simulation, and race execution
- **MainWindow.xaml/.cs**: WPF UI for configuring connection settings, delays, and monitoring bot status
- **TrackData/**: Contains track-specific racing lines and speed data for different circuits
  - `TrackDataBase`: Abstract base class defining segment-based track data structure
  - `GTOTrackData.cs`: Specific track data for Grand Valley East circuit
  - `TrackDataReader.cs`: Handles loading and parsing track data files

### PDTools Library Integration

The project heavily relies on the PDTools library ecosystem:
- **PDTools.SimulatorInterface**: Handles network communication with Gran Turismo games
- **PDTools.Crypto**: Encryption utilities for GT game data
- Additional PDTools modules for file formats, save files, and game structures

### Controller Simulation

Uses ViGEm (Nefarius.ViGEm.Client) to create a virtual DualShock4 controller that the game recognizes as a real controller.

## Build and Development

### C#版本构建命令
```bash
# Build the solution
dotnet build clubmanSharp/ClubmanSharp.sln

# Build specific configuration
dotnet build clubmanSharp/ClubmanSharp.sln --configuration Release

# Run the application
dotnet run --project clubmanSharp/ClubmanSharp.csproj
```

### Rust版本构建命令
```bash
# 构建整个工作空间
cargo build

# 构建发布版本
cargo build --release

# 运行主应用程序
cargo run --bin clubman-sharp-rust

# 运行特定库的测试
cargo test -p rust-vgamepad
cargo test -p gt7-telemetry

# 无头模式运行（指定IP）
cargo run --bin clubman-sharp-rust -- --ip 192.168.1.30 --headless
```

### Dependencies
- .NET 6.0 Windows (WPF)
- Nefarius.ViGEm.Client for controller emulation
- Syroot.BinaryData for binary data handling
- PDTools.SimulatorInterface for game communication

### Project Structure
- Main application: `clubmanSharp/ClubmanSharp.csproj`
- Solution file: `clubmanSharp/ClubmanSharp.sln`
- Referenced PDTools libraries are included as subprojects

## Key Implementation Details

### Track Data System
Track data is defined as arrays of `Segment` structures containing:
- Bounding box coordinates (minX, minZ, maxX, maxZ)
- Target heading and speed (mph) for optimal racing line
- Separate arrays for initial race segments vs normal lap segments

### Bot Logic Flow
1. Connect to Gran Turismo via Simulator Interface (default IP: 192.168.1.30)
2. Monitor game state through telemetry packets
3. Match current position to track segments
4. Calculate steering and throttle inputs based on segment targets
5. Send controller inputs via virtual DualShock4

### Configuration
Settings are stored in `Settings.settings` and include:
- Network IP for GT connection
- Delay timings for different console types (PS4/PS5)
- Throttle limits and custom delay values
- Button mappings (Cross/Circle for different regions)

## Rust版本架构

### 项目结构
```
├── Cargo.toml                    # 工作空间配置
├── clubman-sharp-rust/           # 主应用程序
│   ├── src/
│   │   ├── main.rs              # 程序入口
│   │   ├── app.rs               # 应用核心逻辑
│   │   ├── config.rs            # 配置管理
│   │   ├── controller.rs        # 控制器管理
│   │   └── telemetry.rs         # 遥测数据处理
│   ├── ui/main_window.slint     # Slint UI定义
│   └── build.rs                 # 构建脚本
├── rust-vgamepad/               # 虚拟游戏手柄库
│   └── src/
│       ├── lib.rs               # 库入口
│       ├── error.rs             # 错误定义
│       ├── controller.rs        # DualShock4控制器
│       ├── windows.rs           # Windows ViGEm实现
│       └── macos.rs             # macOS IOKit实现
└── gt7-telemetry/               # GT7遥测库
    └── src/
        ├── lib.rs               # 库入口
        ├── error.rs             # 错误定义
        ├── packet.rs            # 数据包解析
        ├── client.rs            # 网络客户端
        └── types.rs             # 数据类型定义
```

### 核心特性

#### 错误处理
- **库层面**: 使用`thiserror`定义结构化错误类型
- **应用层面**: 使用`anyhow`进行错误传播和上下文管理
- **错误转换**: 库错误自动转换为`anyhow::Error`

#### 多客户端支持
- 同时连接多个PS4/PS5设备
- 每个设备独立的控制器实例
- 统一的遥测数据处理

#### 配置管理
- TOML格式配置文件
- 用户配置目录自动创建
- 运行时配置验证和更新

#### UI框架
- 使用Slint的现代化跨平台UI
- 响应式设计，支持实时数据更新
- 多标签页界面（控制面板、实时数据、日志、关于）