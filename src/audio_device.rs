// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Triffit

//! Windows Core Audio device enumeration and control.

use crate::config::AppResult;
use crate::log_dbg;
use std::sync::{Arc, Mutex};
use windows::core::{GUID, HSTRING, Interface, IUnknown, IUnknown_Vtbl, PCWSTR, HRESULT, Result};
use windows::Win32::Foundation::BOOL;
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, MMDeviceEnumerator, eRender, DEVICE_STATE_ACTIVE,
    IMMDeviceCollection, IMMDevice, eConsole, eMultimedia, eCommunications, ERole,
    IMMNotificationClient_Vtbl, IMMNotificationClient, DEVICE_STATE, EDataFlow, WAVEFORMATEX
};
use windows::Win32::System::Com::{
    CoInitializeEx, COINIT_APARTMENTTHREADED, CoCreateInstance, CLSCTX_ALL,
    CoTaskMemFree, STGM_READ
};
use windows::Win32::System::Com::StructuredStorage::{PROPVARIANT, PropVariantClear};
use windows::Win32::System::Variant::VT_LPWSTR;

// Manually define PKEY_Device_FriendlyName since it's not in the crate features
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub struct PROPERTYKEY {
    fmtid: GUID,
    pid: u32,
}

#[allow(non_upper_case_globals)]
const PKEY_Device_FriendlyName: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

/// Represents an audio output device.
#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Enumerate all active audio output devices.
pub fn enumerate_audio_devices() -> AppResult<Vec<AudioDevice>> {
    unsafe {
        // Initialize COM
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        )?;

        // Get default device ID for marking
        let default_id = get_default_device_id(&enumerator).ok();

        let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
        let count = collection.GetCount()?;

        let mut devices = Vec::new();
        for i in 0..count {
            match get_device_info(&collection, i, default_id.as_ref()) {
                Ok(device) => devices.push(device),
                Err(_e) => { 
                    log_dbg!("audio: failed to get device {i}: {:?}", _e);
                }
            }
        }

        log_dbg!("audio: enumerated {} output devices", devices.len());
        Ok(devices)
    }
}

/// Get device info for a specific index.
unsafe fn get_device_info(
    collection: &IMMDeviceCollection,
    index: u32,
    default_id: Option<&String>,
) -> AppResult<AudioDevice> {
    let device = collection.Item(index)?;
    
    let id = get_device_id(&device)?;
    let name = get_device_name(&device, index)?;
    let is_default = default_id.is_some_and(|d| d == &id);

    Ok(AudioDevice { id, name, is_default })
}

/// Get device ID string.
unsafe fn get_device_id(device: &IMMDevice) -> AppResult<String> {
    let id_pwstr = device.GetId()
        .map_err(|e| format!("Failed to get device ID: {:?}", e))?;
    let id = id_pwstr.to_string()
        .map_err(|e| format!("Failed to convert device ID to string: {:?}", e))?;
    CoTaskMemFree(Some(id_pwstr.as_ptr() as _));
    Ok(id)
}

/// Get friendly device name.
unsafe fn get_device_name(device: &IMMDevice, index: u32) -> AppResult<String> {
    let props = device.OpenPropertyStore(STGM_READ)
        .map_err(|e| format!("Failed to open property store for device {}: {:?}", index, e))?;
    
    // GetValue takes one argument - a REFPROPERTYKEY (a reference to PROPERTYKEY)
    let name_var = props.GetValue(std::ptr::from_ref(&PKEY_Device_FriendlyName).cast())?;
    
    // Access the pwszVal field through the union structure
    let name = if name_var.Anonymous.Anonymous.vt == VT_LPWSTR {
        let pwstr = name_var.Anonymous.Anonymous.Anonymous.pwszVal;
        let result = pwstr.to_string().unwrap_or_else(|_| format!("Device {index}"));
        let _ = PropVariantClear((&raw const name_var).cast_mut());
        result
    } else {
        let _ = PropVariantClear((&raw const name_var).cast_mut());
        format!("Device {index}")
    };
    
    Ok(name)
}

/// Get the current default audio device ID.
fn get_default_device_id(enumerator: &IMMDeviceEnumerator) -> AppResult<String> {
    unsafe {
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        get_device_id(&device)
    }
}

// CLSID for PolicyConfig (used in set_default_audio_device)
const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

