use std::{thread, time};
use std::ptr::addr_of_mut;
use sysinfo::Pid;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

macro_rules! as_ptr {
    ($value:expr) => {
        $value as *mut core::ffi::c_void
    };
}

macro_rules! as_lparam {
    ($value:expr, $type:ty) => {
        LPARAM(&mut $value as *mut $type as isize)
    };
}

pub fn get_process_path(process_id: u32) -> Option<String> {
    let mut system = sysinfo::System::new();
    system.refresh_all();
    let pid = Pid::from_u32(process_id);
    let process = system.process(pid)?;
    let process_path = process.exe()?.to_str()?;
    Some(process_path.to_string())
}

pub fn get_process_id_by_hwnd(hwnd: isize) -> Option<u32> {
    let mut pid_nr = 0;
    let hwnd = HWND(as_ptr!(hwnd));
    unsafe {
        GetWindowThreadProcessId(hwnd, Option::from(&raw mut pid_nr));
    }

    let mut system = sysinfo::System::new();
    system.refresh_all();
    let mut pid = Pid::from_u32(pid_nr);
    let process = system.process(pid)?;
    if process.name() == "ApplicationFrameHost.exe" {

        let lookup = RealProcessLookup {
            afh_pid: pid_nr,
            hwnd,
            real_pid: None
        };

        let mut real_pid = get_real_process(&lookup);

        if !real_pid.is_some() {
            let sleep_time = time::Duration::from_millis(500);
            thread::sleep(sleep_time);
            real_pid = get_real_process(&lookup)
        };

        pid = Pid::from_u32(real_pid?)
    }

    Some(pid.as_u32())
}

#[derive(Clone)]
struct RealProcessLookup {
    afh_pid: u32,
    hwnd: HWND,
    real_pid: Option<u32>
}

fn get_real_process(lookup: &RealProcessLookup) -> Option<u32> {
    let mut lookup = lookup.clone();
    unsafe {
        let _ = EnumChildWindows(lookup.hwnd, Some(enum_child_windows_callback), as_lparam!(lookup, RealProcessLookup));
    }
    lookup.real_pid
}

extern "system" fn  enum_child_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let real_lookup = &mut *(lparam.0 as *mut RealProcessLookup);
        let mut temp_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Option::from(addr_of_mut!(temp_pid)));

        if real_lookup.afh_pid != temp_pid {
            real_lookup.real_pid = Some(temp_pid);

            return  BOOL::from(false)
        }

        BOOL::from(true)
    }
}


