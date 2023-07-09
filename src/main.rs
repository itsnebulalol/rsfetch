use std::collections::HashMap;
use std::env;
use std::io::Result;
use uname_rs::Uname;

pub mod common;

fn add_item(
    colors: &HashMap<&str, &str>,
    icon: &str,
    name: &str,
    content: &str,
    color: &str,
) -> String {
    if let Some(color_code) = colors.get(color) {
        if let Some(reset_code) = colors.get("reset") {
            return format!(
                "│ {}{} {}{:<9}│ {}{}{}",
                color_code, icon, reset_code, name, color_code, content, reset_code
            );
        }
    }
    String::new()
}

fn main() -> Result<()> {
    let uts = Uname::new()?;

    let mut colors: HashMap<&'static str, &'static str> = HashMap::new();
    colors.insert("black", "\x1b[30m");
    colors.insert("red", "\x1b[31m");
    colors.insert("green", "\x1b[32m");
    colors.insert("orange", "\x1b[33m");
    colors.insert("blue", "\x1b[34m");
    colors.insert("purple", "\x1b[35m");
    colors.insert("cyan", "\x1b[36m");
    colors.insert("lightgrey", "\x1b[37m");
    colors.insert("darkgrey", "\x1b[90m");
    colors.insert("lightred", "\x1b[91m");
    colors.insert("lightgreen", "\x1b[92m");
    colors.insert("yellow", "\x1b[93m");
    colors.insert("lightblue", "\x1b[94m");
    colors.insert("pink", "\x1b[95m");
    colors.insert("lightcyan", "\x1b[96m");

    colors.insert("reset", "\x1b[0m");
    colors.insert("bold", "\x1b[1m");
    colors.insert("disable", "\x1b[2m");
    colors.insert("underline", "\x1b[4m");
    colors.insert("reverse", "\x1b[7m");
    colors.insert("strikethrough", "\x1b[9m");
    colors.insert("invisible", "\x1b[8m");

    println!("╭────────────╮");
    println!(
        "{}",
        add_item(&colors, "", "user", &common::get_user(), "red")
    );
    println!(
        "{}",
        add_item(
            &colors,
            "",
            "model",
            &common::get_model(&uts.sysname),
            "yellow"
        )
    );
    println!(
        "{}",
        add_item(
            &colors,
            "",
            "os",
            &common::get_os_version(&uts.sysname),
            "green"
        )
    );
    println!(
        "{}",
        add_item(
            &colors,
            "",
            "cpu",
            &format!("{} ({})", common::get_cpu_info(), env::consts::ARCH),
            "cyan"
        )
    );
    println!(
        "{}",
        add_item(&colors, "", "gpu", &common::get_gpu_info(), "blue")
    );
    println!(
        "{}",
        add_item(&colors, "", "packages", &common::get_packages(), "purple")
    );
    println!(
        "{}",
        add_item(&colors, "", "shell", &common::get_shell(), "lightred")
    );
    println!(
        "{}",
        add_item(
            &colors,
            "",
            "memory",
            &common::get_memory_usage(),
            "yellow"
        )
    );
    println!(
        "{}",
        add_item(&colors, "", "uptime", &common::get_uptime(), "lightgreen")
    );
    println!("├────────────┤");
    println!(
        "{}",
        add_item(
            &colors,
            "",
            "colors",
            &format!(
                "{}● {}● {}● {}● {}● {}● {}● {}●",
                colors.get("black").unwrap_or(&""),
                colors.get("red").unwrap_or(&""),
                colors.get("yellow").unwrap_or(&""),
                colors.get("green").unwrap_or(&""),
                colors.get("cyan").unwrap_or(&""),
                colors.get("blue").unwrap_or(&""),
                colors.get("purple").unwrap_or(&""),
                colors.get("reset").unwrap_or(&""),
            ),
            "reset"
        )
    );
    println!("╰────────────╯");

    Ok(())
}

/*fn main() {
println!("╭────────────╮");
println!("{}", common::get_os_version("Darwin"));
println!("{}", common::get_model("Darwin"));
println!("{}", common::get_packages());
println!("{}", common::get_uptime());
println!("{}", common::get_gpu_info());
println!("{}", common::get_memory_usage());
println!("{}", common::get_shell());
}*/
