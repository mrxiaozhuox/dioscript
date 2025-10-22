use colored::Colorize;
use dioscript_parser::ast::DioscriptAst;
use dioscript_runtime::{types::Value, Runtime};
use rustyline::{error::ReadlineError, DefaultEditor};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::Instant,
};

pub fn playground_main() {
    // inital print
    print_welcome_message();

    // inital status
    let mut code_buffer: Vec<String> = Vec::new();
    let mut runtime = Runtime::new();
    let mut editor = setup_editor();

    loop {
        // dynamic prompt
        let prompt = if code_buffer.is_empty() { ">> " } else { ".. " };

        // read input
        match editor.readline(prompt) {
            Ok(input) => {
                editor.add_history_entry(&input).ok();

                // check its
                if input.starts_with('.') {
                    if !process_command(&input, &mut code_buffer, &mut runtime, &mut editor) {
                        break;
                    }
                } else {
                    code_buffer.push(input);
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C handle
                if let Err(ReadlineError::Interrupted) =
                    editor.readline(&format!("{}", "[ds] Press Ctrl-C Again to exit.".yellow()))
                {
                    println!("\n👋 {}\n", "Bye!".green().bold());
                    break;
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D 处理
                println!("\n👋 {}\n", "Bye!".green().bold());
                break;
            }
            Err(err) => {
                eprintln!("\n{} {}\n", "[ds] Error: ".red().bold(), err);
                break;
            }
        }
    }

    // save history
    save_history(&mut editor);
}

// handle command
// when return `false`, app should exit
fn process_command(
    input: &str,
    code_buffer: &mut Vec<String>,
    runtime: &mut Runtime,
    editor: &mut DefaultEditor,
) -> bool {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts[0];

    match cmd {
        ".run" | ".r" => execute_code(code_buffer, runtime),
        ".undo" | ".u" => undo_last_line(code_buffer),
        ".clear" | ".c" => clear_buffer(code_buffer),
        ".trace" | ".t" => trace_runtime(runtime),
        ".save" | ".s" => save_code(code_buffer, parts.get(1)),
        ".load" | ".l" => load_code(code_buffer, parts.get(1), editor),
        ".show" | ".sh" => show_buffer(code_buffer),
        ".edit" | ".e" => edit_line(code_buffer, parts.get(1), editor),
        ".reset" | ".rs" => reset_environment(code_buffer, runtime, editor),
        ".help" | ".h" => show_help(),
        ".quit" | ".q" => return confirm_quit(code_buffer, editor),
        _ => {
            println!("\n{} Unknown Command: {}\n", "[ds]".red(), cmd);
            println!("Use {} to find available commands\n", ":help".green());
        }
    }

    true // keep work
}

/// welcome message
fn print_welcome_message() {
    println!("\n{}", "Dioscript Playground".blue().bold());
    println!(
        "{}",
        "Input code and use :execute or :r to exwecute".green()
    );
    println!("{}\n", "Input :help to find all commands".green());
}

// setup readline editor
fn setup_editor() -> DefaultEditor {
    let mut editor = DefaultEditor::new().expect("Cannot inital terminal editor");

    // load history
    if let Some(history_path) = get_history_path() {
        let _ = editor.load_history(&history_path);
    }

    editor
}

/// get history path
fn get_history_path() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(|home| format!("{}/.dioscript_history", home))
}

/// save history
fn save_history(editor: &mut DefaultEditor) {
    if let Some(history_path) = get_history_path() {
        if let Err(err) = editor.save_history(&history_path) {
            eprintln!("Unable to save history: {}", err);
        }
    }
}

/// execute
fn execute_code(code_buffer: &[String], runtime: &mut Runtime) {
    if code_buffer.is_empty() {
        println!("\n{} No code to execute\n", "[ds]".yellow());
        return;
    }

    let code = code_buffer.join("\n");

    // parser
    match DioscriptAst::from_string(&code) {
        Ok(ast) => {
            // execute and start runtime
            let start_time = Instant::now();
            match runtime.execute_ast(ast) {
                Ok(result) => {
                    let elapsed = start_time.elapsed();

                    // result
                    if !result.as_none() {
                        println!("\n{} Result:", "[ds]".green());
                        print_value_result(&result);
                    } else {
                        println!("\n{} {}", "[ds]".green(), "Successful!".green().bold());
                    }

                    println!("{}: {:.2?}\n", "[ds] Execute Timer".green(), elapsed);
                }
                Err(e) => {
                    println!("\n{} {}\n", "[ds] Runtime Error:".red().bold(), e);
                }
            }
        }
        Err(e) => {
            println!("\n{} {}\n", "[ds] Parser Error:".red().bold(), e);
        }
    }
}

/// make value pretty
fn print_value_result(value: &Value) {
    match value {
        Value::String(s) => println!("{:?}", s),
        Value::Number(n) => println!("{}", n),
        Value::Boolean(b) => println!("{}", b),
        Value::List(items) => {
            println!("[");
            for item in items {
                print!("  ");
                print_value_result(item);
            }
            println!("]");
        }
        Value::Dict(map) => {
            println!("{{");
            for (k, v) in map {
                print!("  {}: ", k);
                print_value_result(v);
            }
            println!("}}");
        }
        _ => println!("{:#?}", value),
    }
}

/// Undo last line
fn undo_last_line(code_buffer: &mut Vec<String>) {
    if let Some(line) = code_buffer.pop() {
        println!("\n{} Undo: {}\n", "[ds]".yellow(), line);
    } else {
        println!("\n{} Not code to undo\n", "[ds]".yellow());
    }
}

/// clean buffer
fn clear_buffer(code_buffer: &mut Vec<String>) {
    if !code_buffer.is_empty() {
        code_buffer.clear();
        println!("\n{} The code buffer has been cleared.\n", "[ds]".green());
    } else {
        println!("\n{} The code buffer is already empty.\n", "[ds]".yellow());
    }
}

/// runtime trace
fn trace_runtime(runtime: &mut Runtime) {
    println!("\n{} Runtime Trace:", "[ds]".cyan());
    runtime.trace();
    println!();
}

/// save code
fn save_code(code_buffer: &[String], path_arg: Option<&&str>) {
    if code_buffer.is_empty() {
        println!("\n{} CodeBuffer is empty\n", "[ds]".yellow());
        return;
    }

    let code = code_buffer.join("\n");
    let path = path_arg
        .map(|&p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from("./playground.ds"));

    match File::create(&path) {
        Ok(mut file) => match file.write_all(code.as_bytes()) {
            Ok(_) => {
                println!(
                    "\n{} {} {}\n",
                    "[ds]".green(),
                    "Code save to:".green(),
                    path.display()
                );
            }
            Err(e) => {
                println!("\n{} {}\n", "[ds] CodeBuffer save failed: ".red(), e);
            }
        },
        Err(e) => {
            println!("\n{} {}\n", "[ds] Create file failed: ".red(), e);
        }
    }
}

/// load code
fn load_code(code_buffer: &mut Vec<String>, path_arg: Option<&&str>, editor: &mut DefaultEditor) {
    let path = path_arg
        .map(|&p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from("./playground.ds"));

    // code buffer is not empty
    if !code_buffer.is_empty() {
        println!(
            "\n{}",
            "[ds] The current buffer is not empty, loading will overwrite the existing code."
                .yellow()
        );
        match editor.readline("Sure? (y/N) ") {
            Ok(input) if input.trim().to_lowercase() == "y" => {}
            _ => {
                println!("\n{} Canceled\n", "[ds]".yellow());
                return;
            }
        }
    }

    match File::open(&path) {
        Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => {
                    code_buffer.clear();
                    *code_buffer = content.lines().map(String::from).collect();
                    println!(
                        "\n{} {} {} ({} line)\n",
                        "[ds]".green(),
                        "Loaded from:".green(),
                        path.display(),
                        code_buffer.len()
                    );
                }
                Err(e) => {
                    println!("\n{} Read file failed: {}\n", "[ds]".red(), e);
                }
            }
        }
        Err(e) => {
            println!("\n{} Open  file failed: {}\n", "[ds]".red(), e);
        }
    }
}

