#[cfg(windows)]
pub struct NativeWatcher {
    handle: windows::Win32::Devices::DeviceAndDriverInstallation::HCMNOTIFICATION,
    context: *mut tokio::sync::mpsc::UnboundedSender<()>,
}

#[cfg(windows)]
impl NativeWatcher {
    pub fn register(sender: tokio::sync::mpsc::UnboundedSender<()>) -> Result<Self, String> {
        use windows::Win32::Devices::DeviceAndDriverInstallation::*;
        let context = Box::into_raw(Box::new(sender));
        let filter = CM_NOTIFY_FILTER {
            cbSize: std::mem::size_of::<CM_NOTIFY_FILTER>() as u32,
            Flags: CM_NOTIFY_FILTER_FLAG_ALL_INTERFACE_CLASSES,
            FilterType: CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE,
            Reserved: 0,
            u: CM_NOTIFY_FILTER_0::default(),
        };
        let mut handle = HCMNOTIFICATION::default();
        let result = unsafe {
            CM_Register_Notification(&filter, Some(context.cast()), Some(callback), &mut handle)
        };
        if result.0 != 0 {
            unsafe { drop(Box::from_raw(context)) };
            return Err(format!("CM_Register_Notification={}", result.0));
        }
        Ok(Self { handle, context })
    }
}

#[cfg(windows)]
unsafe extern "system" fn callback(
    _: windows::Win32::Devices::DeviceAndDriverInstallation::HCMNOTIFICATION,
    context: *const core::ffi::c_void,
    _: windows::Win32::Devices::DeviceAndDriverInstallation::CM_NOTIFY_ACTION,
    _: *const windows::Win32::Devices::DeviceAndDriverInstallation::CM_NOTIFY_EVENT_DATA,
    _: u32,
) -> u32 {
    let sender = unsafe { &*(context as *const tokio::sync::mpsc::UnboundedSender<()>) };
    let _ = sender.send(());
    0
}

#[cfg(windows)]
impl Drop for NativeWatcher {
    fn drop(&mut self) {
        unsafe {
            let _ =
                windows::Win32::Devices::DeviceAndDriverInstallation::CM_Unregister_Notification(
                    self.handle,
                );
            drop(Box::from_raw(self.context));
        }
    }
}

#[cfg(not(windows))]
pub struct NativeWatcher;

#[cfg(not(windows))]
impl NativeWatcher {
    pub fn register(_: tokio::sync::mpsc::UnboundedSender<()>) -> Result<Self, String> {
        Err("Windows only".into())
    }
}
