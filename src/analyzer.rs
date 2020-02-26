use super::parser;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use std::fs::File;
use std::io::Write;

type AbsPath = String;
type RelPath = String;

pub struct Analyzer {
    asm: HashMap<RelPath, parser::Nodes>, // アセンブリファイルの解析結果が入る
    text: HashSet<RelPath>,               // テキストファイルの相対パスを格納する
    remaining: Vec<AbsPath>,              // 解析対象のファイルの絶対パス一覧
    macros: HashSet<String>,
    cur_dir: AbsPath,
    base_dir: AbsPath,
    file_name: String,
}

impl Analyzer {
    pub fn new() -> Self {
        return Self {
            asm: HashMap::new(),
            text: HashSet::new(),
            remaining: Default::default(),
            macros: HashSet::new(),
            cur_dir: Default::default(),
            base_dir: Default::default(),
            file_name: Default::default(),
        };
    }

    pub fn init_analyze(&mut self, abs_file_path_list: Vec<String>) {
        for abs_file_path in abs_file_path_list {
            // すでに解析済みのasmファイルであるならskipする
            if self.base_dir != "" {
                let rel_file_path = self.to_relative(&abs_file_path);
                match self.asm.get(&rel_file_path) {
                    Some(_) => {
                        continue;
                    }
                    _ => {}
                }
            }
            self.analyze(abs_file_path);
        }
    }

    fn analyze(&mut self, abs_file_path: String) {
        if !is_asm(&abs_file_path) {
            println!("invalid format.");
            return;
        }

        // base_dirはabs_file_pathを起点とした相対パスを得るのに必要
        let path = Path::new(&abs_file_path);
        self.base_dir = path.parent().unwrap().to_str().unwrap().to_string();
        self.file_name = path.file_stem().unwrap().to_str().unwrap().to_string();

        // 最初のファイルを解析
        self.analyze_file(abs_file_path);

        // 2つ目以降のファイルを処理していく
        loop {
            if self.remaining.len() == 0 {
                break;
            }

            let next_file = self.remaining[0].clone();
            self.remaining.retain(|x| *x != next_file);
            self.analyze_file(next_file);
        }
    }

    pub fn analyze_file(&mut self, abs_file_path: String) {
        println!("analyze {}...", self.to_relative(&abs_file_path));
        // 拡張子を見てrednex asmファイルかどうかを判定し、処理を分岐させる
        if is_asm(&abs_file_path) {
            self.analyze_asm_file(abs_file_path);
        } else {
            self.analyze_text_file(abs_file_path);
        }
    }

    pub fn analyze_asm_file(&mut self, abs_file_path: String) {
        // 絶対パスを必要な形に加工する
        let path = Path::new(&abs_file_path);
        self.cur_dir = path.parent().unwrap().to_str().unwrap().to_string();
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();

        // 解析を行う
        let mut p = parser::Parser::new(abs_file_path, self.base_dir.clone(), self.macros.clone());
        let (result, macros) = p.parse(filename);
        // 解析結果を保存する
        for (filename, nodes) in result {
            self.asm.insert(filename, nodes);
        }

        // マクロテーブルを更新する
        self.macros = macros;
    }

    pub fn analyze_text_file(&mut self, abs_file_path: String) {
        // テキストファイルはそのテキストファイルの相対パスを保存しておく
        let rel_file_path = self.to_relative(&abs_file_path);
        self.text.insert(rel_file_path);
        self.remaining.retain(|x| *x != abs_file_path);
    }

    #[allow(dead_code)]
    /// 解析結果をjson形式で出力する
    pub fn dump_json(&self, dir_path: &str) {
        let path = Path::new(dir_path);

        if !path.exists() {
            println!("target dir does not exist");
            return;
        }

        if !path.is_dir() {
            println!("target path is not a directory");
            return;
        }

        let dir = dir_path.trim_end_matches('/').to_string();
        let filename = dir.clone() + "/" + &self.file_name + ".json";
        let mut file = File::create(filename).unwrap();
        let json = serde_json::to_string(&self.asm).unwrap() + "\n"; // "\n"を足しとかないとvscodeの自動整形機能でjsonが壊れる
        write!(file, "{}", json).unwrap();
        file.flush().unwrap();
    }

