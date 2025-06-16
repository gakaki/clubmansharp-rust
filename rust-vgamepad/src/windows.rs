//! Windows平台实现 - ViGEm集成
//! 
//! 参考nefarius/ViGEmBus和vgamepad的Windows实现

use crate::controller::{DS4ControllerState, DS4Report};
use crate::error::{Result, VGamepadError};
use std::ffi::CString;
use std::ptr;
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

/// ViGEm客户端句柄类型
type PVIGEM_CLIENT = *mut std::ffi::c_void;
/// ViGEm目标句柄类型
type PVIGEM_TARGET = *mut std::ffi::c_void;

/// ViGEm错误代码 (参考ViGEmBus/Common.h)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViGEmError {
    None = 0x20000000,
    BusNotFound = 0xE0000001,
    NoFreeSlot = 0xE0000002,
    InvalidTarget = 0xE0000003,
    RemovalFailed = 0xE0000004,
    AlreadyConnected = 0xE0000005,
    TargetUninitialized = 0xE0000006,
    TargetNotPluggedIn = 0xE0000007,
    BusVersionMismatch = 0xE0000008,
    BusAccessFailed = 0xE0000009,
    CallbackAlreadyRegistered = 0xE000000A,
    CallbackNotFound = 0xE000000B,
    UnknownUsbDevice = 0xE000000C,
    IllegalArgument = 0xE000000D,
    XusbUserIndexOutOfRange = 0xE000000E,
    InvalidParameter = 0xE000000F,
    NotSupported = 0xE0000010,
}

/// ViGEm目标类型
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ViGEmTargetType {
    Xbox360Wired = 0,
    XboxOneWired = 1,
    DualShock4Wired = 2,
}

/// ViGEm函数指针类型定义
type FnViGEmClientAlloc = unsafe extern "C" fn() -> PVIGEM_CLIENT;
type FnViGEmClientFree = unsafe extern "C" fn(PVIGEM_CLIENT);
type FnViGEmClientConnect = unsafe extern "C" fn(PVIGEM_CLIENT) -> u32;
type FnViGEmClientDisconnect = unsafe extern "C" fn(PVIGEM_CLIENT);
type FnViGEmTargetAlloc = unsafe extern "C" fn(ViGEmTargetType) -> PVIGEM_TARGET;
type FnViGEmTargetFree = unsafe extern "C" fn(PVIGEM_TARGET);
type FnViGEmTargetAdd = unsafe extern "C" fn(PVIGEM_CLIENT, PVIGEM_TARGET) -> u32;
type FnViGEmTargetRemove = unsafe extern "C" fn(PVIGEM_CLIENT, PVIGEM_TARGET) -> u32;
type FnViGEmTargetDS4Update = unsafe extern "C" fn(PVIGEM_CLIENT, PVIGEM_TARGET, *const u8) -> u32;

/// ViGEm动态库函数表
struct ViGEmFunctions {
    client_alloc: FnViGEmClientAlloc,
    client_free: FnViGEmClientFree,
    client_connect: FnViGEmClientConnect,
    client_disconnect: FnViGEmClientDisconnect,
    target_alloc: FnViGEmTargetAlloc,
    target_free: FnViGEmTargetFree,
    target_add: FnViGEmTargetAdd,
    target_remove: FnViGEmTargetRemove,
    target_ds4_update: FnViGEmTargetDS4Update,
}

/// Windows ViGEm客户端
pub struct WindowsClient {
    /// ViGEm动态库句柄
    lib_handle: HANDLE,
    /// ViGEm函数表
    functions: ViGEmFunctions,
    /// ViGEm客户端句柄
    client_handle: PVIGEM_CLIENT,
}

impl WindowsClient {
    /// 创建新的Windows客户端
    pub fn new() -> Result<Self> {
        log::info!("正在加载ViGEmClient.dll...");
        
        // 加载ViGEmClient动态库
        let lib_name = CString::new("ViGEmClient.dll")
            .map_err(|e| VGamepadError::vigem_library_error(format!("无法创建库名称: {}", e)))?;
        
        let lib_handle = unsafe { LoadLibraryA(windows::core::PCSTR(lib_name.as_ptr() as *const u8)) };
        
        if lib_handle.is_invalid() {
            return Err(VGamepadError::driver_not_installed(
                "ViGEm总线驱动程序",
                "https://github.com/nefarius/ViGEmBus/releases"
            ));
        }
        
        // 获取函数地址
        let functions = unsafe { Self::load_functions(lib_handle)? };
        
        // 分配客户端
        let client_handle = unsafe { (functions.client_alloc)() };
        if client_handle.is_null() {
            return Err(VGamepadError::vigem_function_error(
                "vigem_alloc",
                "无法分配ViGEm客户端内存"
            ));
        }
        
        // 连接到ViGEm总线
        let result = unsafe { (functions.client_connect)(client_handle) };
        if result != ViGEmError::None as u32 {
            unsafe { (functions.client_free)(client_handle) };
            return Err(VGamepadError::vigem_error(
                "无法连接到ViGEm总线",
                result
            ));
        }
        
        log::info!("ViGEm客户端初始化成功");
        
        Ok(Self {
            lib_handle,
            functions,
            client_handle,
        })
    }
    
