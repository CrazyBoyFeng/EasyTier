#[cfg(target_os = "android")]
use std::os::fd::RawFd;

#[cfg(target_os = "android")]
use once_cell::sync::{Lazy, OnceCell};

#[cfg(target_os = "android")]
use crate::tunnel::TunnelError;

/// Android 平台上的 socket 保护接口，用于调用 VpnService.protect(fd)
#[cfg(target_os = "android")]
pub trait SocketProtector: Send + Sync + 'static {
    /// 返回 true 表示保护成功或认为无需保护
    fn protect(&self, fd: RawFd) -> bool;
}

/// 全局 SocketProtector 实例
#[cfg(target_os = "android")]
static PROTECTOR: OnceCell<Box<dyn SocketProtector>> = OnceCell::new();

/// 是否启用保护逻辑的开关，由上层根据 bind_device 配置控制
#[cfg(target_os = "android")]
pub static ENABLED: Lazy<std::sync::atomic::AtomicBool> = Lazy::new(|| {
    std::sync::atomic::AtomicBool::new(false)
});

/// 注册 SocketProtector，仅第一次生效
#[cfg(target_os = "android")]
pub fn set_socket_protector(p: Box<dyn SocketProtector>) {
    let _ = PROTECTOR.set(p);
}

/// 打开或关闭保护逻辑
#[cfg(target_os = "android")]
pub fn set_enabled(enabled: bool) {
    use std::sync::atomic::Ordering;
    ENABLED.store(enabled, Ordering::SeqCst);
}

/// 当前是否启用保护逻辑
#[cfg(target_os = "android")]
pub fn is_enabled() -> bool {
    use std::sync::atomic::Ordering;
    ENABLED.load(Ordering::SeqCst)
}

/// 在 Android 上对给定 fd 调用已注册的 SocketProtector
#[cfg(target_os = "android")]
pub fn protect_fd(fd: RawFd) -> Result<(), TunnelError> {
    if !is_enabled() {
        return Ok(());
    }

    if let Some(p) = PROTECTOR.get() {
        if p.protect(fd) {
            Ok(())
        } else {
            Err(TunnelError::InternalError(
                "VpnService.protect() failed".to_string(),
            ))
        }
    } else {
        // 未注册具体实现时保持兼容性，直接认为成功
        Ok(())
    }
}
