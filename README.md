# ClubmanSharp - Gran Turismo自动驾驶机器人

## 项目概述

ClubmanSharp是一个Gran Turismo自动驾驶机器人项目，包含C#原版和Rust重写版本。

### 技术栈
- **Rust版本**: 使用Slint UI + ViGEm/IOKit虚拟控制器
- **C#版本**: WPF + ViGEm虚拟控制器

## 当前任务进度

### 已完成任务 ✅
- ✅ 创建rust-vgamepad库基础架构
- ✅ 实现DualShock4控制器数据结构
- ✅ 实现错误处理系统
- ✅ Windows ViGEm基础集成
- ✅ 完善rust-vgamepad库Windows实现
- ✅ 清理编译警告和未使用代码
- ✅ 实现macOS IOKit HID支持（基础框架）
- ✅ 完善GT7遥测数据解析库
- ✅ 清理gt7-telemetry库编译警告
- ✅ 创建现代化Slint UI界面
- ✅ 实现选项卡式布局和实时数据可视化
- ✅ 集成UI回调和事件处理
- ✅ 完善设备发现功能 - 基于chiaki-ng的精确PS4/PS5识别机制
- ✅ 实现设备过滤系统 - 支持隐藏设备、PlayStation专用过滤等
- ✅ **现代化设备卡片UI** - 美观的卡片式设计，显示详细设备信息
- ✅ **多选设备功能** - 支持复选框多选、全选、批量操作
- ✅ **设备管理界面** - 连接、隐藏、删除设备的完整UI交互
- ✅ **实时状态显示** - 在线/睡眠/离线状态，GT7活跃检测，响应时间

### 正在进行的任务 🔄
- 实现自动驾驶核心算法
- 完善设备连接稳定性

### 下一步任务 📋
- 实现设备状态实时同步
- 添加设备连接历史记录
- 优化网络扫描性能

### 未来待开发 🚀
- 高级自动驾驶算法
- 赛道数据管理系统
- 多设备并发控制
- UI界面优化和可视化
- 性能监控和分析
- 机器学习优化
- 云端数据同步

## 架构说明

### Rust版本项目结构
```
├── clubman-sharp-rust/     # 主应用程序
├── rust-vgamepad/         # 虚拟游戏手柄库
├── gt7-telemetry/         # GT7遥测数据库
└── README.md              # 项目文档
```

### 核心组件
- **rust-vgamepad**: 跨平台虚拟控制器库，支持Windows（ViGEm）和macOS（IOKit）
- **gt7-telemetry**: GT7游戏遥测数据解析和网络通信
- **clubman-sharp-rust**: 主应用程序，集成UI和自动驾驶逻辑

## 开发指南

### 构建项目
```bash
# 构建整个工作空间
cargo build

# 构建发布版本
cargo build --release

# 运行主应用程序
cargo run --bin clubman-sharp-rust
```

### 测试
```bash
# 运行特定库的测试
cargo test -p rust-vgamepad
cargo test -p gt7-telemetry
```

### 代码质量检查
```bash
# 运行clippy检查
cargo clippy

# 格式化代码
cargo fmt
``` 