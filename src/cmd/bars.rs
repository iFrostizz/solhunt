use indicatif::{ProgressBar, ProgressStyle};

pub fn get_bar(len: u64, message: String) -> ProgressBar {
    let bar = ProgressBar::new(len);

    bar.set_style(
        ProgressStyle::with_template(
            "{msg} {spinner:.blue} [{elapsed_precise}] {bar:100.cyan/blue} [{human_pos}/{human_len}]",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    bar.set_message(message);

    bar
}
