use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

pub fn create_progress_bar(len: u64) -> ProgressBar {
    //设置进度条 进度条的样式也会影响性能，进度条越简单性能也好，影响比较小
    let progress_bar = ProgressBar::new(len).with_finish(indicatif::ProgressFinish::AndLeave);
    let progress_style = ProgressStyle::default_bar()
        // .template("[{elapsed_precise}] {spinner} {pos:7}/{len:7} throughput:{per_sec} (eta:{eta})")
        .template("[{elapsed_precise}] {wide_bar} {pos}/{len} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    progress_bar.set_draw_target(draw_target);
    progress_bar
}
