use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufReader};
use std::process::Command;
use std::str;

use ansi_rgb::Foreground;
use css_color_parser::Color;
use regex;
use rgb::RGB8;
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::EventReader;

fn main() {
    let url = get_url();
    let langs = select_cc(&url);
    let cc_files = langs
        .iter()
        .map(|lang| download_cc(&url, lang))
        .collect::<Vec<_>>();
    for cc_file in cc_files {
        let paths = fs::read_dir("./").unwrap();
        let path = &paths
            .filter_map(|path| path.ok())
            .map(|path| {
                path.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|path| path.contains(&cc_file))
            .collect::<Vec<_>>()[0];
        read_ttml_cc(path.to_string());
    }
}

fn get_url() -> String {
    println!("Youtube video url...");
    print!(">> ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read from stdin.");
    let buf = buf.trim();
    let url = String::from(buf);
    url
}

fn select_cc(url: &String) -> Vec<String> {
    let output = power_shell!()
        .arg("./youtube-dl.exe")
        .arg("--list-subs")
        .arg(url)
        .output()
        .expect("Failed to execute command.")
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    println!("{}", output);

    println!("Select language...");
    print!(">> ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read from stdin.");
    let buf = buf.trim();
    let languages = buf
        .split(char::is_whitespace)
        .map(|lang| String::from(lang))
        .collect::<Vec<String>>();
    languages
}

fn download_cc(url: &String, lang: &String) -> String {
    let output = power_shell!()
        .arg("./youtube-dl.exe")
        .args(&["--sub-lang", &lang])
        .arg("--write-sub")
        .arg("--skip-download")
        .args(&["--sub-format", "ttml"])
        .arg(url)
        .output()
        .expect("Failed to execute command")
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    let re = regex::Regex::new(r"(?P<cc>\w{11}\..+\.ttml)").unwrap();
    let cap = &re.captures_iter(output).next().unwrap()["cc"];
    String::from(cap)
}

fn read_ttml_cc(file_path: String) {
    let file = File::open(file_path).expect("Failed to open file.");
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let mut color_set = HashMap::new();

    let mut fg = RGB8::new(255, 255, 255);
    let white = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 1.0,
    };
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => match name.local_name.as_str() {
                "style" => {
                    let mut id = String::new();
                    let mut color = String::new();
                    for attribute in attributes {
                        let OwnedName {
                            local_name,
                            namespace: _,
                            prefix: _,
                        } = attribute.name;
                        match local_name.as_str() {
                            "xml:id" => id = attribute.value,
                            "tts:color" => color = attribute.value,
                            _ => {},
                        }
                    }
                    *color_set.entry(id).or_default() = color;
                }
                "span" => {
                    if let Some(color) = color_set.get(&attributes[0].value) {
                        // --- dead code
                        eprintln!("{}", color);
                        let Color { r, g, b, a: _ } = &color.parse::<Color>().unwrap_or(white);
                        fg = RGB8::new(*r, *g, *b);
                        // ---
                    }
                }
                "p" => {
                    for attribute in attributes {
                        let OwnedName {
                            local_name,
                            namespace: _,
                            prefix: _,
                        } = attribute.name;
                        match local_name.as_str() {
                            "begin" => print!("{}\t", attribute.value),
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Ok(XmlEvent::Characters(data)) => {
                println!("{}", data.fg(fg));
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

#[macro_export]
macro_rules! power_shell {
    () => {
        Command::new("powershell").arg("-c")
    };
}
