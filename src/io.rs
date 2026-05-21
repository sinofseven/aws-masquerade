use std::fmt::Display;
use std::io;
use std::io::Write;

// https://magidropack.hatenablog.com/entry/2018/12/18/194442
pub fn get_input<T>(message: T) -> String
where
    T: Display,
{
    print!("{}", message);
    let mut input = String::new();
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).ok();

    input.trim().to_string()
}

pub fn get_confirm_with_default<T>(message: T, default: bool) -> Option<bool>
where
    T: Display,
{
    let resp = get_input(message);
    match resp.to_lowercase().as_str() {
        "" => Some(default),
        "y" => Some(true),
        "yes" => Some(true),
        "n" => Some(false),
        "no" => Some(false),
        _ => None,
    }
}
