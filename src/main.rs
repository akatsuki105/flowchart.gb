use std::env;
use std::process::exit;

mod analyzer;
mod dialog;
mod disasm;
mod parser;
mod variable;

use std::path::Path;

fn main() {
    exit(run());
}

fn run() -> i32 {
    let cd_path = env::current_dir().unwrap();
    let cd = cd_path.to_str().unwrap();
    // println!("The current directory is {}", cd);

    // 開始地点となるファイルを受け取る
    let file_path = dialog::open_file_dialog();

    // 解析結果格納するフォルダを産出する
    let tmp = file_path.clone();
    let path = Path::new(&tmp);
    let extension = path.extension();
    let outputdir = path.parent().unwrap().to_str().unwrap();

    match extension {
        Some(ext) if ext == "gb" || ext == "gbc" => {
            // 逆アセンブルする
            disasm::disassemble(&file_path);
            let file_path = cd.to_string() + "\\disassembly\\game.asm";
            let init_label = "Jump_000_0150";

            // 解析を開始する
            let mut a = analyzer::Analyzer::new();
            a.init_analyze(vec![file_path]);

            // 解析結果をダンプする
            a.dump_flowchart(outputdir, init_label);
            return 0;
        }
        Some(ext) if ext == "asm" => {
            // コマンドライン引数をパースする
            let args: Vec<String> = env::args().collect();
            if args.len() <= 1 {
                println!("If a target is asm file, a label for the starting point is needed.");
                return 1;
            }
            let init_label = &args[1];

            // 解析を開始する
            let mut a = analyzer::Analyzer::new();
            a.init_analyze(vec![file_path]);

            // 解析結果をダンプする
            a.dump_flowchart(outputdir, init_label);
            return 0;
        }
        _ => {
            return 1;
        }
    }
}
