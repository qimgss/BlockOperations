use std::ptr;
use libc;

// 块设备专用 ioctl 命令
const BLKGETSIZE64: u32 = 0x80081272;  // 获取块设备大小
const BLKFLSBUF: u32 = 0x1261;        // 刷新块设备缓存

pub struct BlockDevice {
    fd: i32,
    size: u64,
}

impl BlockDevice {
    /// 以特殊方式打开块设备
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        println!("Opening block device: {}", path);
        
        // 使用 libc 的 open 系统调用，以 O_RDWR | O_SYNC 方式打开
        let fd = unsafe {
            libc::open(
                path.as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_SYNC | libc::O_LARGEFILE,
            )
        };
        
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        
        // 获取设备大小
        let mut size: u64 = 0;
        let ret = unsafe {
            // 将 u32 转换为 i32（ioctl 期望 i32）
            libc::ioctl(fd, BLKGETSIZE64 as i32, &mut size as *mut u64)
        };
        
        if ret < 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error());
        }
        
        println!("Block device opened, size: {} bytes", size);
        Ok(Self { fd, size })
    }
    
    /// 读取块设备数据
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
    
    /// 写入块设备数据
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
    
    /// 刷新块设备缓存
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
    
    /// 重新读取分区表（仅适用于整个磁盘，不适用于分区）
    /// 对于分区设备，这个方法应该被跳过
    pub fn reread_part(&self) -> Result<(), std::io::Error> {
        // 注意：BLKRRPART 通常用于整个磁盘设备，而不是分区
        // 我们返回一个 Ok，但打印警告
        println!("Warning: reread_part() called on partition device, skipping");
        Ok(())
    }
    
    pub fn size(&self) -> u64 {
        self.size
    }
    
    pub fn fd(&self) -> i32 {
        self.fd
    }
}

impl Drop for BlockDevice {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}