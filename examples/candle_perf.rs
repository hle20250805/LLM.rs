use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Semaphore};
use llmrs::backend::EmbeddingBackend;
use sysinfo::System;

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         Candle 向量推理性能测试 (纯推理层)                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut sys = System::new_all();
    sys.refresh_all();
    let pid = sysinfo::get_current_pid().unwrap();

    println!("[{}] 正在加载模型...", timestamp());
    let load_start = Instant::now();
    let backend = Arc::new(
        llmrs::backend::candle::CandleBackend::new("./models/Yuan-embedding-2.0-zh".to_string())
            .expect("Failed to create backend"),
    );
    println!("[{}] 模型加载完成，耗时: {:.2}s\n", timestamp(), load_start.elapsed().as_secs_f64());

    sys.refresh_all();
    let mem_after_load = get_process_memory(&sys, pid);
    println!("[{}] 模型加载后内存: {:.0} MB\n", timestamp(), mem_after_load);

    println!("[{}] 预热中...", timestamp());
    let _ = backend.embed(vec!["预热文本用于初始化模型状态".to_string()], true, 1).await;
    println!("[{}] 预热完成\n", timestamp());

    let test_text = "这是一段用于性能测试的中文文本内容，长度约为三十个汉字左右。";
    let test_text_vec = vec![test_text.to_string()];

    let concurrency_levels = vec![20, 50, 70];
    let test_duration = Duration::from_secs(60);

    println!("测试配置:");
    println!("  - 测试文本: \"{}\"", test_text);
    println!("  - 文本长度: {} 字符", test_text.chars().count());
    println!("  - 每种并发测试时长: 60秒");
    println!("  - 并发级别: {:?}\n", concurrency_levels);

    println!("{}", "═".repeat(80));

    let mut total_all_requests = 0usize;

    for concurrency in concurrency_levels {
        println!("\n┌─────────────────────────────────────────────────────────────────────────┐");
        println!("│  并发数: {:^60} │", concurrency);
        println!("└─────────────────────────────────────────────────────────────────────────┘");
        println!("[{}] 开始测试 (并发 {} 持续 60 秒)...", timestamp(), concurrency);

        let semaphore = Arc::new(Semaphore::new(concurrency));
        let backend_clone = backend.clone();
        let test_text_clone = test_text_vec.clone();
        let request_count = Arc::new(AtomicUsize::new(0));
        let success_count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::new()));
        let running = Arc::new(AtomicUsize::new(1));

        let start_time = Instant::now();

        let progress_request_count = request_count.clone();
        let progress_success_count = success_count.clone();
        let progress_running = running.clone();

        let progress_handle = tokio::spawn(async move {
            let mut last_count = 0;
            let mut last_print = Instant::now();
            let mut sys_monitor = System::new_all();

            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                if progress_running.load(Ordering::Relaxed) == 0 {
                    break;
                }

                let current_count = progress_request_count.load(Ordering::Relaxed);
                let success = progress_success_count.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed().as_secs();

                sys_monitor.refresh_all();
                let (cpu, mem) = get_process_stats(&sys_monitor, pid);

                if last_print.elapsed() >= Duration::from_secs(5) {
                    let rps = if elapsed > 0 {
                        success as f64 / elapsed as f64
                    } else {
                        0.0
                    };
                    let delta = current_count.saturating_sub(last_count);
                    let instant_rps = delta as f64 / 5.0;
                    println!("[{}] 进度: {}/60s | 总请求: {} | 成功: {} | 累计RPS: {:.1} | 瞬时RPS: {:.1} | CPU: {:.1}% | 内存: {:.0}MB",
                        timestamp(), elapsed, current_count, success, rps, instant_rps, cpu, mem);
                    last_print = Instant::now();
                    last_count = current_count;
                }
            }
        });

        let mut handles = Vec::new();

        while start_time.elapsed() < test_duration {
            match semaphore.clone().try_acquire_owned() {
                Ok(permit) => {
                    let backend_ref = backend_clone.clone();
                    let text = test_text_clone.clone();
                    let req_count = request_count.clone();
                    let succ_count = success_count.clone();
                    let lat = latencies.clone();

                    req_count.fetch_add(1, Ordering::Relaxed);

                    let handle = tokio::spawn(async move {
                        let req_start = Instant::now();
                        let result = backend_ref.embed(text, true, 1).await;
                        let duration = req_start.elapsed();
                        drop(permit);

                        if result.is_ok() {
                            succ_count.fetch_add(1, Ordering::Relaxed);
                            let mut l = lat.lock().await;
                            l.push(duration);
                        }
                    });
                    handles.push(handle);
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        }

        running.store(0, Ordering::Relaxed);
        progress_handle.await.unwrap();

        println!("[{}] 测试完成，等待所有请求结束...", timestamp());

        let remaining = handles.len();
        if remaining > 0 {
            println!("[{}] 还有 {} 个请求未完成，等待中...", timestamp(), remaining);
        }

        for handle in handles {
            let _ = handle.await;
        }

        let total_duration = start_time.elapsed();
        let total_requests = request_count.load(Ordering::Relaxed);
        let success = success_count.load(Ordering::Relaxed);
        let mut latencies_vec = latencies.lock().await.clone();
        latencies_vec.sort();

        total_all_requests += success;

        let avg_latency = if !latencies_vec.is_empty() {
            latencies_vec.iter().sum::<Duration>() / latencies_vec.len() as u32
        } else {
            Duration::ZERO
        };

        let p95_index = (latencies_vec.len() as f64 * 0.95) as usize;
        let p99_index = (latencies_vec.len() as f64 * 0.99) as usize;
        let min_latency = latencies_vec.first().copied().unwrap_or(Duration::ZERO);
        let max_latency = latencies_vec.last().copied().unwrap_or(Duration::ZERO);

        let p95_latency = latencies_vec.get(p95_index.min(latencies_vec.len().saturating_sub(1))).copied().unwrap_or(Duration::ZERO);
        let p99_latency = latencies_vec.get(p99_index.min(latencies_vec.len().saturating_sub(1))).copied().unwrap_or(Duration::ZERO);

        let rps = if total_duration.as_secs_f64() > 0.0 {
            success as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        sys.refresh_all();
        let (cpu, mem) = get_process_stats(&sys, pid);

        println!("\n┌─────────────────────────────────────────────────────────────────────────┐");
        println!("│  性能指标                                                                │");
        println!("├─────────────────────────────────────────────────────────────────────────┤");
        println!("│  平均延迟:        {:>15.4} s                              │", avg_latency.as_secs_f64());
        println!("│  P95延迟:         {:>15.4} s                              │", p95_latency.as_secs_f64());
        println!("│  P99延迟:         {:>15.4} s                              │", p99_latency.as_secs_f64());
        println!("│  最小延迟:        {:>15.4} s                              │", min_latency.as_secs_f64());
        println!("│  最大延迟:        {:>15.4} s                              │", max_latency.as_secs_f64());
        println!("│  每秒请求数(RPS): {:>15.2} req/s                         │", rps);
        println!("│  处理总请求数:    {:>15} Req                             │", success);
        println!("│  发起总请求数:    {:>15} Req                             │", total_requests);
        println!("│  成功率:          {:>15.2} %                              │", if total_requests > 0 { (success as f64 / total_requests as f64) * 100.0 } else { 0.0 });
        println!("│  总耗时:          {:>15.2} s                              │", total_duration.as_secs_f64());
        println!("├─────────────────────────────────────────────────────────────────────────┤");
        println!("│  CPU使用率:       {:>15.1} %                              │", cpu);
        println!("│  内存使用量:      {:>15.0} MB                             │", mem);
        println!("└─────────────────────────────────────────────────────────────────────────┘");

        println!("\n[{}] 等待3秒后开始下一轮测试...", timestamp());
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    println!("\n{}", "═".repeat(80));
    println!("                        总体测试结果汇总");
    println!("{}", "═".repeat(80));
    println!("│  总调用次数:      {:>15} Req                             │", total_all_requests);
    println!("└─────────────────────────────────────────────────────────────────────────┘");
    println!("[{}] 所有测试完成！", timestamp());
}

fn timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let secs = duration.as_secs();
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn get_process_memory(sys: &System, pid: sysinfo::Pid) -> f64 {
    sys.process(pid)
        .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0)
}

fn get_process_stats(sys: &System, pid: sysinfo::Pid) -> (f32, f64) {
    sys.process(pid)
        .map(|p| {
            let cpu = p.cpu_usage();
            let mem = p.memory() as f64 / 1024.0 / 1024.0;
            (cpu, mem)
        })
        .unwrap_or((0.0, 0.0))
}