fn show_buffer(code_buffer: &[String]) {
    if code_buffer.is_empty() {
        println!("\n{}\n", "[ds] CodeBuffer is empty".yellow());
        return;
    }

    println!(
        "\n{} {}{}",
        "[ds]".cyan().bold(),
        "#".yellow().italic(),
        "CodeBuffer".green().bold(),
    );
    println!();

    for (i, line) in code_buffer.iter().enumerate() {
        println!("{:4}: {}", i + 1, line);
    }

    println!(
        "\n{} Total {} {} lines {}",
        "[ds]".cyan().bold(),
        "(".bright_black(),
        code_buffer.len().to_string().yellow(),
        ")".bright_black()
    );
    println!();
}

fn edit_line(code_buffer: &mut [String], line_arg: Option<&&str>, editor: &mut DefaultEditor) {
    if code_buffer.is_empty() {
        println!("\n{} CodeBuffer is empty\n", "[ds]".yellow());
        return;
    }

    let line_num = match line_arg {
        Some(num_str) => match num_str.parse::<usize>() {
            Ok(num) if num > 0 && num <= code_buffer.len() => num,
            _ => {
                println!("\n{} Invalid line number\n", "[ds]".red());
                return;
            }
        },
        None => {
            println!("\n{} Specify line number (.edit <line>)\n", "[ds]".yellow());
            return;
        }
    };

    let idx = line_num - 1;
    println!("Edit line {} : {}", line_num, code_buffer[idx]);

    match editor.readline("edit>> ") {
        Ok(new_line) => {
            code_buffer[idx] = new_line;
            println!("\nLine {} updated\n", "[ds]".green());
        }
        Err(_) => {
            println!("\n{} Canceled\n", "[ds]".yellow());
        }
    }
}

