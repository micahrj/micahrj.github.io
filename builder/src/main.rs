use std::collections::HashMap;
use std::error::Error;
use std::fs::*;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::{DateTime, FixedOffset, Local, SecondsFormat};
use serde::Deserialize;

struct Post {
    url: String,
    title: String,
    date: DateTime<FixedOffset>,
    excerpt: String,
    contents: String,
}

#[derive(Deserialize)]
struct Metadata {
    #[serde(default)]
    title: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    draft: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let now = Local::now();

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

    let post_template = read_to_string("templates/post.html")?;

    let mut posts = Vec::new();
    create_dir("output/posts/")?;
    for post in read_dir("posts/")? {
        let post = post?;
        let mut url = "posts/".to_string();
        let post_filename = post.file_name().into_string().map_err(|_| "")?;
        url.push_str(&post_filename[11..].to_string());

        let mut contents_path = post.path().to_path_buf();
        contents_path.push("index.md");

        let source = read_to_string(contents_path)?;

        let mut parts = source.split("+++");
        let _ = parts.next().ok_or_else(|| {
            format!(
                "unexpected content before front matter in {}",
                post_filename
            )
        })?;
        let front = parts
            .next()
            .ok_or_else(|| format!("expected front matter in {}", post_filename))?;
        let content = parts
            .next()
            .ok_or_else(|| format!("expected content after front matter in {}", post_filename))?;

        let metadata: Metadata = toml::from_str(front)?;

        if metadata.draft {
            continue;
        }

        let date = DateTime::from_str(&metadata.date)?;

        let mut out_dir = PathBuf::from("output/");
        out_dir.push(&url);

        create_dir(&out_dir)?;

        for file in read_dir(post.path())? {
            let file = file?;
            let file_name = file.file_name();
            if file_name != "index.md" {
                let mut out_path = out_dir.clone();
                out_path.push(file_name);
                copy(file.path(), out_path)?;
            }
        }

        let rendered = render_markdown(&render_katex(content)?)?;

        let mut post = String::new();
        post.push_str(&header);
        post.push_str(&post_template);
        post.push_str(&footer);

        let mut vars = HashMap::new();
        vars.insert("date".to_string(), format!("{}", date.format("%Y-%m-%d")));
        vars.insert("title".to_string(), metadata.title.clone());
        vars.insert("contents".to_string(), rendered.clone());
        let substituted = substitute(&post, &vars);

        let mut post_dir = out_dir.clone();
        post_dir.push("index.html");
        write(post_dir, &substituted)?;

        let excerpt = rendered.split("<!--excerpt-->").nth(0).unwrap().to_string();

        posts.push(Post {
            url,
            title: metadata.title,
            date,
            excerpt,
            contents: rendered,
        });
    }

    posts.sort_by(|a, b| a.date.cmp(&b.date).reverse());

    {
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "Home".to_string());

        let mut index = String::new();
        index.push_str(&header);

        let post_list_item = read_to_string("templates/post-list-item.html")?;
        for post in &posts {
            let mut vars = HashMap::new();
            vars.insert("url".to_string(), post.url.clone());
            vars.insert(
                "date".to_string(),
                format!("{}", post.date.format("%Y-%m-%d")),
            );
            vars.insert("title".to_string(), post.title.clone());
            vars.insert("excerpt".to_string(), post.excerpt.clone());
            index.push_str(&substitute(&post_list_item, &vars));
        }

        index.push_str(&footer);

        write("output/index.html", substitute(&index, &vars))?;

        copy("output/index.html", "output/posts/index.html")?;
    }

    {
        let page_template = read_to_string("templates/page.html")?;

        let source = read_to_string("pages/about.md")?;
        let rendered = render_markdown(&source)?;

        let mut page = String::new();
        page.push_str(&header);
        page.push_str(&page_template);
        page.push_str(&footer);

        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "About Me".to_string());
        vars.insert("contents".to_string(), rendered.clone());
        let substituted = substitute(&page, &vars);

        create_dir("output/about/")?;
        write("output/about/index.html", &substituted)?;
    }

    {
        let header = read_to_string("templates/feed/header.xml")?;
        let entry = read_to_string("templates/feed/entry.xml")?;
        let footer = read_to_string("templates/feed/footer.xml")?;

        let mut w = BufWriter::new(File::create("output/feed.xml")?);

        let updated = now.to_rfc3339_opts(SecondsFormat::Secs, true);

        let mut vars = HashMap::new();
        vars.insert("updated".to_string(), updated.clone());
        write!(w, "{}", substitute(&header, &vars))?;

        for post in &posts {
            let date = post.date.to_rfc3339_opts(SecondsFormat::Secs, true);

            let mut vars = HashMap::new();
            vars.insert("url".to_string(), post.url.clone());
            vars.insert("title".to_string(), post.title.clone());
            vars.insert("published".to_string(), date.clone());
            vars.insert("updated".to_string(), date.clone());
            vars.insert("content".to_string(), post.contents.clone());
            vars.insert("summary".to_string(), post.excerpt.clone());
            write!(w, "{}", substitute(&entry, &vars))?;
        }

        write!(w, "{}", footer)?;
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
    use pulldown_cmark::{
        escape::{escape_href, escape_html},
        CodeBlockKind,
        Event::*,
        Parser,
        Tag::*,
    };

    let mut output = String::new();

    let mut parser = Parser::new(input).into_offset_iter();
    while let Some((event, range)) = parser.next() {
        match event {
            Start(Paragraph) => output.push_str("<p>"),
            End(Paragraph) => output.push_str("</p>"),
            Start(Heading(_)) => output.push_str("<h3>"),
            End(Heading(_)) => output.push_str("</h3>"),
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
                output.push_str("<a href=\"");
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
    use std::process::{Command, Stdio};

    let mut child = Command::new("node")
        .arg("katex.js")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .ok_or("couldn't open katex")?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn syntax_highlight(lang: &str, input: &str) -> Result<String, Box<dyn Error>> {
    use syntect::highlighting::ThemeSet;
    use syntect::html::highlighted_html_for_string;
    use syntect::parsing::SyntaxSet;

    let ss = SyntaxSet::load_defaults_newlines();
    let theme = ThemeSet::get_theme("dropin.tmtheme").unwrap();

    let syntax = ss
        .find_syntax_by_token(lang)
        .ok_or(format!("language \"{}\" not found", lang))?;

    Ok(highlighted_html_for_string(input, &ss, &syntax, &theme))
}
