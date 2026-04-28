//! RTSS OSD integration via `RTSSSharedMemoryV2` file mapping.
//!
//! This follows the same slot selection / update pattern as common RTSS sample code:
//! - validate signature `RTSS` and version >= 2.0
//! - locate OSD entries via `dwOSDArrOffset`, `dwOSDEntrySize`, `dwOSDArrSize`
//! - write `szOSDEx` for RTSS >= 2.7 else `szOSD`
//! - increment `dwOSDFrame` to force refresh

#[cfg(windows)]
mod imp {
    use std::ffi::CString;

    use winapi::ctypes::c_void;
    use winapi::shared::minwindef::{DWORD, FALSE};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::memoryapi::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS};
    use winapi::um::synchapi::Sleep;
    use winapi::um::winnt::{HANDLE, LPCWSTR};

    // RTSS stores the v2 header signature as a DWORD built from the four ASCII bytes 'R','T','S','S'.
    // Depending on endianness interpretation, this commonly appears as 0x52545353 (what we read via *(DWORD*)).
    const RTSS_SIGNATURE_A: DWORD = 0x5254_5353; // 'RTSS' as commonly observed in RTSS headers / samples
    const RTSS_SIGNATURE_B: DWORD = u32::from_le_bytes(*b"RTSS"); // alternate encoding (kept for safety)

    fn rtss_version(major: u32, minor: u32) -> u32 {
        (major << 16) | (minor & 0xFFFF)
    }

    const OSD_TEXT_CORE: usize = 256 + 256 + 4096; // szOSD + szOSDOwner + szOSDEx
    const OSD_BUFFER_V212: usize = 262_144; // BYTE buffer[262144] in RTSS header definitions (v2.12+)

    fn osd_stride_bytes(version: u32, dw_osd_entry_size: u32) -> usize {
        let reported = dw_osd_entry_size as usize;
        let mut min = OSD_TEXT_CORE;
        if version >= rtss_version(2, 12) {
            min += OSD_BUFFER_V212;
        }
        // RTSS sometimes exposes a small dwOSDEntrySize (e.g. 256) even though the in-memory layout is larger.
        // Never use a stride smaller than the known minimum for this RTSS version.
        reported.max(min)
    }

    pub struct RtssMap {
        h_map: HANDLE,
        base: *mut u8,
    }

    impl RtssMap {
        pub fn open() -> Result<Self, String> {
            // RTSS mapping name is usually "RTSSSharedMemoryV2". Some setups require an explicit
            // Global\\ / Local\\ prefix depending on session / isolation.
            let candidates = [
                "RTSSSharedMemoryV2",
                "Global\\RTSSSharedMemoryV2",
                "Local\\RTSSSharedMemoryV2",
            ];

            let mut last_open_err: Option<u32> = None;
            let mut last_bad_sig: Option<(String, u32, u32)> = None;
            for c in candidates {
                let name: Vec<u16> = format!("{c}\0").encode_utf16().collect();
                let h_map = unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, name.as_ptr() as LPCWSTR) };
                if h_map.is_null() {
                    last_open_err = Some(unsafe { GetLastError() });
                    continue;
                }

                let base = unsafe { MapViewOfFile(h_map, FILE_MAP_ALL_ACCESS, 0, 0, 0) } as *mut u8;
                if base.is_null() {
                    let e = unsafe { GetLastError() };
                    unsafe { CloseHandle(h_map) };
                    return Err(format!("MapViewOfFile failed for {c}: {e}"));
                }

                let sig = read_u32(base, 0);
                let ver = read_u32(base, 4);
                if sig != RTSS_SIGNATURE_A && sig != RTSS_SIGNATURE_B {
                    last_bad_sig = Some((c.to_string(), sig, ver));
                    unsafe {
                        let _ = UnmapViewOfFile(base as *mut c_void);
                        let _ = CloseHandle(h_map);
                    }
                    // Try next candidate
                    continue;
                }
                if ver < rtss_version(2, 0) {
                    unsafe {
                        let _ = UnmapViewOfFile(base as *mut c_void);
                        let _ = CloseHandle(h_map);
                    }
                    return Err(format!(
                        "Unsupported RTSS shared memory version on {c}: 0x{ver:08X}"
                    ));
                }

                return Ok(Self { h_map, base });
            }

            if let Some((name, sig, ver)) = last_bad_sig {
                return Err(format!(
                    "Opened a mapping object named like RTSS, but header didn't validate as RTSS.\n\
                     mapping='{name}' dwSignature=0x{sig:08X} dwVersion=0x{ver:08X}\n\
                     This usually means the mapping name points at the wrong object, RTSS shared memory isn't published on this machine/session, or RTSS is blocked/restricted.\n\
                     Last OpenFileMappingW error (if any): {:?}",
                    last_open_err
                ));
            }

            if let Some(e) = last_open_err {
                Err(format!(
                    "Could not open RTSS shared memory mapping (tried RTSSSharedMemoryV2 / Global\\\\ / Local\\\\). Last OpenFileMappingW error: {e}"
                ))
            } else {
                Err(
                    "Could not open RTSS shared memory mapping (tried RTSSSharedMemoryV2 / Global\\\\ / Local\\\\)."
                        .into(),
                )
            }
        }

        fn osd_entry_ptr(&self, index: u32) -> Result<*mut u8, String> {
            // Field offsets in RTSS_SHARED_MEMORY v2 header (see RTSSSharedMemory.h in RTSS SDK / community repos).
            let version = read_u32(self.base, 4);
            let reported = read_u32(self.base, 20); // dwOSDEntrySize
            let off = read_u32(self.base, 24) as usize; // dwOSDArrOffset
            let stride = osd_stride_bytes(version, reported);
            let count = read_u32(self.base, 28); // dwOSDArrSize
            if index >= count {
                return Err(format!("OSD slot {index} out of range (size={count})"));
            }
            Ok(unsafe { self.base.add(off + stride * index as usize) })
        }

        fn write_osd_strings(entry: *mut u8, version: u32, text: &CString) -> Result<(), String> {
            let text_bytes = text.as_bytes_with_nul();
            if text_bytes.len() > 4096 {
                return Err("text must be <=4095 chars plus NUL in ANSI representation".into());
            }

            unsafe {
                let p_osd = entry;
                let p_ex = entry.add(256 + 256);

                if version >= rtss_version(2, 7) {
                    std::ptr::write_bytes(p_osd, 0, 256);
                    std::ptr::write_bytes(p_ex, 0, 4096);
                    let n = (text_bytes.len().min(4096)).saturating_sub(1);
                    std::ptr::copy_nonoverlapping(text_bytes.as_ptr(), p_ex, n);
                    *p_ex.add(n) = 0;
                } else {
                    std::ptr::write_bytes(p_osd, 0, 256);
                    std::ptr::write_bytes(p_ex, 0, 4096);
                    let n = (text_bytes.len().min(256)).saturating_sub(1);
                    std::ptr::copy_nonoverlapping(text_bytes.as_ptr(), p_osd, n);
                    *p_osd.add(n) = 0;
                }
            }
            Ok(())
        }

        fn write_owner(entry: *mut u8, owner: &CString) -> Result<(), String> {
            let ob = owner.as_bytes_with_nul();
            if ob.len() > 256 {
                return Err("owner must be <=255 chars plus NUL in ANSI representation".into());
            }
            unsafe {
                let p_owner = entry.add(256);
                std::ptr::write_bytes(p_owner, 0, 256);
                let n = ob.len().saturating_sub(1);
                std::ptr::copy_nonoverlapping(ob.as_ptr(), p_owner, n);
                *p_owner.add(n) = 0;
            }
            Ok(())
        }

        fn owner_is_empty(entry: *mut u8) -> bool {
            unsafe {
                let p_owner = entry.add(256);
                *p_owner == 0
            }
        }

        fn owner_equals(entry: *mut u8, owner: &CString) -> bool {
            let want = owner.as_bytes_with_nul();
            unsafe {
                let p_owner = entry.add(256);
                let slice = std::slice::from_raw_parts(p_owner, 256);
                let len = slice.iter().position(|&b| b == 0).unwrap_or(256);
                &slice[..len] == &want[..want.len().saturating_sub(1)]
            }
        }

        fn clear_entry(entry: *mut u8, stride: usize) {
            unsafe {
                std::ptr::write_bytes(entry, 0, stride);
            }
        }

        fn prepare_entry_for_write(entry: *mut u8, stride: usize) {
            // Always wipe the full slot. RTSS v2.12+ includes a large per-slot buffer that may contain
            // leftover embedded-object bytes from other tools; those can surface as odd "^..." fragments.
            Self::clear_entry(entry, stride);
        }

        fn bump_osd_frame(&self) {
            unsafe {
                // RTSS v2 header: dwOSDArrSize @ +28, dwOSDFrame @ +32.
                // Incrementing +28 corrupts slot count and can cause undefined OSD behavior.
                let p = self.base.add(32) as *mut u32;
                *p = (*p).wrapping_add(1);
            }
        }

        pub fn update(&self, owner: &CString, text: &CString, preferred_slot: u32) -> Result<(), String> {
            let version = read_u32(self.base, 4);

            let osd_size = read_u32(self.base, 28);

            let slot = if preferred_slot != 0 {
                if preferred_slot >= osd_size {
                    return Err(format!("preferred slot {preferred_slot} out of range (size={osd_size})"));
                }
                let e = self.osd_entry_ptr(preferred_slot)?;
                let stride = {
                    let reported = read_u32(self.base, 20);
                    osd_stride_bytes(version, reported)
                };
                if Self::owner_is_empty(e) {
                    Self::prepare_entry_for_write(e, stride);
                    Self::write_owner(e, owner)?;
                    preferred_slot
                } else if Self::owner_equals(e, owner) {
                    preferred_slot
                } else {
                    return Err(format!(
                        "preferred slot {preferred_slot} is owned by a different OSD owner"
                    ));
                }
            } else {
                // Scan like RTSSSharedMemoryNET sample: start at 1
                let mut found: Option<u32> = None;
                for i in 1..osd_size {
                    let e = self.osd_entry_ptr(i)?;
                    if Self::owner_is_empty(e) {
                        let reported = read_u32(self.base, 20);
                        let stride = osd_stride_bytes(version, reported);
                        Self::prepare_entry_for_write(e, stride);
                        Self::write_owner(e, owner)?;
                        found = Some(i);
                        break;
                    }
                    if Self::owner_equals(e, owner) {
                        found = Some(i);
                        break;
                    }
                }
                found.ok_or_else(|| "No free RTSS OSD slot (and owner not found)".to_string())?
            };

            let e = self.osd_entry_ptr(slot)?;
            let reported = read_u32(self.base, 20);
            let stride = osd_stride_bytes(version, reported);
            // If we re-use an existing slot for the same owner, clear first to avoid stale szOSD/szOSDEx/buffer data.
            if Self::owner_equals(e, owner) {
                Self::prepare_entry_for_write(e, stride);
                Self::write_owner(e, owner)?;
            }
            Self::write_osd_strings(e, version, text)?;
            self.bump_osd_frame();
            Ok(())
        }

        pub fn release_owner(&self, owner: &CString) -> Result<(), String> {
            let version = read_u32(self.base, 4);
            let reported = read_u32(self.base, 20);
            let stride = osd_stride_bytes(version, reported);
            let osd_size = read_u32(self.base, 28);
            for i in 1..osd_size {
                let e = self.osd_entry_ptr(i)?;
                if Self::owner_equals(e, owner) {
                    Self::clear_entry(e, stride);
                    self.bump_osd_frame();
                    break;
                }
            }
            Ok(())
        }

        pub fn clear_all_slots(&self) -> Result<(), String> {
            let version = read_u32(self.base, 4);
            let reported = read_u32(self.base, 20);
            let stride = osd_stride_bytes(version, reported);
            let osd_size = read_u32(self.base, 28);
            for i in 1..osd_size {
                let e = self.osd_entry_ptr(i)?;
                Self::clear_entry(e, stride);
            }
            self.bump_osd_frame();
            Ok(())
        }
    }

    impl Drop for RtssMap {
        fn drop(&mut self) {
            unsafe {
                if !self.base.is_null() {
                    let _ = UnmapViewOfFile(self.base as *mut c_void);
                }
                if !self.h_map.is_null() {
                    let _ = CloseHandle(self.h_map);
                }
            }
        }
    }

    fn read_u32(base: *mut u8, offset: usize) -> u32 {
        unsafe { *(base.add(offset) as *const u32) }
    }

    pub fn update(owner: &str, text: &str, preferred_slot: u32) -> Result<(), String> {
        let owner_c = CString::new(owner).map_err(|_| "owner contains NUL".to_string())?;
        let text_c = CString::new(text).map_err(|_| "text contains NUL".to_string())?;
        let map = RtssMap::open()?;
        map.update(&owner_c, &text_c, preferred_slot)
    }

    pub fn release(owner: &str) -> Result<(), String> {
        let owner_c = CString::new(owner).map_err(|_| "owner contains NUL".to_string())?;
        let map = RtssMap::open()?;
        map.release_owner(&owner_c)
    }

    pub fn clear_all() -> Result<(), String> {
        let map = RtssMap::open()?;
        map.clear_all_slots()
    }

    pub fn debug_dump(limit_slots: u32) -> Result<String, String> {
        let _ = limit_slots;
        let map = RtssMap::open()?;
        let version = read_u32(map.base, 4);
        let reported = read_u32(map.base, 20);
        let arr_off = read_u32(map.base, 24);
        let arr_size = read_u32(map.base, 28);
        let mut lines = Vec::new();
        lines.push(format!(
            "RTSS v=0x{version:08X} entry_size={} arr_off={} slots={}",
            reported, arr_off, arr_size
        ));
        lines.push("slot dump temporarily disabled (unsafe mapping layouts observed)".to_string());
        Ok(lines.join("\n"))
    }

    pub fn sleep_ms(ms: u32) {
        unsafe { Sleep(ms) };
    }
}

#[cfg(windows)]
pub use imp::{release, sleep_ms, update};
#[cfg(windows)]
pub use imp::{clear_all, debug_dump};

#[cfg(not(windows))]
pub fn update(_owner: &str, _text: &str, _preferred_slot: u32) -> Result<(), String> {
    Err("RTSS OSD is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn release(_owner: &str) -> Result<(), String> {
    Err("RTSS OSD is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn debug_dump(_limit_slots: u32) -> Result<String, String> {
    Err("RTSS OSD is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn clear_all() -> Result<(), String> {
    Err("RTSS OSD is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn sleep_ms(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}
