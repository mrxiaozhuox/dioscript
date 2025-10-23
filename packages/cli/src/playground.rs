use colored::Colorize;
use dioscript_parser::ast::DioscriptAst;
use dioscript_runtime::{Executor, OutputHandler, Value};
use rustyline::{error::ReadlineError, DefaultEditor};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::Instant,
};

use crate::print_value_result;

pub struct PlaygroundOutputHandler;

impl OutputHandler for PlaygroundOutputHandler {
    fn print(&mut self, content: Value) {
        print_value_result(&content);
    }
}

pub fn playground_main() {
    // Initial print
    print_welcome_message();

    // Initial state
    let mut runtime = Executor::init();

    runtime.with_output_handler(Box::new(PlaygroundOutputHandler));

    let mut editor = setup_editor();
    let mut multiline_mode = false;
    let mut current_input: Vec<String> = Vec::new();
    let mut code_buffer: Vec<String> = Vec::new();

    loop {
        // Dynamic prompt, similar to Python's >>> and ...
        let prompt = if multiline_mode { "... " } else { ">>> " };

        // Read input
        match editor.readline(prompt) {
            Ok(input) => {
                let trimmed_input = input.trim();

                // Handle special commands
                if trimmed_input.starts_with('.') {
                    if !process_command(trimmed_input, &mut code_buffer, &mut runtime, &mut editor)
                    {
                        break;
                    }
                    multiline_mode = false;
                    current_input.clear();
                    continue;
                }

                editor.add_history_entry(&input).ok();

                // Handle multiline input
                if multiline_mode {
                    // Empty line ends multiline input
                    if trimmed_input.is_empty() {
                        multiline_mode = false;

                        // Add current input to code buffer before execution
                        for line in &current_input {
                            code_buffer.push(line.clone());
                        }

                        execute_code(&current_input, &mut runtime);
                        current_input.clear();
                    } else {
                        current_input.push(input);
                    }
                } else {
                    // Check if need to enter multiline mode
                    if trimmed_input.ends_with(':') || trimmed_input.ends_with('{') {
                        multiline_mode = true;
                        current_input.push(input);
                    } else {
                        // Single line mode, execute immediately
                        current_input.push(input.clone());

                        if execute_code(&current_input, &mut runtime) {
                            // Add to code buffer after execution
                            code_buffer.push(input);
                        }

                        current_input.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C handling
                if multiline_mode {
                    // In multiline mode, Ctrl-C cancels current input
                    println!("\n{}", "[ds] Multiline input cancelled.".yellow());
                    multiline_mode = false;
                    current_input.clear();
                } else {
                    // In single line mode, Ctrl-C prompts to press again to exit
                    if let Err(ReadlineError::Interrupted) =
                        editor.readline(&format!("{}", "[ds] Press Ctrl-C Again to exit.".yellow()))
                    {
                        println!("\n👋 {}\n", "Bye!".green().bold());
                        break;
                    }
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D handling
                println!("\n👋 {}\n", "Bye!".green().bold());
                break;
            }
            Err(err) => {
                eprintln!("\n{} {}\n", "[ds] Error: ".red().bold(), err);
                break;
            }
        }
    }

    // Save history
    save_history(&mut editor);
}

// Process commands
// When returning `false`, the application should exit
fn process_command(
    input: &str,
    code_buffer: &mut Vec<String>,
    runtime: &mut Executor,
    editor: &mut DefaultEditor,
) -> bool {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts[0];

    match cmd {
        ".debug" => debug(runtime, parts.get(1)),
        ".clear" | ".c" => clear_buffer(code_buffer),
        ".save" | ".s" => save_code(code_buffer, parts.get(1)),
        ".load" | ".l" => load_code(code_buffer, parts.get(1), runtime, editor),
        ".show" | ".sh" => show_buffer(code_buffer),
        ".edit" | ".e" => edit_line(code_buffer, parts.get(1), editor),
        ".reset" | ".rs" => reset_environment(code_buffer, runtime, editor),
        ".help" | ".h" => show_help(),
        ".quit" | ".q" => return confirm_quit(code_buffer, editor),
        _ => {
            println!("\n{} Unknown Command: {}\n", "[ds]".red(), cmd);
            println!("Use {} to find available commands\n", ".help".green());
        }
    }

    true // Continue working
}

/// Welcome message
fn print_welcome_message() {
    println!("\n{}", "Dioscript Interactive Shell".blue().bold());
    println!(
        "{}",
        "Type code to execute immediately, or end with ':' for multi-line input".green()
    );
    println!("{}\n", "Type .help to find all commands".green());
}

// Setup readline editor
fn setup_editor() -> DefaultEditor {
    let mut editor = DefaultEditor::new().expect("Cannot initialize terminal editor");

    // Load history
    if let Some(history_path) = get_history_path() {
        let _ = editor.load_history(&history_path);
    }

    editor
}

/// Get history path
fn get_history_path() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(|home| format!("{}/.dioscript_history", home))
}

/// Save history
fn save_history(editor: &mut DefaultEditor) {
    if let Some(history_path) = get_history_path() {
        if let Err(err) = editor.save_history(&history_path) {
            eprintln!("Unable to save history: {}", err);
        }
    }
}

/// Execute code
fn execute_code(code_input: &[String], runtime: &mut Executor) -> bool {
    if code_input.is_empty() {
        return false;
    }

    let code = code_input.join("\n");

    // Parse
    match DioscriptAst::from_string(&code) {
        Ok(ast) => {
            // Execute and start runtime
            let start_time = Instant::now();
            match runtime.execute(ast) {
                Ok(result) => {
                    let elapsed = start_time.elapsed();

                    // Result - similar to Python's direct expression result display
                    if !result.as_none() {
                        print_value_result(&result);
                    }

                    // Only show timing for multiline or complex executions
                    if code_input.len() > 1 {
                        println!("{}: {:.2?}", "[ds] Execute Time".bright_black(), elapsed);
                    }
                    true
                }
                Err(e) => {
                    println!("{} {}", "[ds] Error:".red().bold(), e);
                    false
                }
            }
        }
        Err(e) => {
            println!("{} {}", "[ds] Syntax Error:".red().bold(), e);
            false
        }
    }
}

/// Clear buffer
fn clear_buffer(code_buffer: &mut Vec<String>) {
    if !code_buffer.is_empty() {
        code_buffer.clear();
        println!("{} Buffer cleared", "[ds]".green());
    } else {
        println!("{} Buffer is already empty", "[ds]".yellow());
    }
}

/// Save code
fn save_code(code_buffer: &[String], path_arg: Option<&&str>) {
    if code_buffer.is_empty() {
        println!("{} Buffer is empty", "[ds]".yellow());
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
                    "{} {} {}",
                    "[ds]".green(),
                    "Code saved to:".green(),
                    path.display()
                );
            }
            Err(e) => {
                println!("{} {}", "[ds] Save failed: ".red(), e);
            }
        },
        Err(e) => {
            println!("{} {}", "[ds] Create file failed: ".red(), e);
        }
    }
}

