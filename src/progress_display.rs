use std::io::{self, Write};
use std::time::Duration;
use crate::batch_processor::BatchStats;

/// 进度显示器
pub struct ProgressDisplay {
    /// 是否启用彩色输出
    pub colored: bool,
    /// 进度条宽度
    pub bar_width: usize,
    /// 上次更新时间
    last_update: std::time::Instant,
    /// 更新间隔（毫秒）
    update_interval: Duration,
}

impl Default for ProgressDisplay {
    fn default() -> Self {
        Self {
            colored: true,
            bar_width: 50,
            last_update: std::time::Instant::now(),
            update_interval: Duration::from_millis(100), // 100ms更新间隔
        }
    }
}

impl ProgressDisplay {
    /// 创建新的进度显示器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否启用彩色输出
    pub fn with_colored(mut self, colored: bool) -> Self {
        self.colored = colored;
        self
    }

    /// 设置进度条宽度
    pub fn with_bar_width(mut self, width: usize) -> Self {
        self.bar_width = width;
        self
    }

    /// 显示进度信息
    pub fn show_progress(&mut self, stats: &BatchStats, current_file: &str) {
        // 限制更新频率
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }
        self.last_update = now;

        // 清除当前行
        print!("\r");
        
        // 计算进度
        let progress = stats.progress_percentage();
        let filled_width = ((progress / 100.0) * self.bar_width as f64) as usize;
        let empty_width = self.bar_width - filled_width;
        
        // 构建进度条
        let progress_bar = if self.colored {
            format!(
                "{}{}{}[{}{}{}{}{}] {:.1}%{}",
                "\x1b[2K", // 清除整行
                "\x1b[32m", // 绿色
                "\x1b[1m",  // 粗体
                "\x1b[42m", // 绿色背景
                "=".repeat(filled_width),
                "\x1b[0m",  // 重置
                "\x1b[37m", // 白色
                " ".repeat(empty_width),
                "\x1b[0m",  // 重置
                progress
            )
        } else {
            format!(
                "[{}{}] {:.1}%",
                "=".repeat(filled_width),
                " ".repeat(empty_width),
                progress
            )
        };
        
        // 格式化统计信息
        let stats_info = format!(
            " ({}/{}) 速度: {:.1} 文件/秒",
            stats.processed_files,
            stats.total_files,
            stats.processing_speed()
        );
        
        // 格式化预估时间
        let eta = stats.estimated_remaining_time();
        let eta_info = if eta.as_secs() > 0 {
            format!(" ETA: {}", self.format_duration(eta))
        } else {
            String::new()
        };
        
        // 格式化当前文件信息
        let current_file_info = if !current_file.is_empty() {
            let max_filename_len = 30;
            let display_name = if current_file.len() > max_filename_len {
                format!("...{}", &current_file[current_file.len() - max_filename_len + 3..])
            } else {
                current_file.to_string()
            };
            format!(" | {}", display_name)
        } else {
            String::new()
        };
        
