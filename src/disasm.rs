use std::process::Command;

pub fn disassemble(gb_file_path: &str) {
    let output = if cfg!(target_os = "windows") {
        let cmd = format!("python ./mgbdis/mgbdis.py {}", gb_file_path);
        // println!("{}", &cmd);
        Command::new("cmd")
            .args(&["/C", &cmd])
            .output()
            .expect("failed to execute process")
    } else {
        let cmd = format!("python3 ./mgbdis/mgbdis.py {}", gb_file_path);
        // println!("{}", &cmd);
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    };
    let stdout = String::from_utf8(output.stdout).unwrap();
    if stdout.len() > 0 {
        println!("{}", stdout);
    }
    let stderr = String::from_utf8(output.stderr).unwrap();
    if stderr.len() > 0 {
        println!("{}", stderr);
    }
}