/// Set the specified device as the default audio output.
pub fn set_default_audio_device(device_id: &str) -> AppResult<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        // Use IPolicyConfig (undocumented but widely used) to set default device
        let policy_config: IPolicyConfig = CoCreateInstance(
            &CLSID_POLICY_CONFIG,
            None,
            CLSCTX_ALL,
        ).map_err(|e| format!("Failed to create PolicyConfig COM instance: {:?}", e))?;

        let device_id_hstring = HSTRING::from(device_id);
        policy_config.SetDefaultEndpoint(&device_id_hstring, eConsole)
            .map_err(|e| format!("Failed to set console endpoint: {:?}", e))?;
        policy_config.SetDefaultEndpoint(&device_id_hstring, eMultimedia)
            .map_err(|e| format!("Failed to set multimedia endpoint: {:?}", e))?;
        policy_config.SetDefaultEndpoint(&device_id_hstring, eCommunications)
            .map_err(|e| format!("Failed to set communications endpoint: {:?}", e))?;

        log_dbg!("audio: set default device to {device_id}");

        log_dbg!("audio: set default device to {device_id}");
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Undocumented IPolicyConfig COM Interface
// Microsoft doesn't officially expose this, but it's the standard way to
// programmatically set default audio devices on Windows.
// ═══════════════════════════════════════════════════════════════════════════════

#[allow(non_camel_case_types, non_snake_case)]
#[repr(transparent)]
pub struct IPolicyConfig(std::ptr::NonNull<std::ffi::c_void>);

unsafe impl Interface for IPolicyConfig {
    type Vtable = IPolicyConfig_Vtbl;
    const IID: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);
}

impl std::ops::Deref for IPolicyConfig {
    type Target = IUnknown;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(std::ptr::from_ref(self).cast::<IUnknown>()) }
    }
}

impl Clone for IPolicyConfig {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct IPolicyConfig_Vtbl {
    pub base__: IUnknown_Vtbl,
    pub GetMixFormat: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        format: *mut *mut WAVEFORMATEX,
    ) -> HRESULT,
    pub GetDeviceFormat: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        default: BOOL,
        format: *mut *mut WAVEFORMATEX,
    ) -> HRESULT,
    pub ResetDeviceFormat: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
    ) -> HRESULT,
    pub SetDeviceFormat: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        endpoint_format: *const WAVEFORMATEX,
        mix_format: *const WAVEFORMATEX,
    ) -> HRESULT,
    pub GetProcessingPeriod: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        default: BOOL,
        period: *mut i64,
        device_period: *mut i64,
    ) -> HRESULT,
    pub SetProcessingPeriod: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        period: *const i64,
    ) -> HRESULT,
    pub GetShareMode: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        share_mode: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    pub SetShareMode: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        share_mode: *const std::ffi::c_void,
    ) -> HRESULT,
    pub GetPropertyValue: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        key: *const PROPERTYKEY,
        value: *mut PROPVARIANT,
    ) -> HRESULT,
    pub SetPropertyValue: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        key: *const PROPERTYKEY,
        value: *const PROPVARIANT,
    ) -> HRESULT,
    pub SetDefaultEndpoint: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        role: ERole,
    ) -> HRESULT,
    pub SetEndpointVisibility: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        device_id: PCWSTR,
        visible: BOOL,
    ) -> HRESULT,
}