        // 输出完整的进度信息
        print!("{}{}{}{}", progress_bar, stats_info, eta_info, current_file_info);
        io::stdout().flush().unwrap();
    }

    /// 显示最终结果
    pub fn show_final_result(&self, stats: &BatchStats) {
        println!(); // 换行
        
        let total_time = stats.start_time.elapsed();
        
        if self.colored {
            println!("\x1b[32m\x1b[1m=== 批量处理完成 ===\x1b[0m");
            println!("\x1b[36m总文件数:\x1b[0m {}", stats.total_files);
            println!("\x1b[32m成功解码:\x1b[0m {}", stats.successful_files);
            
            if stats.failed_files > 0 {
                println!("\x1b[31m解码失败:\x1b[0m {}", stats.failed_files);
            }
            
            println!("\x1b[33m总二维码数:\x1b[0m {}", stats.total_qr_codes);
            println!("\x1b[35m平均速度:\x1b[0m {:.2} 文件/秒", stats.processing_speed());
            println!("\x1b[34m总耗时:\x1b[0m {}", self.format_duration(total_time));
            
            // 成功率
            let success_rate = if stats.total_files > 0 {
                (stats.successful_files as f64 / stats.total_files as f64) * 100.0
            } else {
                0.0
            };
            
            let success_color = if success_rate >= 90.0 {
                "\x1b[32m" // 绿色
            } else if success_rate >= 70.0 {
                "\x1b[33m" // 黄色
            } else {
                "\x1b[31m" // 红色
            };
            
            println!("{}成功率:\x1b[0m {:.1}%", success_color, success_rate);
        } else {
            println!("=== 批量处理完成 ===");
            println!("总文件数: {}", stats.total_files);
            println!("成功解码: {}", stats.successful_files);
            println!("解码失败: {}", stats.failed_files);
            println!("总二维码数: {}", stats.total_qr_codes);
            println!("平均速度: {:.2} 文件/秒", stats.processing_speed());
            println!("总耗时: {}", self.format_duration(total_time));
            
            let success_rate = if stats.total_files > 0 {
                (stats.successful_files as f64 / stats.total_files as f64) * 100.0
            } else {
                0.0
            };
            println!("成功率: {:.1}%", success_rate);
        }
    }

    /// 显示错误信息
    pub fn show_error(&self, message: &str) {
        if self.colored {
            eprintln!("\x1b[31m\x1b[1m错误:\x1b[0m {}", message);
        } else {
            eprintln!("错误: {}", message);
        }
    }

    /// 显示警告信息
    pub fn show_warning(&self, message: &str) {
        if self.colored {
            println!("\x1b[33m\x1b[1m警告:\x1b[0m {}", message);
        } else {
            println!("警告: {}", message);
        }
    }

    /// 显示信息
    pub fn show_info(&self, message: &str) {
        if self.colored {
            println!("\x1b[36m\x1b[1m信息:\x1b[0m {}", message);
        } else {
            println!("信息: {}", message);
        }
    }

    /// 显示成功信息
    pub fn show_success(&self, message: &str) {
        if self.colored {
            println!("\x1b[32m\x1b[1m成功:\x1b[0m {}", message);
        } else {
            println!("成功: {}", message);
        }
    }

    /// 格式化时间duration
    fn format_duration(&self, duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let millis = duration.subsec_millis();
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else if seconds > 0 {
            format!("{}.{:03}s", seconds, millis)
        } else {
            format!("{}ms", millis)
        }
    }

    /// 清除当前行
    pub fn clear_line(&self) {
        print!("\r\x1b[2K");
        io::stdout().flush().unwrap();
    }

    /// 显示开始信息
    pub fn show_start_info(&self, directory: &str, total_files: usize, recursive: bool) {
        if self.colored {
            println!("\x1b[36m\x1b[1m=== 开始批量处理 ===\x1b[0m");
            println!("\x1b[33m目录:\x1b[0m {}", directory);
            println!("\x1b[33m文件数:\x1b[0m {}", total_files);
            println!("\x1b[33m递归处理:\x1b[0m {}", if recursive { "是" } else { "否" });
            println!();
        } else {
            println!("=== 开始批量处理 ===");
            println!("目录: {}", directory);
            println!("文件数: {}", total_files);
            println!("递归处理: {}", if recursive { "是" } else { "否" });
            println!();
        }
    }

    /// 显示文件处理结果
    pub fn show_file_result(&self, file_name: &str, success: bool, qr_count: usize, processing_time: Duration) {
        if success {
            if self.colored {
                println!(
                    "\x1b[32m✓\x1b[0m {} - 解码 {} 个二维码 ({:.3}s)",
                    file_name,
                    qr_count,
                    processing_time.as_secs_f64()
                );
            } else {
                println!(
                    "✓ {} - 解码 {} 个二维码 ({:.3}s)",
                    file_name,
                    qr_count,
                    processing_time.as_secs_f64()
                );
            }
        } else {
            if self.colored {
                println!(
                    "\x1b[31m✗\x1b[0m {} - 解码失败 ({:.3}s)",
                    file_name,
                    processing_time.as_secs_f64()
                );
            } else {
                println!(
                    "✗ {} - 解码失败 ({:.3}s)",
                    file_name,
                    processing_time.as_secs_f64()
                );
            }
        }
    }
}

/// 简单的进度回调函数
pub fn simple_progress_callback(stats: &BatchStats, current_file: &str) {
    let mut display = ProgressDisplay::new();
    display.show_progress(stats, current_file);
}

/// 详细的进度回调函数
pub fn detailed_progress_callback(stats: &BatchStats, current_file: &str) {
    let mut display = ProgressDisplay::new();
    display.show_progress(stats, current_file);
    
    // 每处理10个文件显示一次详细信息
    if stats.processed_files % 10 == 0 && stats.processed_files > 0 {
        println!(); // 换行
        display.show_info(&format!(
            "已处理 {} 个文件，成功 {} 个，失败 {} 个",
            stats.processed_files,
            stats.successful_files,
            stats.failed_files
        ));
    }
}