/// Load code
fn load_code(
    code_buffer: &mut Vec<String>,
    path_arg: Option<&&str>,
    runtime: &mut Executor,
    editor: &mut DefaultEditor,
) {
    let path = path_arg
        .map(|&p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from("./playground.ds"));

    // Code buffer is not empty
    if !code_buffer.is_empty() {
        println!(
            "{}",
            "[ds] Buffer is not empty, loading will overwrite existing code.".yellow()
        );
        match editor.readline("Continue? (y/N) ") {
            Ok(input) if input.trim().to_lowercase() == "y" => {}
            _ => {
                println!("{} Canceled", "[ds]".yellow());
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
                        "{} {} {} ({} lines)",
                        "[ds]".green(),
                        "Loaded from:".green(),
                        path.display(),
                        code_buffer.len()
                    );

                    let ok = execute_code(code_buffer, runtime);
                    if ok {
                        println!("{}", "[ds] Code executed.".bright_black());
                    }
                }
                Err(e) => {
                    println!("{} Read file failed: {}", "[ds]".red(), e);
                }
            }
        }
        Err(e) => {
            println!("{} Open file failed: {}", "[ds]".red(), e);
        }
    }
}

/// Show current buffer contents
fn show_buffer(code_buffer: &[String]) {
    if code_buffer.is_empty() {
        println!("{} Buffer is empty", "[ds]".yellow());
        return;
    }

    println!("{} {}", "[ds]".cyan(), "Current Buffer:".green().bold());

    for (i, line) in code_buffer.iter().enumerate() {
        println!("{:4}: {}", i + 1, line);
    }

    println!(
        "{} {} {} {}",
        "[ds]".cyan(),
        "(".bright_black(),
        code_buffer.len().to_string().yellow(),
        "lines )".bright_black()
    );
}

