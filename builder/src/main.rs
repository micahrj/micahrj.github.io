use std::collections::HashMap;
use std::error::Error;
use std::fs::*;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let _ = remove_dir_all("output/");
    create_dir("output/")?;
    copy_dir("static/", "output/")?;

    let header = read_to_string("templates/header.html")?;
    let footer = read_to_string("templates/footer.html")?;

    {
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "404".to_string());

        let mut error_404 = String::new();
        error_404.push_str(&header);
        error_404.push_str(&read_to_string("404.html")?);
        error_404.push_str(&footer);

        write("output/404.html", substitute(&error_404, &vars))?;
    }

    let post_header = read_to_string("templates/post-header.html")?;

    let mut posts = Vec::new();
    create_dir("output/posts/")?;
    for post in read_dir("posts/")? {
        let post = post?;
        let mut url = "posts/".to_string();
        let post_filename = post.file_name().into_string().map_err(|_| "")?;
        if post_filename.starts_with('_') { continue; }
        url.push_str(&post_filename[9..].to_string());

        let mut info_path = post.path().to_path_buf();
        info_path.push("info.txt");
        let info = read_to_string(info_path)?;
        let info_lines: Vec<&str> = info.lines().collect();

        let title = info_lines[0].to_string();
        let date = info_lines[1].to_string();
        posts.push((url.clone(), title.clone(), date.clone()));

        let mut out_dir = PathBuf::from("output/");
        out_dir.push(&url);

        create_dir(&out_dir)?;

        for file in read_dir(post.path())? {
            let file = file?;
            let file_name = file.file_name();
            if file_name != "index.md" && file_name != "info.txt" {
                let mut out_path = out_dir.clone();
                out_path.push(file_name);
                copy(file.path(), out_path)?;
            }
        }

        let mut contents_path = post.path().to_path_buf();
        contents_path.push("index.md");

        let contents = render_markdown(&render_katex(&read_to_string(contents_path)?)?)?;

        let mut vars = HashMap::new();
        vars.insert("date".to_string(), date);
        vars.insert("title".to_string(), title);

        let mut post = String::new();
        post.push_str(&header);
        post.push_str(&post_header);
        post.push_str(&contents);
        post.push_str(&footer);

        let mut post_dir = out_dir.clone();
        post_dir.push("index.html");
        write(post_dir, substitute(&post, &vars))?;
    }

    posts.sort_by(|a, b| a.2.cmp(&b.2).reverse());

    {
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "home".to_string());

        let mut index = String::new();
        index.push_str(&header);

        index.push_str(&read_to_string("templates/home.html")?);

        index.push_str(&read_to_string("templates/post-list-begin.html")?);

        let post_list_item = read_to_string("templates/post-list-item.html")?;
        for (url, title, date) in posts {
            let mut vars = HashMap::new();
            vars.insert("url".to_string(), url);
            vars.insert("date".to_string(), date);
            vars.insert("title".to_string(), title);
            index.push_str(&substitute(&post_list_item, &vars));
        }

        index.push_str(&read_to_string("templates/post-list-end.html")?);
        index.push_str(&footer);

        write("output/index.html", substitute(&index, &vars))?;

        copy("output/index.html", "output/posts/index.html")?;
    }    

    Ok(())
}

fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), Box<dyn Error>> {
    let _ = create_dir(&to);

    for entry in read_dir(from)? {
        let entry = entry?;
        let mut out_path = to.as_ref().to_path_buf();
        out_path.push(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(entry.path(), out_path)?;
        } else {
            copy(entry.path(), out_path)?;
        }
    }

    Ok(())
}

fn substitute(input: &str, vars: &HashMap<String, String>) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(c) = chars.next() {
                if c == '{' {
                    let mut name = String::new();
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        }
                        name.push(c);
                    }

                    if let Some(value) = vars.get(&name) {
                        output.push_str(value);
                    }
                } else {
                    output.push('$');
                    output.push(c);
                }
            } else {
                output.push('$');
            }
        } else {
            output.push(c);
        }
    }

    output
}

fn render_markdown(input: &str) -> Result<String, Box<dyn Error>> {
    use pulldown_cmark::{CodeBlockKind, Event::*, Parser, Tag::*, escape::{escape_href, escape_html}};

    let mut output = String::new();

    let mut parser = Parser::new(input).into_offset_iter();
    while let Some((event, range)) = parser.next() {
        match event {
            Start(Paragraph) => output.push_str("<p>"),
            End(Paragraph) => output.push_str("</p>"),
            Start(Heading(_)) => output.push_str("<div class=\"heading\">"),
            End(Heading(_)) => output.push_str("</div>"),
            Start(CodeBlock(info)) => {
                let lang = if let CodeBlockKind::Fenced(info) = info {
                    info.split(' ').next().unwrap().to_string()
                } else {
                    String::new()
                };
                let mut code_range = range.end..range.start;
                while let Some((event, range)) = parser.next() {
                    match event {
                        End(CodeBlock(_)) => {
                            break;
                        }
                        Text(_) => {
                            code_range.start = code_range.start.min(range.start);
                            code_range.end = code_range.end.max(range.end);
                        }
                        _ => {}
                    }
                }
                code_range.end = code_range.end.max(code_range.start);
                output.push_str(&syntax_highlight(&lang, &input[code_range])?);
            }
            Start(List(None)) => output.push_str("<ul>"),
            End(List(None)) => output.push_str("</ul>"),
            Start(List(Some(1))) => output.push_str("<ol>"),
            Start(List(Some(start))) => {
                output.push_str("<ol start=\"");
                output.push_str(&format!("{}", start));
                output.push_str("\">\n")
            }
            End(List(Some(_))) => output.push_str("</ol>"),
            Start(Item) => output.push_str("<li>"),
            End(Item) => output.push_str("</li>"),
            Start(Emphasis) => output.push_str("<em>"),
            End(Emphasis) => output.push_str("</em>"),
            Start(Strong) => output.push_str("<strong>"),
            End(Strong) => output.push_str("</strong>"),
            Start(Strikethrough) => output.push_str("<del>"),
            End(Strikethrough) => output.push_str("</del>"),
            Start(Link(_, dest, _)) => {
                output.push_str("<a class=\"link\" href=\"");
                escape_href(&mut output, &dest)?;
                output.push_str("\">");
            }
            End(Link(_, _, _)) => output.push_str("</a>"),
            Text(text) => escape_html(&mut output, &text)?,
            Code(text) => {
                output.push_str("<code>");
                escape_html(&mut output, &text)?;
                output.push_str("</code>");
            }
            Html(html) => output.push_str(&html),
            _ => {}
        }
    }

    Ok(output)
}

fn render_katex(input: &str) -> Result<String, Box<dyn Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("node")
        .arg("katex.js")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child.stdin.as_mut().ok_or("couldn't open katex")?.write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn syntax_highlight(lang: &str, input: &str) -> Result<String, Box<dyn Error>> {
    use syntect::parsing::SyntaxSet;
    use syntect::highlighting::ThemeSet;
    use syntect::html::highlighted_html_for_string;

    let ss = SyntaxSet::load_defaults_newlines();
    let theme = ThemeSet::get_theme("dropin.tmtheme").unwrap();

    let syntax = ss.find_syntax_by_token(lang).ok_or(format!("language \"{}\" not found", lang))?;

    Ok(highlighted_html_for_string(input, &ss, &syntax, &theme))
}
