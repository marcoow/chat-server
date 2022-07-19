use std::env;

pub fn is_dev_mode() -> bool {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(dev_mode_switch) => dev_mode_switch == "--dev",
        None => false,
    }
}
