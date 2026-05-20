use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use bytesize::ByteSize;

/// 进度显示 trait
pub trait ProgressDisplay {
    fn update(&self, bytes_processed: u64);
    fn finish(&self);
}

/// 简单的进度显示器
pub struct SimpleProgress {
    total: u64,
    current: Arc<Mutex<u64>>,
    start_time: Instant,
    last_update: Arc<Mutex<Instant>>,
    update_interval_ms: u64,
}

impl SimpleProgress {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
            last_update: Arc::new(Mutex::new(Instant::now())),
            update_interval_ms: 100, // 每100ms更新一次
        }
    }
    
    fn should_update(&self) -> bool {
        let last_update = *self.last_update.lock().unwrap();
        last_update.elapsed().as_millis() >= self.update_interval_ms as u128
    }
}

impl ProgressDisplay for SimpleProgress {
    fn update(&self, bytes_processed: u64) {
        let mut current = self.current.lock().unwrap();
        *current += bytes_processed;
        
        if self.should_update() {
            let percent = if self.total > 0 {
                (*current as f64 / self.total as f64 * 100.0) as u8
            } else {
                0
            };
            
            let elapsed = self.start_time.elapsed();
            let speed = if elapsed.as_secs() > 0 {
                *current as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };
            
            let output = format!(
                "\rProgress: {}% | {}/{} | {}/s",
                percent,
                ByteSize(*current),
                ByteSize(self.total),
                ByteSize(speed as u64)
            );
            
            print!("{}", output);
            io::stdout().flush().ok();
            
            *self.last_update.lock().unwrap() = Instant::now();
        }
    }
    
    fn finish(&self) {
        let current = *self.current.lock().unwrap();
        let elapsed = self.start_time.elapsed();
        
        if elapsed.as_secs() > 0 {
            let speed = current as f64 / elapsed.as_secs_f64();
            println!(
                "\nCompleted! Total: {} in {:.2}s ({}/s)",
                ByteSize(current),
                elapsed.as_secs_f32(),
                ByteSize(speed as u64)
            );
        } else {
            println!("\nCompleted! Total: {}", ByteSize(current));
        }
    }
}

/// 带进度条的进度显示器
pub struct ProgressBar {
    total: u64,
    current: Arc<Mutex<u64>>,
    start_time: Instant,
    message: String,
    show_bytes: bool,
}

impl ProgressBar {
    pub fn new(total: u64, message: &str) -> Self {
        Self {
            total,
            current: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
            message: message.to_string(),
            show_bytes: true,
        }
    }
    
    pub fn with_bytes_display(mut self, show: bool) -> Self {
        self.show_bytes = show;
        return self;
    }
    
    fn display(&self) {
        let current = *self.current.lock().unwrap();
        
        if self.total == 0 {
            return;
        }
        
        let percent = (current as f64 / self.total as f64 * 100.0) as u8;
        let elapsed = self.start_time.elapsed();
        
        // 计算速度
        let speed = if elapsed.as_secs() > 0 {
            current as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        // 计算剩余时间
        let remaining = if speed > 0.0 {
            let remaining_bytes = self.total.saturating_sub(current) as f64;
            Duration::from_secs_f64(remaining_bytes / speed)
        } else {
            Duration::from_secs(0)
        };
        
        // 创建进度条
        let width = 50;
        let filled = (percent as usize * width) / 100;
        let bar = "=".repeat(filled) + &" ".repeat(width - filled);
        
        // 格式化输出
        let mut output = format!("\r{} [{}] {}%", self.message, bar, percent);
        
        if self.show_bytes {
            let current_fmt = ByteSize(current);
            let total_fmt = ByteSize(self.total);
            let speed_fmt = format!("{}/s", ByteSize(speed as u64));
            
            output += &format!(" {}/{} ({})", current_fmt, total_fmt, speed_fmt);
        }
        
        // 显示剩余时间
        if remaining.as_secs() > 0 {
            let mins = remaining.as_secs() / 60;
            let secs = remaining.as_secs() % 60;
            output += &format!(" ETA: {:02}:{:02}", mins, secs);
        }
        
        print!("{}", output);
        io::stdout().flush().ok();
    }
    
    pub fn get_current(&self) -> u64 {
        *self.current.lock().unwrap()
    }
}

impl ProgressDisplay for ProgressBar {
    fn update(&self, bytes_processed: u64) {
        let mut current = self.current.lock().unwrap();
        *current += bytes_processed;
        self.display();
    }
    
    fn finish(&self) {
        self.display();
        println!();
    }
}
