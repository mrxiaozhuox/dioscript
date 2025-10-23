use nom::{
    error::{VerboseError, VerboseErrorKind},
    Offset,
};

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{text}")]
    ParseFailure { text: String },
    #[error("unmatch content: `{content}`")]
    UnMatchContent { content: String },
}

pub fn simplify_error<'a>(input: &'a str, e: VerboseError<&'a str>) -> String {
    let first_error = e
        .errors
        .iter()
        .find(|(_, kind)| {
            matches!(
                kind,
                VerboseErrorKind::Char(_) | VerboseErrorKind::Context(_)
            )
        })
        .or_else(|| e.errors.first());

    if let Some((input_slice, error_kind)) = first_error {
        // 计算错误行号
        let offset = input.offset(input_slice);
        let line_number = input[..offset].chars().filter(|&c| c == '\n').count() + 1;

        // 获取错误消息
        let error_message = match error_kind {
            VerboseErrorKind::Context(ctx) => ctx.to_string(),
            VerboseErrorKind::Char(c) => format!("unexpected character '{}'", c),
            _ => "syntax error".to_string(),
        };

        let lines: Vec<&str> = input.lines().collect();
        let total_lines = lines.len();

        let start_line = if line_number > 2 { line_number - 2 } else { 1 };
        let end_line = std::cmp::min(line_number + 2, total_lines);

        let mut context = String::new();
        for i in start_line..=end_line {
            let line_idx = i - 1;
            let new_line = format!("[{i}] | {}\n", lines[line_idx]);
            context.push_str(&new_line);
            if i == line_number {
                let line_start_offset = input[..offset].rfind('\n').map_or(0, |pos| pos + 1);
                let column = offset - line_start_offset;

                // 添加指示符
                let mut indicator = String::from("      ");
                indicator.push_str(&" ".repeat(column));
                indicator.push('^');
                context.push_str(&indicator);
                context.push('\n');
            }
        }

        format!(
            "Parse Error At Line {}: {}\n\n{}",
            line_number, error_message, context
        )
    } else {
        "Unknown Parsing Error".to_string()
    }
}
