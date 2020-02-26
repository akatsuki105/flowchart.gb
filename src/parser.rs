use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize};

use super::variable::opcode;
use super::variable::token;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Element {
    Opcode {
        text: String,
        op: String,
        operand1: String,
        operand2: String,
        comment: String,
    },
    Text {
        text: String,
    },
    Include {
        text: String,
        target: String,
    },
    Macro {
        label: String,
        texts: Vec<String>,
        text: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub global: String,
    pub text: String,
    elements: Vec<Element>,
    pub next: String,
    pub next_cond: String,
    pub calls: Vec<String>,
}

pub type Nodes = HashMap<String, Node>;

#[derive(Serialize, Deserialize)]
pub struct Parser {
    text: Vec<String>,
    line: usize,
    nodes: Nodes,
    global: String,
    local: String,
    includes: Vec<String>,
    macros: HashSet<String>,
    cur_dir: String,
    base_dir: String,
}

impl Parser {
    pub fn new(file_path: String, base_dir: String, macros: HashSet<String>) -> Self {
        let file = File::open(&file_path).unwrap();
        let lines = BufReader::new(file).lines();
        let (num_of_line, _) = lines.size_hint();
        let mut text = Vec::with_capacity(num_of_line);
        for line in lines {
            let l = line.unwrap();
            text.push(l);
        }

        let global = "main";
        let mut nodes = HashMap::new();
        nodes.insert(
            global.to_string(),
            Node {
                global: global.to_string(),
                text: "".to_string(),
                elements: Default::default(),
                next: "".to_string(),
                next_cond: "".to_string(),
                calls: Default::default(),
            },
        );

        let cur_dir = Path::new(&file_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        return Self {
            text: text,
            line: 0,
            nodes: nodes,
            global: global.to_string(),
            local: "".to_string(),
            includes: Default::default(),
            macros: macros,
            cur_dir: cur_dir,
            base_dir,
        };
    }

    pub fn parse(&mut self, filename: String) -> (HashMap<String, Nodes>, HashSet<String>) {
        let mut result = HashMap::new();
        loop {
            let (end, results, macros) = self.parse_element();
            match results {
                Some(results) => {
                    for (filename, nodes) in results {
                        result.insert(filename, nodes);
                    }
                }
                None => {}
            }
            match macros {
                Some(macros) => {
                    for m in macros {
                        self.macros.insert(m);
                    }
                }
                None => {}
            }
            if end {
                break;
            }
        }

        let nodes = self.nodes.clone();
        result.insert(filename, nodes);
        return (result, self.macros.clone());
    }

    fn parse_element(
        &mut self,
    ) -> (
        bool,
        Option<HashMap<String, Nodes>>,
        Option<HashSet<String>>,
    ) {
        let line = eat_space(&self.text[self.line]).to_string();
        let tokens: Vec<&str> = line.split(|c| c == ' ' || c == '\t').collect();
        let first = eat_space(tokens[0]);
        match first {
            t if t.starts_with(";") => {
                return (self.parse_opcode(&tokens), None, None);
            }
            t if opcode::OPCODE_LIST.contains(&&(t.to_ascii_uppercase())[..]) => {
                return (self.parse_opcode(&tokens), None, None);
            }
            token::SECTION => {
                return (self.parse_text(), None, None);
            }
            token::EQU | token::SET => {
                return (self.parse_text(), None, None);
            }
            "REPT" | "rept" | "ENDR" | "endr" => {
                return (self.parse_text(), None, None);
            }
            token::INCLUDE | token::INCBIN => {
                return self.parse_include(&tokens);
            }
            t if opcode::DEFINE_LIST.contains(&&(t.to_ascii_uppercase())[..]) => {
                return (self.parse_text(), None, None);
            }
            "" => {
                return (self.parse_text(), None, None);
            }
            t if self.macros.contains(t) => {
                return (self.parse_text(), None, None);
            }
            _ => {
                if tokens.len() > 1 && eat_space(tokens[1]) == token::MACRO {
                    return (self.parse_macro(), None, None);
                } else if &self.text[self.line] != eat_start_space(&self.text[self.line]) {
                    return (self.parse_text(), None, None);
                } else {
                    return (self.parse_label(&tokens), None, None);
                }
            }
        }
    }

    /// ラベルのパース
    fn parse_label(&mut self, tokens: &Vec<&str>) -> bool {
        let first = tokens[0];
        let label = remove_colon(first);
        if first.starts_with(".") {
            // ローカルラベル

            let name = remove_dash(&self.global.clone()) + "/" + &label;
            let current = self.get_current_label();
            let current_node = self.nodes.get_mut(&current).unwrap();
            current_node.next = name.clone();
            self.nodes.insert(
                name.clone(),
                Node {
                    global: self.global.to_string(),
                    text: name.clone() + "\n",
                    elements: Default::default(),
                    next: "".to_string(),
                    next_cond: "".to_string(),
                    calls: Default::default(),
                },
            );
            self.local = label;
        } else {
            // グローバル
            let current = self.get_current_label();
            let current_node = self.nodes.get_mut(&current).unwrap();
            current_node.next = label.clone();
            self.nodes.insert(
                label.clone(),
                Node {
                    global: label.clone(),
                    text: label.clone() + "\n",
                    elements: Default::default(),
                    next: "".to_string(),
                    next_cond: "".to_string(),
                    calls: Default::default(),
                },
            );
            self.global = label.clone();
            self.local = "".to_string();
        }

        self.line += 1;
        let num_of_line = self.text.len();
        let end = self.line >= num_of_line;
        return end;
    }

    /// 命令のパース
    fn parse_opcode(&mut self, tokens: &Vec<&str>) -> bool {
        // TODO: JP, JR, CALLは特殊な処理が必要
        let text = self.text[self.line].to_string();
        let op = parse_opcode_syntax(text, tokens);

        match op {
            Element::Opcode {
                text,
                op,
                operand1,
                operand2,
                comment,
            } if op == "call".to_string() => {
                self.push_element(Element::Opcode {
                    text,
                    op,
                    operand1: operand1.clone(),
                    operand2: operand2.clone(),
                    comment,
                });
                let callee = operand1;
                let current_label = self.get_current_label();
                let current_node = self.nodes.get_mut(&current_label).unwrap();
                current_node.calls.push(callee);
            }
            Element::Opcode {
                text,
                op,
                operand1,
                operand2,
                comment,
            } if op == "jp".to_string() || op == "jr".to_string() || op == "jpba".to_string() => {
                self.push_element(Element::Opcode {
                    text,
                    op,
                    operand1: operand1.clone(),
                    operand2: operand2.clone(),
                    comment,
                });
                if operand1 == "c" || operand1 == "z" || operand1 == "nc" || operand1 == "nz" {
                    let mut label = operand2;
                    if !label.starts_with("@+$") {
                        if label.starts_with(".") {
                            label = remove_dash(&self.global.clone()) + "/" + &label;
                        }
                        let current_label = self.get_current_label();
                        let current_node = self.nodes.get_mut(&current_label).unwrap();
                        current_node.next = current_label.clone() + "'";
                        current_node.next_cond = label.clone();
                        self.nodes.insert(
                            current_label.clone() + "'",
                            Node {
                                global: self.global.to_string(),
                                text: label.clone() + "\n",
                                elements: Default::default(),
                                next: "".to_string(),
                                next_cond: "".to_string(),
                                calls: Default::default(),
                            },
                        );
                        if self.local != "" {
                            // ローカルスコープ内
                            self.local += "'";
                        } else {
                            self.global += "'";
                        }
                    }
                } else {
                    let mut label = operand1;
                    if !label.starts_with("@+$") {
                        if label.starts_with(".") {
                            label = remove_dash(&self.global.clone()) + "/" + &label;
                        }
                        let current_label = self.get_current_label();
                        let current_node = self.nodes.get_mut(&current_label).unwrap();
                        current_node.next = label.clone();
                        self.nodes.insert(
                            current_label.clone() + "'",
                            Node {
                                global: self.global.to_string(),
                                text: label.clone() + "\n",
                                elements: Default::default(),
                                next: "".to_string(),
                                next_cond: "".to_string(),
                                calls: Default::default(),
                            },
                        );
                        if self.local != "" {
                            // ローカルスコープ内
                            self.local += "'";
                        } else {
                            self.global += "'";
                        }
                    }
                }
            }
            Element::Opcode {
                text,
                op,
                operand1,
                operand2,
                comment,
            } => {
                self.push_element(Element::Opcode {
                    text,
                    op,
                    operand1,
                    operand2,
                    comment,
                });
            }
            _ => {}
        }

        self.line += 1;
        let num_of_line = self.text.len();
        let end = self.line >= num_of_line;
        return end;
    }

    fn parse_text(&mut self) -> bool {
        let text = self.text[self.line].to_string();
        self.push_element(Element::Text { text });

        self.line += 1;
        let num_of_line = self.text.len();
        let end = self.line >= num_of_line;
        return end;
    }

    fn parse_include(
        &mut self,
        tokens: &Vec<&str>,
    ) -> (
        bool,
        Option<HashMap<String, Nodes>>,
        Option<HashSet<String>>,
    ) {
        // includeに入れる
        let include = tokens[1]
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string();
        self.includes.push(include.clone());

        // includeファイルの絶対パスを取得
        let abs_include = Path::new(&self.base_dir)
            .join(include.clone())
            .to_str()
            .unwrap()
            .to_string();
        let mut p = Self::new(
            abs_include.clone(),
            self.base_dir.clone(),
            self.macros.clone(),
        );
        let (result, macros) = p.parse(include);

        // 解析結果を詰める
        let text = self.text[self.line].to_string();
        self.push_element(Element::Include {
            text: text,
            target: abs_include,
        });

        self.line += 1;
        let num_of_line = self.text.len();
        let end = self.line >= num_of_line;
        return (end, Some(result), Some(macros));
    }

    fn push_element(&mut self, element: Element) {
        let current = self.get_current_label();
        let node = self.nodes.get_mut(&current).unwrap();
        (*node).text += match &element {
            Element::Include { text, .. }
            | Element::Opcode { text, .. }
            | Element::Text { text, .. }
            | Element::Macro { text, .. } => &text,
        };
        (*node).text += "\n";
        (*node).elements.push(element);
    }

    fn get_current_label(&self) -> String {
        let current = match &self.local {
            local if local != "" => remove_dash(&self.global.clone()) + "/" + &local,
            _ => self.global.clone(),
        };
        return current;
    }

    fn parse_macro(&mut self) -> bool {
        let line = eat_space(&self.text[self.line]).to_string();
        let tokens: Vec<&str> = line.split(|c| c == ' ' || c == '\t').collect();
        let label = eat_space(tokens[0]).to_string();
        let mut texts = vec![line];

        loop {
            self.line += 1;
            let num_of_line = self.text.len();
            let end = self.line >= num_of_line;
            if end {
                return true;
            }

            let line = eat_space(&self.text[self.line]).to_string();
            texts.push(line.clone());

            let tokens: Vec<&str> = line.split(|c| c == ' ' || c == '\t').collect();
            let first = eat_space(tokens[0]).to_string();
            if first == token::ENDM {
                break;
            }
        }

        let text = texts.join("\n");
        self.macros.insert(label[0..(label.len() - 1)].to_string()); // labelは:を取り除く
        self.push_element(Element::Macro { label, texts, text });
        self.line += 1;
        let num_of_line = self.text.len();
        let end = self.line >= num_of_line;
        return end;
    }
}

pub fn get_node<'a>(nodes: &'a Nodes, label: &str) -> Option<&'a Node> {
    match nodes.get(label) {
        Some(node) => {
            return Some(node);
        }
        None => match nodes.get(&(label.to_string() + ":")) {
            Some(node) => {
                return Some(node);
            }
            None => match nodes.get(&(label.to_string() + "::")) {
                Some(node) => {
                    return Some(node);
                }
                None => {
                    return None;
                }
            },
        },
    }
}

