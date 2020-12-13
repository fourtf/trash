static BUILTINS: phf::Map<&'static str, fn(&Vec<String>)> = phf::phf_map! {
    "exit" => exit,
    "cd" => cd,
    // "clear" => clear,
};

pub fn has_builtin(args: &Vec<String>) -> bool {
    match args.get(0) {
        Some(cmd) => has_builtin_cmd(&cmd),
        None => false,
    }
}

pub fn has_builtin_cmd(cmd: &String) -> bool {
    BUILTINS.contains_key(&cmd[..])
}

pub fn exec_builtin(args: &Vec<String>) -> bool {
    let cmd = args.get(0).map(|x| &x[..]).unwrap_or("");

    if BUILTINS.contains_key(cmd) {
        (BUILTINS[cmd])(args);

        true
    } else {
        false
    }
}

fn exit(_args: &Vec<String>) {
    std::process::exit(0);
}

fn cd(args: &Vec<String>) {
    if let Some(to) = args.get(0) {
        std::env::set_current_dir(to).ok();
    } else if let Some(to) = dirs::home_dir() {
        std::env::set_current_dir(to).ok();
    }
}
