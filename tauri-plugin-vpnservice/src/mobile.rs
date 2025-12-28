use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.vpnservice";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_vpnservice);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Vpnservice<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "VpnServicePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_vpnservice)?;
    Ok(Vpnservice(handle))
}

/// Access to the vpnservice APIs.
pub struct Vpnservice<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Vpnservice<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        self.0
            .run_mobile_plugin("ping", payload)
            .map_err(Into::into)
    }

    pub fn prepare_vpn(&self, payload: VoidRequest) -> crate::Result<Status> {
        self.0
            .run_mobile_plugin("prepare_vpn", payload)
            .map_err(Into::into)
    }

    pub fn start_vpn(&self, payload: StartVpnRequest) -> crate::Result<Status> {
        self.0
            .run_mobile_plugin("start_vpn", payload)
            .map_err(Into::into)
    }

    pub fn stop_vpn(&self, payload: VoidRequest) -> crate::Result<Status> {
        self.0
            .run_mobile_plugin("stop_vpn", payload)
            .map_err(Into::into)
    }
}

#[cfg(target_os = "android")]
pub fn protect_fd(fd: i32) -> bool {
    use jni::objects::JValue;

    let ctx = ndk_context::android_context();
    let vm_ptr = ctx.vm();

    // 安全构造 JavaVM
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr.cast()) };
    let vm = match vm {
        Ok(vm) => vm,
        Err(e) => {
            eprintln!("protect_fd: JavaVM::from_raw failed: {e}");
            return false;
        }
    };

    // attach_current_thread 返回 AttachGuard，它会自动解引用为 JNIEnv
    let mut env = match vm.attach_current_thread() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("protect_fd: attach_current_thread failed: {e}");
            return false;
        }
    };

    let class = match env.find_class("com/plugin/vpnservice/TauriVpnService") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("protect_fd: find_class failed: {e}");
            return false;
        }
    };

    let result = env
        .call_static_method(class, "protectFd", "(I)Z", &[JValue::from(fd)])
        .and_then(|v| v.z());

    match result {
        Ok(b) => b,
        Err(e) => {
            eprintln!("protect_fd: call_static_method failed: {e}");
            false
        }
    }
}