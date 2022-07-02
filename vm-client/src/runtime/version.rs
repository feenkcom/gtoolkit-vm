use colored::Colorize;
use comfy_table::Table;
use vm_bindings::virtual_machine_info;

pub fn app_info() -> &'static str {
    include_str!(env!("APP_BUILD_INFO"))
}

pub fn fetch_version() -> String {
    let mut vm_info = json::parse(virtual_machine_info()).unwrap();
    let mut config = vm_info.remove("config");
    let mut config_data = config.remove("data");
    let built_from = config_data.remove("BUILT_FROM");

    let mut components = vec![];

    let mut app_info_json = json::parse(app_info()).unwrap();
    let app_name = app_info_json.remove("app_name");

    let mut builder_flags = app_info_json.remove("builder_flags");
    let author = builder_flags.remove("author");
    let mut app_build_info = app_info_json.remove("app_build_info");
    let app_build_hash = app_build_info.remove("git_sha");
    let app_build_timestamp = app_build_info.remove("build_timestamp");

    components.push(vec![
        "App".to_string(),
        format!("Commit: {}", app_build_hash.as_str().unwrap_or("unknown")),
    ]);

    let mut app_builder_info = app_info_json.remove("builder_info");
    let app_builder_hash = app_builder_info.remove("git_sha");

    components.push(vec![
        "App Builder".to_string(),
        format!("Commit: {}", app_builder_hash.as_str().unwrap_or("unknown")),
    ]);

    components.push(vec![
        "Pharo VM".to_string(),
        built_from.as_str().unwrap_or("unknown").to_string(),
    ]);

    let mut components_table = Table::new();
    components_table.set_header(vec!["Component", "Version"]);

    for component in components {
        components_table.add_row(component);
    }

    let app_name_str = app_name.as_str().unwrap_or("unknown");
    let author_str = author.as_str().unwrap_or("unknown");
    let app_build_timestamp_str = app_build_timestamp.as_str().unwrap_or("unknown");

    let intro = format!(
        "{} by {} built on {}",
        app_name_str, author_str, app_build_timestamp_str
    )
    .green()
    .bold();

    format!(
        "{}\n\n{components_table}",
        intro
    )
}

pub fn print_version() {
    println!("{}", fetch_version())
}
