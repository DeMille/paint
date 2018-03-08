use std::fmt::Write;
use std::collections::{HashMap, HashSet};

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxDefinition;
use syntect::highlighting::{Theme, Style, Color, FontStyle};

use color;


// holds command line option
#[derive(Debug)]
pub struct Config {
    pub inline: bool,
    pub numbers: bool,
    pub highlighted: HashSet<usize>,
    pub selection: Option<(usize, usize)>,
    pub header: bool,
    pub footer: bool,
    pub border: bool,
    pub title: Option<String>,
    pub filename: String,
    pub css_prefix: String,
}


// wrapper to keep all CSS generation together
struct CssGen<'a> {
    config: &'a Config,
    style_table: HashMap<String, String>,
    fg: Color,
    bg: Color,
    header: Color,
    border: Color,
    highlight: Color,
    line_numbers: Color,
}

impl<'a> CssGen<'a> {
    fn from(theme: &Theme, config: &'a Config) -> CssGen<'a> {
        let bg = theme.settings.background.unwrap_or(Color::WHITE);
        let fg = theme.settings.foreground.unwrap_or(Color::BLACK);

        let style_table = CssGen::make_style_table(theme, &fg, &bg);

        // specific to the github theme
        if theme.name.as_ref().unwrap() == "GitHub Light" {
            return CssGen {
                config,
                style_table,
                fg,
                bg,
                header:       Color { r: 249, g: 249, b: 249, a: 255 }, // #f9f9f9
                border:       Color { r: 221, g: 221, b: 221, a: 255 }, // #dddddd
                highlight:    Color { r: 255, g: 251, b: 221, a: 255 }, // #fffbdd
                line_numbers: Color { r: 190, g: 191, b: 191, a: 255 }, // #bebfbf
            }
        }

        // light themes
        if color::is_light(&bg) {
            return CssGen {
                config,
                style_table,
                fg,
                bg,
                header:       color::darken(&bg, 0.85, 0.95),
                border:       Color { r: 204, g: 204, b: 204, a: 255 }, // #cccccc
                highlight:    theme.settings.selection.unwrap(),
                line_numbers: Color { r: 153, g: 153, b: 153, a: 170 }, // #999999
            }
        }

        // dark themes
        let header = color::lighten(&bg, 0.65, 1.65);
        let border = color::lighten(&header, 0.75, 1.35);
        let highlight = color::lighten(&bg, 1.0, 1.35);
        let line_numbers = color::alpha(&fg, 0.25);

        CssGen {
            config,
            style_table,
            fg,
            bg,
            header,
            border,
            highlight,
            line_numbers,
        }
    }

    fn make_style_table(theme: &Theme, fg: &Color, bg: &Color,)
        -> HashMap<String, String>
    {
        let mut css = Vec::new();

        for scope in &theme.scopes {
            if let Some(fore) = scope.style.foreground {
                if fore != *fg {
                    css.push(format!("color: {};", color::css(&fore)));
                }
            }

            if let Some(back) = scope.style.background {
                if back != *bg {
                    css.push(format!("background: {};", color::css(&back)));
                }
            }
        }

        css.sort();
        css.dedup();

        let mut styles = HashMap::new();
        let mut i = 1;

        for style in css {
            let class = format!("pt{}", i);
            styles.insert(style, class);
            i += 1;
        }

        styles
    }

    fn base(&self) -> String {
        let prefix = &self.config.css_prefix;
        let div = self.outer_div();
        let table = self.table();
        let td = self.td();
        let ln = self.line_numbers();
        let hi = self.highlight();

        // using pseudo ::after for line numbers, this prevents them
        // from being copy-paste-able. (which would be annoying)
        let mut out = collapse_whitespace(3, format!(r#"
            .{prefix} {{
                {}
            }}
            .{prefix} table {{
                {}
            }}
            .{prefix} td {{
                {}
            }}
            .{prefix} .ln {{
                {}
            }}
            .{prefix} .ln::after {{
                content: attr(data-ln);
            }}
            .{prefix} .hi {{
                {}
            }}
            .{prefix} .un {{ text-decoration: underline; }}
            .{prefix} .bo {{ font-weight: bold; }}
            .{prefix} .it {{ font-style: italic; }}
        "#, div, table, td, ln, hi, prefix=prefix));

        for (css, class) in &self.style_table {
            write!(out, ".{} .{} {{ {} }}\n", prefix, class, css).unwrap();;
        }

        out
    }

    fn style(&self, style: &Style) -> Vec<(String, String)> {
        let mut styles = Vec::new();

        if style.foreground != self.fg {
            let css = format!("color: {};", color::css(&style.foreground));
            let class = self.style_table.get(&css).unwrap();

            styles.push((class.clone(), css));
        }

        if style.background != self.bg {
            let css = format!("background: {};", color::css(&style.background));
            let class = self.style_table.get(&css).unwrap();

            styles.push((class.clone(), css));
        }

        if style.font_style.contains(FontStyle::UNDERLINE) {
            styles.push((
                String::from("un"),
                String::from("text-decoration: underline;")
            ));
        }

        if style.font_style.contains(FontStyle::BOLD) {
            styles.push((
                String::from("bo"),
                String::from("font-weight: bold;")
            ));
        }

        if style.font_style.contains(FontStyle::ITALIC) {
            styles.push((
                String::from("it"),
                String::from("font-style: italic;")
            ));
        }

        styles
    }

    fn outer_div(&self) -> String {
        let fg = color::css(&self.fg);
        let bg = color::css(&self.bg);

        collapse_whitespace(2, format!("\
            display: block;
            width: 100%;
            padding: 10px 0;
            overflow-x: auto;
            -webkit-overflow-scrolling: touch;
            color: {};
            background-color: {};\
        ", fg, bg))
    }

    fn table(&self) -> String {
        collapse_whitespace(2, String::from("\
            width: 100%;
            border-spacing: 0;
            border-collapse: separate;
            font-family: SFMono-Regular, Consolas, \"Liberation Mono\", Menlo, monospace;
            font-size: 12px;
            line-height: 20px;
            tab-size: 4;
            color: inherit;
            -webkit-text-size-adjust: 100%;
            -moz-text-size-adjust: 100%;
            -ms-text-size-adjust: 100%;
            text-rendering: optimizeLegibility;
        "))
    }

    fn td(&self) -> &'static str {
        if self.config.numbers {
            "padding: 0 10px; white-space: pre;"
        } else {
            "padding: 0 13px; white-space: pre;"
        }
    }

    fn line_numbers(&self) -> String {
        collapse_whitespace(2, format!("\
            width: 1px;
            min-width: 25px;
            box-sizing: content-box;
            text-align: right;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            user-select: none;
            color: {}; \
        ", color::css(&self.line_numbers)))
    }

    fn highlight(&self) -> String {
        format!("background-color: {};", color::css(&self.highlight))
    }

    fn bordered(&self) -> String {
        let rest = self.base();

        let border = color::css(&self.border);
        let background = color::css(&self.header);
        let color = color::css(&color::alpha(&self.fg, 0.75));
        let divider = color::css(&color::alpha(&self.fg, 0.10));

        collapse_whitespace(3, format!("\
            .{prefix}-bordered {{
                border: 1px solid {border};
                border-radius: 2px;
            }}
            .{prefix}-bordered .info {{
                display: flex;
                justify-content: space-between;
                color: {color};
                background: {background};
                margin: 0;
                padding: 10px 15px 10px;
                font-size: 12px;
                font-family: SFMono-Regular, Consolas, \"Liberation Mono\", Menlo, monospace;
                line-height: 1.2;
            }}
            .{prefix}-bordered .info.header {{
                border-bottom: 1px solid {border};
            }}
            .{prefix}-bordered .info.footer {{
                border-top: 1px solid {border};
            }}
            .{prefix}-bordered .info .left {{
                font-weight: 500;
                font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif;
            }}
            .{prefix}-bordered .info .right span:not(:last-child) {{
                padding-right: 10px;
                margin-right: 10px;
                border-right: 1px solid {divider};
            }}
            {rest}\
        ", prefix = self.config.css_prefix,
           border = border,
           color = color,
           background = background,
           divider = divider,
           rest = rest))
    }
}


pub fn css(theme: &Theme, config: &Config) -> String {
    CssGen::from(theme, config).bordered()
}


pub fn highlight(text: &str,
                 syntax: &SyntaxDefinition,
                 theme: &Theme,
                 config: &Config) -> (String, String) {

    let gen = CssGen::from(theme, config);
    let base = make_base_html(text, syntax, theme, config, &gen);

    let css = if config.border { gen.bordered() } else { gen.base() };
    let html = if config.border { add_border(&base, config) } else { base };

    (html, css)
}


pub fn embed_script(html: &str, css: &str) -> String {
    format!("document.write('<style scoped>{}</style>');\ndocument.write('{}');",
        collapse_whitespace(1, escape_js(css)),
        escape_js(html))
}


pub fn fullpage(html: &str, css: &str, theme: &Theme) -> String {
    let bg = theme.settings.background.unwrap_or(Color::WHITE);
    let background = color::css(&bg);
    let filter = if !color::is_light(&bg) { "filter: brightness(90%)" } else { "" };

    collapse_whitespace(2, format!("\
        <html>
        <head>
            <meta name='viewport' content='width=device-width, initial-scale=1'>
            <style>
                html, body {{
                    margin: 0;
                    padding: 0;
                }}
                div.bg {{
                    position: fixed;
                    top: 0;
                    bottom: -100px;
                    width: 100%;
                    z-index: -1;
                    background-color: {};
                    {}
                }}
                .container {{
                    max-width: 850px;
                    margin: 25px auto;
                    padding: 0 25px;
                }}
                {css}
            </style>
        </head>
        <body>
            <div class='bg'></div>
            <div class='container'>
                {html}
            </div>
        </body>
        </html>
    ", background, filter, css = css, html = html))
}


fn add_border(body: &str, config: &Config) -> String {
    let prefix = &config.css_prefix;
    let class = if config.footer { "footer" } else { "header" };

    if !config.header && !config.footer {
        return format!("<div class='{}-bordered'>{}</div>", prefix, body);
    }

    let (left, right) = config.title.as_ref().map_or_else(
        || {
            (
                format!("{}", config.filename),
                format!("{} lines", body.matches("<tr>").count())
            )
        },
        |title| {
            let parts = title.split("|").collect::<Vec<_>>();

            (
                format!("{}", parts[0]),
                format!("{}", parts.get(1).unwrap_or(&""))
            )
        }
    );

    let info = format!(r#"
        <div class="info {}">
            <span class="left">{}</span>
            <span class="right">{}</span>
        </div>
    "#, class, left, right);

    if config.footer {
        format!("<div class='{}-bordered'>{}{}</div>", prefix, body, info)
    } else {
        format!("<div class='{}-bordered'>{}{}</div>", prefix, info, body)
    }
}


fn make_base_html(text: &str,
                  syntax: &SyntaxDefinition,
                  theme: &Theme,
                  config: &Config,
                  css_gen: &CssGen) -> String {

    let ln = css_gen.line_numbers();
    let hi = css_gen.highlight();
    let td = css_gen.td();

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut out = String::new();
    let mut num = 0;

    if config.inline {
        write!(out, "<div style='{}'>\n", css_gen.outer_div()).unwrap();
        write!(out, "<table style='{}'>\n", css_gen.table()).unwrap();
    } else {;
        write!(out, "<div class='{}'>\n<table>\n", config.css_prefix).unwrap();
    }

    for line in text.lines() {
        num += 1;

        // always pass lines to highlighter so w/e funky regexes it uses
        // across lines will work, even if we don't show that line
        let contents = highlighter.highlight(&line);
        let mut html = line_to_html(&contents[..], css_gen, config.inline);

        // skip lines not included in user selection (if any)
        if let Some((start, finish)) = config.selection {
            if num < start || num > finish { continue; }
        }

        // `line` never contains a newline char, but we *want* a \n for empty
        // lines. Otherwise empty lines would collapse row on the table, and
        // you want newlines to be user copy/paste-able. (Note: '&#10;' vs '\n')
        if html.is_empty() {
            html = String::from("&#10;");
        }

        out.push_str("<tr>");

        if config.inline {
            if config.numbers {
                write!(out, "<td style='{}{}'>{}</td>", ln, td, num).unwrap();
            }

            if config.highlighted.contains(&num) {
                write!(out, "<td style='{}{}'>{}</td>", hi, td, html).unwrap();
            } else {
                write!(out, "<td style='{}'>{}</td>", td, html).unwrap();
            }
        } else {
            if config.numbers {
                write!(out, "<td class='ln' data-ln='{}'></td>", num).unwrap();
            }

            if config.highlighted.contains(&num) {
                write!(out, "<td class='hi'>{}</td>", html).unwrap();
            } else {
                write!(out, "<td>{}</td>", html).unwrap();
            }
        }

        out.push_str("</tr>\n");
    }

    out.push_str("</table>\n</div>\n");
    out
}


fn line_to_html(v: &[(Style, &str)], css_gen: &CssGen, inline: bool) -> String {
    let mut out = String::new();
    let mut prev_style: Option<&Style> = None;

    for &(ref style, text) in v.iter() {
        let unify_style = if let Some(ps) = prev_style {
            style == ps
        } else {
            false
        };

        if unify_style {
            write!(out, "{}", escape_html(text)).unwrap();
        } else {
            if prev_style.is_some() {
                write!(out, "</span>").unwrap();
            }

            let html = escape_html(text);
            let mut classes = Vec::new();
            let mut css = Vec::new();

            for style in css_gen.style(&style) {
                classes.push(style.0);
                css.push(style.1);
            }

            prev_style = if !css.is_empty() { Some(style) } else { None };

            if css.is_empty() {
                write!(out, "{}", html).unwrap();
            } else if inline {
                write!(out, "<span style='{}'>{}", css.join(" "), html).unwrap();
            } else {
                write!(out, "<span class='{}'>{}", classes.join(" "), html).unwrap();
            }
        }
    }

    if prev_style.is_some() {
        write!(out, "</span>").unwrap();
    }

    out
}


fn escape_html(text: &str) -> String {
    let original = text;
    let mut last = 0;
    let mut out = String::new();

    for (i, ch) in text.bytes().enumerate() {
        match ch as char {
            '<' | '>' | '&' | '\'' | '"' => {
                write!(out, "{}", &original[last..i]).unwrap();

                let text = match ch as char {
                    '>' => "&gt;",
                    '<' => "&lt;",
                    '&' => "&amp;",
                    '\'' => "&#39;",
                    '"' => "&quot;",
                    _ => unreachable!(),
                };

                write!(out, "{}", text).unwrap();
                last = i + 1;
            }
            _ => {}
        }
    }

    if last < text.len() {
        write!(out, "{}", &original[last..]).unwrap();
    }

    out
}


fn escape_js(text: &str) -> String {
    let original = text;
    let mut last = 0;
    let mut out = String::new();

    for (i, ch) in text.bytes().enumerate() {
        match ch as char {
            '\n' | '\'' | '\\' => {
                write!(out, "{}", &original[last..i]).unwrap();

                let text = match ch as char {
                    '\n' => "",
                    '\'' => "\\'",
                    '\\' => "\\\\",
                    _ => unreachable!(),
                };

                write!(out, "{}", text).unwrap();
                last = i + 1;
            }
            _ => {}
        }
    }

    if last < text.len() {
        write!(out, "{}", &original[last..]).unwrap();
    }

    out
}


fn collapse_whitespace(n: usize, text: String) -> String {
    let indent = String::from("    ").repeat(n);

    text.split('\n')
        .map(|line| line.trim_left_matches(&indent))
        .collect::<Vec<_>>()
        .join("\n")
}


#[allow(dead_code)]
fn human_readable(filesize: usize) -> String {
    let exp = ((filesize as f64).ln() / 1024_f64.ln()).floor();
    let size = (filesize as f64) / 1024_f64.powi(exp as i32);
    let unit = ["B", "KB", "MB"][exp as usize];

    format!("{:.2} {}", size, unit)
}