/// Edit a specific line
fn edit_line(code_buffer: &mut [String], line_arg: Option<&&str>, editor: &mut DefaultEditor) {
    if code_buffer.is_empty() {
        println!("{} Buffer is empty", "[ds]".yellow());
        return;
    }

    let line_num = match line_arg {
        Some(num_str) => match num_str.parse::<usize>() {
            Ok(num) if num > 0 && num <= code_buffer.len() => num,
            _ => {
                println!("{} Invalid line number", "[ds]".red());
                return;
            }
        },
        None => {
            println!("{} Specify line number (.edit <line>)", "[ds]".yellow());
            return;
        }
    };

    let idx = line_num - 1;
    println!("Line {} : {}", line_num, code_buffer[idx]);

    match editor.readline("edit>>> ") {
        Ok(new_line) => {
            code_buffer[idx] = new_line;
            println!("{} Line updated", "[ds]".green());
        }
        Err(_) => {
            println!("{} Canceled", "[ds]".yellow());
        }
    }
}

fn debug(runtime: &mut Executor, info: Option<&&str>) {
    match info.copied() {
        Some("data") => {
            println!("{:#?}", runtime.debug_data_info());
        }
        None | Some("scopes") => {
            println!("{:#?}", runtime.debug_scopes_info());
        }
        _ => {}
    }
}

/// Reset environment
fn reset_environment(
    code_buffer: &mut Vec<String>,
    runtime: &mut Executor,
    editor: &mut DefaultEditor,
) {
    if !code_buffer.is_empty() {
        println!(
            "{}",
            "[ds] This will clear the buffer and reset the runtime.".yellow()
        );
        match editor.readline("Continue? (y/N) ") {
            Ok(input) if input.trim().to_lowercase() == "y" => {}
            _ => {
                println!("{} Canceled", "[ds]".yellow());
                return;
            }
        }
    }

    // re-init runtime
    *runtime = Executor::init();
    runtime.with_output_handler(Box::new(PlaygroundOutputHandler));

    code_buffer.clear();
    println!("{} Environment reset", "[ds]".green());
}

/// Show help information
fn show_help() {
    println!("{} Available Commands:", "[ds]".cyan());

    println!("  {:<12} - Clear the code buffer", ".clear, .c");
    println!("  {:<12} - Save code to file (.save [path])", ".save, .s");
    println!("  {:<12} - Load code from file (.load [path])", ".load, .l");
    println!("  {:<12} - Show current code buffer", ".show, .sh");
    println!(
        "  {:<12} - Edit specific line (.edit <line_number>)",
        ".edit, .e"
    );
    println!("  {:<12} - Reset runtime environment", ".reset, .rs");
    println!("  {:<12} - Display this help information", ".help, .h");
    println!("  {:<12} - Exit shell", ".quit, .q");

    println!();

    println!("{}", "Python-like REPL mode:".cyan());
    println!("  - Type code to execute immediately");
    println!("  - End a line with ':' or '{{' to start multi-line input");
    println!("  - Press Enter on an empty line to execute multi-line code");
    println!("  - Press Ctrl+C to cancel multi-line input");
    println!();
}

/// Confirm quit with unsaved changes
fn confirm_quit(code_buffer: &[String], editor: &mut DefaultEditor) -> bool {
    if !code_buffer.is_empty() {
        println!("{} There is unsaved code in the buffer", "[ds]".yellow());
        match editor.readline("Quit anyway? (y/N) ") {
            Ok(input) if input.trim().to_lowercase() == "y" => {}
            _ => {
                println!("{} Canceled", "[ds]".yellow());
                return true;
            }
        }
    }

    println!("\n👋 {}\n", "Bye!".green().bold());
    false
}

/// Auto-save buffer contents (unused but kept for reference)
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
