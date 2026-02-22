use crate::{ACCError, Result};
use std::slice;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use winapi::shared::winerror::ERROR_FILE_NOT_FOUND;
#[cfg(windows)]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(windows)]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use winapi::um::memoryapi::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ};
#[cfg(windows)]
use winapi::um::winnt::{GENERIC_READ, HANDLE};

/// Low-level shared memory reader for Windows named shared memory objects.
pub struct SharedMemoryReader {
    #[cfg(windows)]
    handle: HANDLE,
    #[cfg(not(windows))]
    _handle: (),
    ptr: *mut u8,
    size: usize,
    name: String,
}

impl SharedMemoryReader {
    /// Create a new shared memory reader for the given named memory object.
    #[cfg(windows)]
    pub fn new(name: &str, size: usize) -> Result<Self> {
        // Convert the name to UTF-16 (wide string) for OpenFileMappingW
        let wide_name: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(std::iter::once(0)) // Add null terminator
            .collect();

        // Open the shared memory object
        let handle = unsafe {
            OpenFileMappingW(
                GENERIC_READ,
                0, // Don't inherit handle
                wide_name.as_ptr(),
            )
        };

        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { GetLastError() };
            if error_code == ERROR_FILE_NOT_FOUND {
                return Err(ACCError::SharedMemoryNotAvailable);
            }
            return Err(ACCError::SharedMemoryOpen(format!(
                "Failed to open shared memory '{}', error code: {}",
                name, error_code
            )));
        }

        // Map the shared memory into our address space
        let ptr = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, size) } as *mut u8;

        if ptr.is_null() {
            unsafe { CloseHandle(handle) };
            let error_code = unsafe { GetLastError() };
            return Err(ACCError::SharedMemoryMap(format!(
                "Failed to map shared memory '{}', error code: {}",
                name, error_code
            )));
        }

        Ok(Self {
            handle,
            ptr,
            size,
            name: name.to_string(),
        })
    }

    /// Create a new shared memory reader (non-Windows stub).
    #[cfg(not(windows))]
    pub fn new(_name: &str, _size: usize) -> Result<Self> {
        Err(ACCError::SharedMemoryNotAvailable)
    }

    /// Get the raw pointer to the shared memory
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get the size of the shared memory region
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get a slice view of the shared memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.size) }
    }

    /// Read data from a specific offset
    pub fn read_at<T: Copy>(&self, offset: usize) -> Result<T> {
        if offset + std::mem::size_of::<T>() > self.size {
            return Err(ACCError::InvalidData(format!(
                "Read beyond buffer bounds: offset {} + size {} > buffer size {}",
                offset,
                std::mem::size_of::<T>(),
                self.size
            )));
        }

        unsafe {
            let ptr = self.ptr.add(offset) as *const T;
            Ok(ptr.read_unaligned())
        }
    }

    /// Read an array of data from a specific offset
    pub fn read_array_at<T: Copy>(&self, offset: usize, count: usize) -> Result<Vec<T>> {
        let total_size = count * std::mem::size_of::<T>();
        if offset + total_size > self.size {
            return Err(ACCError::InvalidData(format!(
                "Read beyond buffer bounds: offset {} + size {} > buffer size {}",
                offset, total_size, self.size
            )));
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let ptr = self.ptr.add(offset) as *const T;
            for i in 0..count {
                result.push(ptr.add(i).read_unaligned());
            }
        }
        Ok(result)
    }

    /// Read a UTF-16 string from a specific offset, up to a null terminator or buffer end
    pub fn read_utf16_string_at(&self, offset: usize, max_char_count: usize) -> Result<String> {
        // Calculate the maximum number of u16s we can safely read
        let max_bytes = self.size.saturating_sub(offset);
        let max_chars_in_buffer = max_bytes / 2;
        if max_chars_in_buffer == 0 {
            return Err(ACCError::InvalidData(format!(
                "String read beyond buffer bounds: offset {} > buffer size {}",
                offset, self.size
            )));
        }
        let read_count = max_char_count.min(max_chars_in_buffer);
        unsafe {
            let ptr = self.ptr.add(offset) as *const u16;
            let slice = slice::from_raw_parts(ptr, read_count);
            // Find the null terminator
            let mut len = read_count;
            for (i, &ch) in slice.iter().enumerate() {
                if ch == 0 {
                    len = i;
                    break;
                }
            }
            String::from_utf16(&slice[..len])
                .map_err(|_| ACCError::InvalidData("Invalid UTF-16 string".to_string()))
        }
    }

    /// Get the name of this shared memory object
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for SharedMemoryReader {
    #[cfg(windows)]
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                UnmapViewOfFile(self.ptr as *const _);
            }
            if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.handle);
            }
        }
    }

    #[cfg(not(windows))]
    fn drop(&mut self) {
        // Nothing to do on non-Windows platforms
    }
}

unsafe impl Send for SharedMemoryReader {}
unsafe impl Sync for SharedMemoryReader {}