use std::env;
use std::fs;
use std::process;

/// CLI 程序入口。
///
/// 它做的事情非常“薄”：
/// 1) 读入 ArkTS 源文件（.ets）
/// 2) 调用库函数 `arkts2rust::compile` 得到 Rust 源码字符串
/// 3) 把 Rust 源码写到输出文件（默认 output.rs）
///
/// 语法/编译逻辑都在 `src/lib.rs` 以及内部模块里，这里只负责 I/O 和参数解析。
fn main() {
    let mut args = env::args().skip(1);

    let input_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: arkts2rust <input.ets> [-o <output.rs>]");
            process::exit(2);
        }
    };

    // 解析可选参数：
    // -o / --output <path>
    let mut output_path: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                // 下一个参数就是输出路径
                output_path = args.next();
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                eprintln!("Usage: arkts2rust <input.ets> [-o <output.rs>]");
                process::exit(2);
            }
        }
    }

    // 不传 -o 时，默认输出到当前目录下的 output.rs
    let output_path = output_path.unwrap_or_else(|| "output.rs".to_string());

    // 读取输入源文件
    let src = match fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read input file {input_path}: {e}");
            process::exit(2);
        }
    };

    // 调用库函数进行编译（返回 Rust 源码字符串）
    match arkts2rust::compile(&src) {
        Ok(rust_code) => {
            // 写出到文件
            if let Err(e) = fs::write(&output_path, rust_code) {
                eprintln!("Failed to write output file {output_path}: {e}");
                process::exit(2);
            }
        }
        Err(e) => {
            // 编译错误：错误中包含 code 和 span（行列号）方便定位
            eprintln!("Compile failed: {e}");
            process::exit(1);
        }
    }
}