impl IPolicyConfig {
    #[allow(non_snake_case)]
    pub unsafe fn SetDefaultEndpoint(&self, device_id: &HSTRING, role: ERole) -> Result<()> {
        (Interface::vtable(self).SetDefaultEndpoint)(
            Interface::as_raw(self),
            PCWSTR(device_id.as_ptr()),
            role,
        )
        .ok()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMMNotificationClient - Event-driven audio device change notifications
// This eliminates the need for polling and only triggers when devices actually
// connect/disconnect or the default device changes.
// ═══════════════════════════════════════════════════════════════════════════════

/// Callback type for when audio devices change
pub type DeviceChangeCallback = Arc<dyn Fn() + Send + Sync>;

/// COM notification client for audio device state changes.
/// Uses manual vtable implementation since windows-rs implement macro has limitations.
#[repr(C)]
pub struct DeviceNotificationClient {
    vtable: *const IMMNotificationClient_Vtbl,
    ref_count: Mutex<u32>,
    callback: DeviceChangeCallback,
}

impl DeviceNotificationClient {
    pub fn create(callback: DeviceChangeCallback) -> IMMNotificationClient {
        let client = Box::new(DeviceNotificationClient {
            vtable: &raw const NOTIFICATION_VTABLE,
            ref_count: Mutex::new(1),
            callback,
        });
        
        unsafe {
            IMMNotificationClient::from_raw(Box::into_raw(client).cast())
        }
    }
}

// Manual vtable for IMMNotificationClient
#[allow(non_upper_case_globals)]
static NOTIFICATION_VTABLE: IMMNotificationClient_Vtbl = IMMNotificationClient_Vtbl {
    base__: IUnknown_Vtbl {
        QueryInterface: query_interface,
        AddRef: add_ref,
        Release: release,
    },
    OnDeviceStateChanged: on_device_state_changed,
    OnDeviceAdded: on_device_added,
    OnDeviceRemoved: on_device_removed,
    OnDefaultDeviceChanged: on_default_device_changed,
    OnPropertyValueChanged: on_property_value_changed,
};

unsafe extern "system" fn query_interface(
    this: *mut std::ffi::c_void,
    iid: *const GUID,
    object: *mut *mut std::ffi::c_void,
) -> HRESULT {
    let iid = &*iid;
    if iid == &IMMNotificationClient::IID || iid == &IUnknown::IID {
        *object = this;
        add_ref(this);
        HRESULT(0)
    } else {
        *object = std::ptr::null_mut();
        const E_NOINTERFACE: i32 = 0x8000_4002_u32 as i32;
        HRESULT(E_NOINTERFACE)
    }
}

unsafe extern "system" fn add_ref(this: *mut std::ffi::c_void) -> u32 {
    let client = &*(this as *const DeviceNotificationClient);
    let mut count = client.ref_count.lock().unwrap();
    *count += 1;
    *count
}

unsafe extern "system" fn release(this: *mut std::ffi::c_void) -> u32 {
    let client = &*(this.cast::<DeviceNotificationClient>());
    let count = {
        let mut count = client.ref_count.lock().unwrap();
        *count -= 1;
        *count
    };
    
    if count == 0 {
        let _ = Box::from_raw(this.cast::<DeviceNotificationClient>());
    }
    
    count
}

unsafe extern "system" fn on_device_state_changed(
    this: *mut std::ffi::c_void,
    _device_id: PCWSTR,
    _new_state: DEVICE_STATE,
) -> HRESULT {
    let client = &*(this as *const DeviceNotificationClient);
    log_dbg!("audio: device state changed");
    (client.callback)();
    HRESULT(0)
}

unsafe extern "system" fn on_device_added(
    this: *mut std::ffi::c_void,
    _device_id: PCWSTR,
) -> HRESULT {
    let client = &*(this as *const DeviceNotificationClient);
    log_dbg!("audio: device added");
    (client.callback)();
    HRESULT(0)
}

unsafe extern "system" fn on_device_removed(
    this: *mut std::ffi::c_void,
    _device_id: PCWSTR,
) -> HRESULT {
    let client = &*(this as *const DeviceNotificationClient);
    log_dbg!("audio: device removed");
    (client.callback)();
    HRESULT(0)
}

unsafe extern "system" fn on_default_device_changed(
    this: *mut std::ffi::c_void,
    _flow: EDataFlow,
    _role: ERole,
    _default_device_id: PCWSTR,
) -> HRESULT {
    let client = &*(this as *const DeviceNotificationClient);
    log_dbg!("audio: default device changed");
    (client.callback)();
    HRESULT(0)
}

unsafe extern "system" fn on_property_value_changed(
    _this: *mut std::ffi::c_void,
    _device_id: PCWSTR,
    _key: windows::Win32::Foundation::PROPERTYKEY,
) -> HRESULT {
    // Don't trigger on every property change (too noisy)
    HRESULT(0)
}

/// Register for audio device change notifications.
/// Returns a handle that must be kept alive to continue receiving notifications.
pub fn register_device_change_callback(callback: DeviceChangeCallback) -> AppResult<IMMNotificationClient> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ).map_err(|e| format!("Failed to create device enumerator: {:?}", e))?;

        let client = DeviceNotificationClient::create(callback);
        enumerator.RegisterEndpointNotificationCallback(&client)
            .map_err(|e| format!("Failed to register notification callback: {:?}", e))?;
        
        log_dbg!("audio: registered device change notification callback");
        
        log_dbg!("audio: registered device change notification callback");
        
        Ok(client)
    }
}
