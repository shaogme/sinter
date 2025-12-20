//! 简单的同构日志记录工具，用于输出到控制台或终端。

use wasm_bindgen::JsValue;

/// 使用 `println!()` 风格的格式化将内容记录到控制台（在浏览器中）
/// 或通过 `println!()`（如果不在浏览器中）。
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => ($crate::logging::console_log(&format_args!($($t)*).to_string()))
}

/// 使用 `println!()` 风格的格式化将警告记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中）。
#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => ($crate::logging::console_warn(&format_args!($($t)*).to_string()))
}

/// 使用 `println!()` 风格的格式化将错误记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中）。
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => ($crate::logging::console_error(&format_args!($($t)*).to_string()))
}

/// 使用 `println!()` 风格的格式化将内容记录到控制台（在浏览器中）
/// 或通过 `println!()`（如果不在浏览器中），但仅在调试构建时。
#[macro_export]
macro_rules! debug_log {
    ($($x:tt)*) => {
        {
            if cfg!(debug_assertions) {
                $crate::log!($($x)*)
            }
        }
    }
}

/// 使用 `println!()` 风格的格式化将警告记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中），但仅在调试构建时。
#[macro_export]
macro_rules! debug_warn {
    ($($x:tt)*) => {
        {
            if cfg!(debug_assertions) {
                $crate::warn!($($x)*)
            }
        }
    }
}

/// 使用 `println!()` 风格的格式化将错误记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中），但仅在调试构建时。
#[macro_export]
macro_rules! debug_error {
    ($($x:tt)*) => {
        {
            if cfg!(debug_assertions) {
                $crate::error!($($x)*)
            }
        }
    }
}

const fn log_to_stdout() -> bool {
    cfg!(not(all(
        target_arch = "wasm32",
        not(any(target_os = "emscripten", target_os = "wasi"))
    )))
}

/// 将字符串记录到控制台（在浏览器中）
/// 或通过 `println!()`（如果不在浏览器中）。
pub fn console_log(s: &str) {
    #[allow(clippy::print_stdout)]
    if log_to_stdout() {
        println!("{s}");
    } else {
        web_sys::console::log_1(&JsValue::from_str(s));
    }
}

/// 将警告记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中）。
pub fn console_warn(s: &str) {
    if log_to_stdout() {
        eprintln!("{s}");
    } else {
        web_sys::console::warn_1(&JsValue::from_str(s));
    }
}

/// 将错误记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中）。
#[inline(always)]
pub fn console_error(s: &str) {
    if log_to_stdout() {
        eprintln!("{s}");
    } else {
        web_sys::console::error_1(&JsValue::from_str(s));
    }
}

/// 将字符串记录到控制台（在浏览器中）
/// 或通过 `println!()`（如果不在浏览器中），但仅在调试构建中。
#[inline(always)]
pub fn console_debug_log(s: &str) {
    if cfg!(debug_assertions) {
        console_log(s)
    }
}

/// 将警告记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中），但仅在调试构建中。
#[inline(always)]
pub fn console_debug_warn(s: &str) {
    if cfg!(debug_assertions) {
        console_warn(s)
    }
}

/// 将错误记录到控制台（在浏览器中）
/// 或通过 `eprintln!()`（如果不在浏览器中），但仅在调试构建中。
#[inline(always)]
pub fn console_debug_error(s: &str) {
    if cfg!(debug_assertions) {
        console_error(s)
    }
}