    /// 加载ViGEm函数
    unsafe fn load_functions(lib_handle: HANDLE) -> Result<ViGEmFunctions> {
        macro_rules! get_proc_addr {
            ($name:expr) => {{
                let name_cstr = CString::new($name)
                    .map_err(|e| VGamepadError::vigem_library_error(format!("无法创建函数名: {}", e)))?;
                let addr = GetProcAddress(lib_handle, windows::core::PCSTR(name_cstr.as_ptr() as *const u8));
                if addr.is_none() {
                    return Err(VGamepadError::vigem_function_error(
                        $name, 
                        "在ViGEmClient.dll中找不到此函数"
                    ));
                }
                std::mem::transmute(addr.unwrap())
            }};
        }
        
        Ok(ViGEmFunctions {
            client_alloc: get_proc_addr!("vigem_alloc"),
            client_free: get_proc_addr!("vigem_free"),
            client_connect: get_proc_addr!("vigem_connect"),
            client_disconnect: get_proc_addr!("vigem_disconnect"),
            target_alloc: get_proc_addr!("vigem_target_alloc"),
            target_free: get_proc_addr!("vigem_target_free"),
            target_add: get_proc_addr!("vigem_target_add"),
            target_remove: get_proc_addr!("vigem_target_remove"),
            target_ds4_update: get_proc_addr!("vigem_target_ds4_update"),
        })
    }
}

impl Drop for WindowsClient {
    fn drop(&mut self) {
        log::info!("正在清理ViGEm客户端...");
        
        unsafe {
            if !self.client_handle.is_null() {
                (self.functions.client_disconnect)(self.client_handle);
                (self.functions.client_free)(self.client_handle);
            }
        }
    }
}

/// Windows DualShock4控制器
pub struct WindowsDS4Controller {
    /// ViGEm目标句柄
    target_handle: PVIGEM_TARGET,
    /// 客户端引用
    client: *const WindowsClient,
}

impl WindowsDS4Controller {
    /// 创建新的Windows DS4控制器
    pub fn new(client: &WindowsClient) -> Result<Self> {
        log::info!("正在创建DS4虚拟控制器...");
        
        // 分配DS4目标
        let target_handle = unsafe { 
            (client.functions.target_alloc)(ViGEmTargetType::DualShock4Wired)
        };
        
        if target_handle.is_null() {
            return Err(VGamepadError::vigem_error("无法分配DS4目标", 0));
        }
        
        // 添加目标到ViGEm总线
        let result = unsafe { 
            (client.functions.target_add)(client.client_handle, target_handle)
        };
        
        if result != ViGEmError::None as u32 {
            unsafe { (client.functions.target_free)(target_handle) };
            return Err(VGamepadError::vigem_error(format!(
                "无法添加DS4目标，错误代码: 0x{:08X}",
                result
            )));
        }
        
        log::info!("DS4虚拟控制器创建成功");
        
        Ok(Self {
            target_handle,
            client: client as *const WindowsClient,
        })
    }
    
    /// 更新控制器状态
    pub fn update(&mut self, state: &DS4ControllerState) -> Result<()> {
        let client = unsafe { &*self.client };
        
        let result = unsafe {
            (client.functions.target_ds4_update)(
                client.client_handle,
                self.target_handle,
                &state.report as *const _ as *const u8,
            )
        };
        
        if result != ViGEmError::None as u32 {
            return Err(VGamepadError::vigem_error(format!(
                "更新DS4状态失败，错误代码: 0x{:08X}",
                result
            )));
        }
        
        Ok(())
    }
}

impl Drop for WindowsDS4Controller {
    fn drop(&mut self) {
        log::info!("正在移除DS4虚拟控制器...");
        
        let client = unsafe { &*self.client };
        
        unsafe {
            (client.functions.target_remove)(client.client_handle, self.target_handle);
            (client.functions.target_free)(self.target_handle);
        }
    }
}