    fn get_node<'a>(&'a self, init_file: &str, label: &str) -> Option<&'a parser::Node> {
        let init_nodes = self.asm.get(init_file);
        match init_nodes {
            Some(nodes) => {
                match parser::get_node(nodes, label) {
                    Some(node) => {
                        return Some(node);
                    }
                    None => {}
                };
            }
            None => {}
        }

        for (_, nodes) in &self.asm {
            match parser::get_node(&nodes, label) {
                Some(node) => return Some(node),
                None => continue,
            };
        }

        return None;
    }

    /// 解析結果をflowchart形式で出力する
    pub fn dump_flowchart(&self, dir_path: &str, init_label: &str) {
        let path = Path::new(dir_path);

        if !path.exists() {
            println!("target dir does not exist");
            return;
        }

        if !path.is_dir() {
            println!("target path is not a directory");
            return;
        }

        let dir = dir_path.trim_end_matches('/').to_string();
        let filename = dir.clone() + "\\" + &self.file_name + ".flowchart";
        let mut file = File::create(filename.clone()).unwrap();

        let mut charts = "".to_string();
        for (filename, nodes) in &self.asm {
            let mut done = vec![];
            let header = format!(
                "--------------------------------------------------------------------\n[{}]",
                filename
            )
            .to_string();
            let mut ns = vec![header, "st=>start: Start".to_string()];
            let mut flows = vec![format!("st->{}", init_label)];
            let mut current_label = init_label;
            let mut current_node = match parser::get_node(&nodes, init_label) {
                Some(node) => node,
                None => continue,
            };
            loop {
                done.push(current_label);

                if current_node.next_cond != "" {
                    let next_label = &current_node.next;
                    ns.push(format!(
                        "{}=>parallel:  {}",
                        current_label, current_node.text
                    ));
                    if done.contains(&&current_node.next_cond[..]) {
                        flows.push(format!("{}(path1, right)->{}", current_label, next_label));
                        flows.push(format!(
                            "{}(path2, bottom)->{}",
                            current_label, &current_node.next_cond
                        ));
                    } else {
                        flows.push(format!(
                            "{}(path1, right)->{}",
                            current_label, &current_node.next_cond
                        ));
                        flows.push(format!("{}(path2, bottom)->{}", current_label, next_label));
                    }
                    current_label = next_label;
                    // println!("{}", current_label);
                    match self.get_node(filename, current_label) {
                        Some(node) => {
                            current_node = node;
                        }
                        None => {
                            if current_label == "hl" {
                                ns.push(format!(
                                    "{}=>operation:  This flowchart ends here, because PC jumps to HL which dynamically changes.",
                                    current_label
                                ));
                            } else {
                                ns.push(format!(
                                    "{}=>operation:  {}\n;Moved to another bank.",
                                    current_label, current_label
                                ));
                            }
                            break;
                        }
                    }
                } else if current_node.next != "" {
                    let next_label = &current_node.next;
                    ns.push(format!(
                        "{}=>operation:  {}",
                        current_label, current_node.text
                    ));
                    flows.push(format!("{}->{}", current_label, next_label));
                    current_label = next_label;
                    // println!("{}", current_label);
                    match self.get_node(filename, current_label) {
                        Some(node) => {
                            current_node = node;
                        }
                        None => {
                            if current_label == "hl" {
                                ns.push(format!(
                                    "{}=>operation:  This flowchart ends here, because PC jumps to HL which dynamically changes.",
                                    current_label
                                ));
                            } else {
                                ns.push(format!(
                                    "{}=>operation:  {}\n;Moved to another bank.",
                                    current_label, current_label
                                ));
                            }
                            break;
                        }
                    }
                } else {
                    ns.push(format!(
                        "{}=>operation:  {}",
                        current_label, current_node.text
                    ));
                    break;
                }

                if done.contains(&current_label) {
                    break;
                }
            }

            flows.push(format!("{}->e", current_label));
            charts += &ns.join("\n");
            charts += "\n";
            charts += "e=>end\n";
            charts += "\n";
            charts += &flows.join("\n");
            charts += "\n";
        }
        write!(file, "{}", charts).unwrap();
        file.flush().unwrap();
        println!("dump into {}...", filename);
    }

    // 最初に渡したabs_file_pathのディレクトリを起点とした相対パスを得るのに必要
    fn to_relative(&self, abs_file_path: &str) -> String {
        let path = Path::new(abs_file_path);
        let rel_file_path = path
            .strip_prefix(&self.base_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        return rel_file_path;
    }
}

fn is_asm(file_path: &str) -> bool {
    let path = Path::new(file_path);
    match path.extension() {
        Some(ext) if ext == "asm" => {
            return true;
        }
        _ => {
            return false;
        }
    }
}
