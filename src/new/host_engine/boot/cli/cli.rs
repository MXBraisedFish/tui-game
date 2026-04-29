pub fn handle_command() -> Result<bool> {
    // 读取指令
    let arg = std::env::args().nth(1);

    if arg.is_none() {
        return Ok(false);
    }

    // 准备语言
    language::load();

    // 解析指令
    let command = arg.unwrap().to_ascii_lowercase();

    match command.as_str() {
        "-h" | "-help"          => function::help::execute(),
        "-v" | "-version"       => function::version::execute(),
        "-cc" | "-clear-cache"  => function::clear_cache::execute(),
        "-cd" | "-clear-data"   => function::clear_data::execute(),
        "-p" | "-path"          => function::path::execute(),
        _ => {
            eprintln!("{}", language::ERROR_UNKNOWN);
            return Ok(false);
        }
    }

    Ok(true)
}