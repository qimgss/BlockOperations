use std::ptr;
use libc;

const BLKGETSIZE64: u32 = 0x80081272;
const BLKFLSBUF: u32 = 0x1261;

pub struct BlockDevice {
    fd: i32,
    size: u64,
}

impl BlockDevice {
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        println!("Opening block device: {}", path);
        
        let fd = unsafe {
            libc::open(
                path.as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_SYNC | libc::O_LARGEFILE,
            )
        };
        
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        
        let mut size: u64 = 0;
        let ret = unsafe {
            libc::ioctl(fd, BLKGETSIZE64 as i32, &mut size as *mut u64)
        };
        
        if ret < 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error());
        }
        
        println!("Block device opened, size: {} bytes", size);
        Ok(Self { fd, size })
    }
    
    pub fn read(&self, buffer: &mut [u8], offset: u64) -> Result<usize, std::io::Error> {
        let ret = unsafe {
            libc::pread(
                self.fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                offset as libc::off_t,
            )
        };
        
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }
    
    pub fn write(&self, buffer: &[u8], offset: u64) -> Result<usize, std::io::Error> {
        let ret = unsafe {
            libc::pwrite(
                self.fd,
                buffer.as_ptr() as *const libc::c_void,
                buffer.len(),
                offset as libc::off_t,
            )
        };
        
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }
    
    pub fn flush(&self) -> Result<(), std::io::Error> {
        let ret = unsafe {
            libc::ioctl(self.fd, BLKFLSBUF as i32, ptr::null_mut::<libc::c_void>())
        };
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    
    pub fn size(&self) -> u64 {
        self.size
    }
    
    // 删除未使用的 reread_part 和 fd 方法
}

impl Drop for BlockDevice {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}