// reset
fn reset_environment(
    code_buffer: &mut Vec<String>,
    runtime: &mut Runtime,
    editor: &mut DefaultEditor,
) {
    if !code_buffer.is_empty() {
        println!(
            "\n{} This will clear the CodeBuffer and reset the runtime.",
            "[ds]".yellow()
        );
        match editor.readline("Sure? (y/N) ") {
            Ok(input) if input.trim().to_lowercase() == "y" => {}
            _ => {
                println!("\n{} Canceled\n", "[ds]".yellow());
                return;
            }
        }
    }

    *runtime = Runtime::new();
    code_buffer.clear();
    println!("\n{} Successful\n", "[ds]".green());
}

fn show_help() {
    println!("\n{} Available Commands:", "[ds]".cyan());

    println!("  {:<12} - Execute code in the buffer", ".run, .r");
    println!("  {:<12} - Undo the last line of code", ".undo, .u");
    println!("  {:<12} - Clear the code buffer", ".clear, .c");
    println!("  {:<12} - Display runtime trace", ".trace, .t");
    println!("  {:<12} - Save code to file (.save [path])", ".save, .s");
    println!("  {:<12} - Load code from file (.load [path])", ".load, .l");
    println!("  {:<12} - Show current code buffer", ".show, .sh");
    println!(
        "  {:<12} - Edit specific line (.edit <line_number>)",
        ".edit, .e"
    );
    println!("  {:<12} - Reset runtime environment", ".reset, .rs");
    println!("  {:<12} - Display this help information", ".help, .h");
    println!("  {:<12} - Exit Playground", ".quit, .q");
    println!();
}

fn confirm_quit(code_buffer: &[String], editor: &mut DefaultEditor) -> bool {
    if !code_buffer.is_empty() {
        println!("\n{} There is unsaved code in the buffer", "[ds]".yellow());
        match editor.readline("Sure? (y/N) ") {
            Ok(input) if input.trim().to_lowercase() == "y" => {}
            _ => {
                println!("\n{} Canceled\n", "[ds]".yellow());
                return true;
            }
        }
    }

    println!("\n👋 {}\n", "Bye!".green().bold());
    false
}

#[allow(dead_code)]
fn auto_save(code_buffer: &[String]) {
    if code_buffer.is_empty() {
        return;
    }

    let auto_save_path = PathBuf::from("./.dioscript_autosave.ds");
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(auto_save_path)
    {
        let _ = file.write_all(code_buffer.join("\n").as_bytes());
    }
}
