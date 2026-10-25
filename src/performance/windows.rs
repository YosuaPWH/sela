use super::{Application, HardwareReading};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    os::windows::process::CommandExt,
    path::Path,
    time::{Duration, Instant},
};
use windows::{
    core::{w, HSTRING, PCSTR, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, FreeLibrary, BOOL, HMODULE, HWND, LPARAM},
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
            Performance::*,
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                EnumWindows, GetForegroundWindow, GetWindowThreadProcessId, IsWindowVisible,
            },
        },
    },
};

fn application(hwnd: HWND) -> Option<Application> {
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 || pid == std::process::id() {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = vec![0u16; 32768];
        let mut size = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        result.ok()?;
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        let name = path.rsplit('\\').next()?.to_owned();
        if [
            "explorer.exe",
            "dwm.exe",
            "shellexperiencehost.exe",
            "startmenuexperiencehost.exe",
            "searchhost.exe",
            "applicationframehost.exe",
        ]
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
        {
            return None;
        }
        Some(Application { pid, name })
    }
}
pub fn foreground() -> Option<Application> {
    unsafe { application(GetForegroundWindow()) }
}
pub fn applications() -> Vec<Application> {
    unsafe extern "system" fn collect(hwnd: HWND, data: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd).as_bool() {
            if let Some(app) = application(hwnd) {
                (*(data.0 as *mut Vec<Application>)).push(app);
            }
        }
        BOOL(1)
    }
    let mut apps: Vec<Application> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect),
            LPARAM(&mut apps as *mut Vec<Application> as isize),
        );
    }
    apps.sort_by(|a, b| a.name.cmp(&b.name).then(a.pid.cmp(&b.pid)));
    apps.dedup_by_key(|p| p.pid);
    apps
}
pub fn prepare_command(command: &mut std::process::Command) {
    command.creation_flags(0x08000000);
}
pub fn open_folder(path: &Path) -> Result<(), String> {
    unsafe {
        let result = ShellExecuteW(
            None,
            w!("open"),
            &HSTRING::from(path.as_os_str()),
            None,
            None,
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        if result.0 as usize <= 32 {
            Err("Cannot open the export folder.".into())
        } else {
            Ok(())
        }
    }
}

struct Counters {
    query: isize,
    engine: isize,
    memory: isize,
    started: Instant,
}
impl Default for Counters {
    fn default() -> Self {
        unsafe {
            let mut s = Self {
                query: 0,
                engine: 0,
                memory: 0,
                started: Instant::now(),
            };
            if PdhOpenQueryW(PCWSTR::null(), 0, &mut s.query) != 0 {
                s.query = 0;
                return s;
            }
            if PdhAddEnglishCounterW(
                s.query,
                w!("\\GPU Engine(*)\\Utilization Percentage"),
                0,
                &mut s.engine,
            ) != 0
            {
                s.engine = 0;
            }
            if PdhAddEnglishCounterW(
                s.query,
                w!("\\GPU Adapter Memory(*)\\Dedicated Usage"),
                0,
                &mut s.memory,
            ) != 0
            {
                s.memory = 0;
            }
            PdhCollectQueryData(s.query);
            s
        }
    }
}
impl Drop for Counters {
    fn drop(&mut self) {
        if self.query != 0 {
            unsafe {
                PdhCloseQuery(self.query);
            }
        }
    }
}
fn values(counter: isize) -> Vec<(String, f64)> {
    unsafe {
        if counter == 0 {
            return vec![];
        }
        let mut bytes = 0;
        let mut count = 0;
        if PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut bytes, &mut count, None)
            != PDH_MORE_DATA
            || bytes == 0
            || bytes > 16 * 1024 * 1024
        {
            return vec![];
        }
        let mut buffer = vec![0u64; (bytes as usize).div_ceil(8)];
        if PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut bytes,
            &mut count,
            Some(buffer.as_mut_ptr().cast()),
        ) != 0
            || count as usize * std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>()
                > buffer.len() * 8
        {
            return vec![];
        }
        std::slice::from_raw_parts(
            buffer.as_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>(),
            count as usize,
        )
        .iter()
        .filter_map(|item| {
            let value = item.FmtValue.Anonymous.doubleValue;
            if item.FmtValue.CStatus > 1 || !value.is_finite() || value < 0.0 {
                return None;
            }
            Some((item.szName.to_string().ok()?, value))
        })
        .collect()
    }
}
impl Counters {
    fn sample(&mut self) -> HardwareReading {
        unsafe {
            if self.query == 0 || PdhCollectQueryData(self.query) != 0 {
                return HardwareReading {
                    source: "GPU counters unavailable on this device.".into(),
                    ..Default::default()
                };
            }
            let mut engines = BTreeMap::<String, f64>::new();
            if self.started.elapsed() > Duration::from_millis(250) {
                for (name, value) in values(self.engine) {
                    if !name.ends_with("engtype_3D") {
                        continue;
                    }
                    if let Some(i) = name.find("luid_") {
                        *engines.entry(name[i..].to_owned()).or_default() += value;
                    }
                }
            }
            let gpu = engines
                .values()
                .copied()
                .reduce(f64::max)
                .map(|v| v.clamp(0.0, 100.0));
            let memories = values(self.memory);
            let vram_gb = (!memories.is_empty())
                .then(|| memories.iter().map(|(_, v)| v).sum::<f64>() / 1_073_741_824.0);
            HardwareReading{gpu,vram_gb,source:"Windows GPU counters · busiest 3D engine; VRAM across adapters. Temperature/power require a supported sensor.".into(),..Default::default()}
        }
    }
}
#[repr(C)]
#[derive(Default)]
struct Utilization {
    gpu: u32,
    memory: u32,
}
#[repr(C)]
#[derive(Default)]
struct Memory {
    total: u64,
    free: u64,
    used: u64,
}
type Device = *mut c_void;
type Init = unsafe extern "C" fn() -> u32;
type DeviceAt = unsafe extern "C" fn(u32, *mut Device) -> u32;
type Util = unsafe extern "C" fn(Device, *mut Utilization) -> u32;
type Mem = unsafe extern "C" fn(Device, *mut Memory) -> u32;
type Temp = unsafe extern "C" fn(Device, u32, *mut u32) -> u32;
type Power = unsafe extern "C" fn(Device, *mut u32) -> u32;
struct Nvml {
    library: HMODULE,
    device: Device,
    shutdown: Init,
    util: Option<Util>,
    memory: Option<Mem>,
    temp: Option<Temp>,
    power: Option<Power>,
}
unsafe fn symbol<F: Copy>(library: HMODULE, name: &[u8]) -> Option<F> {
    let f = GetProcAddress(library, PCSTR(name.as_ptr()))?;
    Some(std::mem::transmute_copy(&f))
}
impl Nvml {
    fn load() -> Option<Self> {
        unsafe {
            // Never search the working directory for a vendor DLL.
            let library =
                LoadLibraryExW(w!("nvml.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?;
            let initialize = symbol::<Init>(library, b"nvmlInit_v2\0");
            let shutdown = symbol::<Init>(library, b"nvmlShutdown\0");
            let device_at = symbol::<DeviceAt>(library, b"nvmlDeviceGetHandleByIndex_v2\0");
            let (Some(initialize), Some(shutdown), Some(device_at)) =
                (initialize, shutdown, device_at)
            else {
                let _ = FreeLibrary(library);
                return None;
            };
            if initialize() != 0 {
                let _ = FreeLibrary(library);
                return None;
            }
            let mut device = std::ptr::null_mut();
            if device_at(0, &mut device) != 0 || device.is_null() {
                shutdown();
                let _ = FreeLibrary(library);
                return None;
            }
            Some(Self {
                library,
                device,
                shutdown,
                util: symbol(library, b"nvmlDeviceGetUtilizationRates\0"),
                memory: symbol(library, b"nvmlDeviceGetMemoryInfo\0"),
                temp: symbol(library, b"nvmlDeviceGetTemperature\0"),
                power: symbol(library, b"nvmlDeviceGetPowerUsage\0"),
            })
        }
    }
    fn sample(&self) -> HardwareReading {
        unsafe {
            let mut u = Utilization::default();
            let mut m = Memory::default();
            let mut t = 0;
            let mut p = 0;
            HardwareReading {
                gpu: self
                    .util
                    .filter(|f| f(self.device, &mut u) == 0 && u.gpu <= 100)
                    .map(|_| u.gpu as f64),
                vram_gb: self
                    .memory
                    .filter(|f| f(self.device, &mut m) == 0)
                    .map(|_| m.used as f64 / 1_073_741_824.0),
                gpu_temp: self
                    .temp
                    .filter(|f| f(self.device, 0, &mut t) == 0 && t <= 150)
                    .map(|_| t as f64),
                power: self
                    .power
                    .filter(|f| f(self.device, &mut p) == 0)
                    .map(|_| p as f64 / 1000.0),
                source: "NVIDIA NVML · first NVIDIA adapter; CPU temperature unavailable.".into(),
                ..Default::default()
            }
        }
    }
}
impl Drop for Nvml {
    fn drop(&mut self) {
        unsafe {
            (self.shutdown)();
            let _ = FreeLibrary(self.library);
        }
    }
}
pub struct Hardware {
    counters: Counters,
    nvml: Option<Nvml>,
}
impl Default for Hardware {
    fn default() -> Self {
        Self {
            counters: Default::default(),
            nvml: Nvml::load(),
        }
    }
}
impl Hardware {
    pub fn sample(&mut self) -> HardwareReading {
        if let Some(nvml) = &self.nvml {
            nvml.sample()
        } else {
            self.counters.sample()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_sensors_are_optional() {
        let r = Hardware::default().sample();
        assert!(r
            .gpu
            .is_none_or(|v| v.is_finite() && (0.0..=100.0).contains(&v)));
        assert!(r.vram_gb.is_none_or(|v| v.is_finite() && v >= 0.0));
    }
}

pub struct ChildGuard(windows::Win32::Foundation::HANDLE);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
pub fn guard_child(child: &std::process::Child) -> Result<ChildGuard, String> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::System::JobObjects::*;
    unsafe {
        let guard = ChildGuard(CreateJobObjectW(None, None).map_err(|e| e.to_string())?);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            guard.0,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of_val(&limits) as u32,
        )
        .map_err(|e| e.to_string())?;
        AssignProcessToJobObject(
            guard.0,
            windows::Win32::Foundation::HANDLE(child.as_raw_handle()),
        )
        .map_err(|e| format!("Cannot manage the capture process lifetime: {e}"))?;
        Ok(guard)
    }
}

/// Exercises actual DXGI presents through the embedded PresentMon CLI on Windows.
/// WARP makes this independent of a physical GPU, but ETW permissions are required.
#[cfg(test)]
mod capture_test {
    use super::*;
    use windows::Win32::{
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_WARP,
            Direct3D11::{
                D3D11CreateDeviceAndSwapChain, ID3D11Texture2D, D3D11_CREATE_DEVICE_FLAG,
                D3D11_SDK_VERSION,
            },
            Dxgi::{
                Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_SAMPLE_DESC},
                DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    };
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
    #[test]
    #[ignore = "Windows ETW integration probe; run explicitly on the Windows CI runner"]
    fn embedded_presentmon_captures_real_dxgi_presents() {
        unsafe {
            let instance = GetModuleHandleW(None).unwrap();
            let class =
                windows::core::HSTRING::from(format!("SelaCaptureProbe{}", std::process::id()));
            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class.as_ptr()),
                ..Default::default()
            };
            assert_ne!(RegisterClassW(&wc), 0);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                &class,
                w!("Sela capture probe"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                0,
                0,
                128,
                128,
                None,
                None,
                instance,
                None,
            )
            .unwrap();
            struct Window(HWND);
            impl Drop for Window {
                fn drop(&mut self) {
                    unsafe {
                        let _ = DestroyWindow(self.0);
                    }
                }
            }
            let _window = Window(hwnd);
            let desc = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC {
                    Width: 64,
                    Height: 64,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ..Default::default()
                },
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                OutputWindow: hwnd,
                Windowed: BOOL(1),
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                ..Default::default()
            };
            let mut swap = None;
            let mut device = None;
            let mut context = None;
            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_WARP,
                None,
                D3D11_CREATE_DEVICE_FLAG::default(),
                None,
                D3D11_SDK_VERSION,
                Some(&desc),
                Some(&mut swap),
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .unwrap();
            let swap = swap.unwrap();
            let device = device.unwrap();
            let context = context.unwrap();
            let buffer: ID3D11Texture2D = swap.GetBuffer(0).unwrap();
            let mut view = None;
            device
                .CreateRenderTargetView(&buffer, None, Some(&mut view))
                .unwrap();
            let view = view.unwrap();
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 128, 128, SWP_SHOWWINDOW);
            let mut capture = super::super::capture(&Application {
                pid: std::process::id(),
                name: "SelaCaptureProbe.exe".into(),
            })
            .unwrap();
            let start = Instant::now();
            let mut frames = vec![];
            while start.elapsed() < Duration::from_secs(20) && frames.len() < 10 {
                let mut message = MSG::default();
                while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                context.ClearRenderTargetView(&view, &[0.1, 0.3, 0.2, 1.0]);
                context.Flush();
                swap.Present(0, DXGI_PRESENT(0)).ok().unwrap();
                std::thread::sleep(Duration::from_millis(16));
                frames.extend(capture.frames.try_iter());
                if let Some(code) = capture.child.try_wait().unwrap() {
                    panic!(
                        "PresentMon exited before capturing frames: {code}; {}",
                        super::super::capture_diagnostics(&capture)
                    );
                }
            }
            assert!(
                frames.len() >= 10,
                "No DXGI frames captured. Provider diagnostics: {}",
                super::super::capture_diagnostics(&capture)
            );
            assert!(frames
                .iter()
                .all(|f| f.pid == std::process::id() && f.ms > 0.0));
            let stats = super::super::FrameStats::calculate(
                &frames.iter().map(|f| f.ms).collect::<Vec<_>>(),
            )
            .unwrap();
            println!(
                "Real PresentMon capture: {} frames, {:.1} FPS",
                stats.frames, stats.fps
            );
            drop(capture);
            drop(swap);
            drop(device);
        }
    }
}
