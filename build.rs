fn main() {
    let style = std::env::var("SLINT_STYLE").unwrap_or("fluent".into());
    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new().with_style(style),
    )
    .unwrap();
}