fn eat_space(s: &str) -> &str {
    return eat_start_space(s).trim_end().trim_end_matches('\t');
}

fn eat_start_space(s: &str) -> &str {
    return s.trim_start().trim_start_matches('\t');
}

fn parse_opcode_syntax(text: String, tokens: &Vec<&str>) -> Element {
    let mut op = "".to_string();
    let mut operand1 = "".to_string();
    let mut operand2 = "".to_string();
    let mut comment = "".to_string();

    if tokens[0].starts_with(";") {
        // ;コメント
        comment = tokens.join(" ");
    } else {
        op = tokens[0].to_string();

        let mut comment_exist = false;
        if tokens.len() >= 2 {
            for (i, token) in tokens.iter().enumerate() {
                if token.starts_with(";") {
                    operand1 = eat_space(&tokens[1..i].join(" ")).to_string();
                    comment = tokens[i..].join(" ");
                    comment_exist = true;
                    break;
                }
            }

            if !comment_exist {
                operand1 = tokens[1..].join(" ");
            }
        }
    }

    if operand1 != "" {
        let tmp = operand1.clone();
        let operands: Vec<&str> = tmp.split(',').collect();
        if operands.len() == 2 {
            operand1 = eat_space(operands[0]).to_string();
            operand2 = eat_space(operands[1]).to_string();
        }
    }

    return Element::Opcode {
        text,
        op,
        operand1,
        operand2,
        comment,
    };
}

fn remove_colon(src: &str) -> String {
    return src.trim_end_matches(":").to_string();
}

fn remove_dash(src: &str) -> String {
    return src.trim_end_matches("'").to_string();
}
