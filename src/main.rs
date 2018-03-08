use std::io::{self, Read, Write};
use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::process;
use std::sync::mpsc::channel;
use std::time::Duration;

#[macro_use]
extern crate clap;
extern crate syntect;
extern crate notify;
extern crate regex;

use clap::{App, ArgMatches, SubCommand};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxDefinition};
use syntect::dumps;
use regex::{Regex, Captures};
use notify::{RecommendedWatcher, Watcher, RecursiveMode};

mod color;
mod paint;
use paint::Config;


fn main() {
    let usage = r#"
        [FILE]                   'File to highlight'
        -o, --out=[file]         'Save result to file instead of stdout'
        --filetype=[type]        'Specify the filetype when using stdin'
        --embed                  'Emit a js embed script instead of html'
        -t, --theme=[name/path]  'Theme name or .tmTheme path, (defaults to "github")'
        --syntax=[file]          'Use given .sublime-syntax for syntax parsing'
        --html-only              'Output html only'
        --css-only               'Output css only'
        --css-inline             'Put styles inline instead of using classes'
        --css-prefix=[prefix]    'CSS style prefix, defaults to ".paint"'
        -n, --line-numbers       'Include line numbers'
        -b, --border             'Wrap output in a border'
        -h, --header             'Adds header'
        -f, --footer             'Adds footer'
        -g, --gist-like          'Adds line numbers, border, and header'
        --title=[string]         'Title to use for the header or footer'
        --highlight=[lines]      'Highlight lines: X[-Y][,...]'
        --selection=[lines]      'Only include range of lines: N-M'
    "#;

    let replace_usage = format!("{}\n{}",
                                usage,
                                "-w, --watch 'Watch input file for changes'");

    let args = App::new("paint")
        .about("A sublime text style syntax highlighter that outputs HTML\n
EXAMPLE:
    paint ./file.xx --theme=\"oceanic next\" > index.html")
        .version(crate_version!())
        .args_from_usage(usage)

        .subcommand(SubCommand::with_name("replace")
            .about(r#"Replaces html <pre> blocks in <FILE> with a highlighted version.
You need to specify language type w/: <pre data-paint="xx">
Enable watch mode with --watch"#)
            .args_from_usage(&replace_usage))

        .subcommand(SubCommand::with_name("css")
            .about("Generate css for a theme")
            .args_from_usage("[theme] 'Get css for this theme'"))

        .subcommand(SubCommand::with_name("list-themes"))
        .subcommand(SubCommand::with_name("list-syntaxes"))

        // development commands, easier to add here
        .subcommand(SubCommand::with_name("dump-themes")
            .args_from_usage("<folder> 'Dump syntaxes from folder'"))
        .subcommand(SubCommand::with_name("dump-syntaxes")
            .args_from_usage("<folder> 'Dump themes from folder'"))
        .get_matches();

    match args.subcommand() {
        ("list-themes", _)         => list_themes(),
        ("list-syntaxes", _)       => list_syntaxes(),
        ("dump-themes", Some(a))   => dump_themes(a.value_of("folder").unwrap()),
        ("dump-syntaxes", Some(a)) => dump_syntaxes(a.value_of("folder").unwrap()),
        ("css", Some(a))           => make_css(a),
        ("replace", Some(a))       => replace(a),
        _                          => highlight(&args),
    };
}


fn get_included_themes() -> ThemeSet {
    let set = include_bytes!("../assets/themes.themedump");
    dumps::from_binary(set)
}


fn get_syntaxes() -> SyntaxSet {
    let bytes = include_bytes!("../assets/syntaxes.packdump");
    let extra: SyntaxSet = dumps::from_binary(bytes);

    let mut defaults = SyntaxSet::load_defaults_nonewlines();

    for syntax in extra.syntaxes() {
        defaults.add_syntax(syntax.to_owned());
    }

    defaults.link_syntaxes();
    defaults
}


fn list_themes() {
    println!("Included themes:");

    for name in get_included_themes().themes.keys() {
        println!("- {}", name);
    }
}


fn list_syntaxes() {
    println!("Included syntaxes:");

    for syntax in get_syntaxes().syntaxes() {
        println!("- {}", syntax.name);
    }
}


fn dump_themes(folder: &str) {
    println!("Dumping themes to ./assets/themes.themedump");

    let set = ThemeSet::load_from_folder(folder).unwrap();
    dumps::dump_to_file(&set, "./assets/themes.themedump").unwrap();
}


fn dump_syntaxes(folder: &str) {
    println!("Dumping syntaxes to ./assets/syntaxes.packdump");

    let mut set = SyntaxSet::new();
    set.load_syntaxes(folder, false).unwrap();

    dumps::dump_to_file(&set, "./assets/syntaxes.packdump").unwrap();
}


fn parse_selection(lines: Option<&str>) -> Option<(usize, usize)> {
    match lines {
        None => None,
        Some(s) => {
            let ns = s.split('-').map(|n| n.parse().unwrap()).collect::<Vec<_>>();
            let start = ns.get(0).expect("Problem parsing selection number");
            let end = ns.get(1).unwrap_or(&start);

            Some((*start,  *end))
        }
    }
}


fn parse_highlighted(lines: Option<&str>) -> HashSet<usize> {
    let mut highlighted = HashSet::new();

    if let Some(sections) = lines {
        for section in sections.split(',') {
            let mut start = 0;
            let mut end = 0;

            for number in section.split('-').map(|n| n.parse().unwrap()) {
                if start == 0 && number > start { start = number; }
                if number > end { end = number; }
            }

            for n in start..end + 1 {
                highlighted.insert(n);
            }
        }
    }

    highlighted
}


fn get_input_from(args: &ArgMatches) -> String {
    if args.value_of("FILE").or(args.value_of("filetype")).is_none() {
        eprintln!("missing FILE to highlight (or use --filetype with stdin)");
        process::exit(1);
    }

    let mut input = String::new();

    match args.value_of("FILE") {
        Some(file) => File::open(file)
                           .expect("Can't open file!")
                           .read_to_string(&mut input)
                           .unwrap(),

        None => io::stdin()
                   .read_to_string(&mut input)
                   .expect("Problem reading from stdin..."),
    };

    input
}


fn get_theme(setting: Option<&str>) -> Theme {
    let set = get_included_themes();

    // get theme from a name or path
    if let Some(name) = setting {
        let theme = if set.themes.contains_key(name) {
            Ok(set.themes[name].to_owned())
        } else {
            ThemeSet::get_theme(name)
        };

        if theme.is_err() {
            eprintln!(
                "'{}' is not included or there was a problem with the theme file:\n{:?}",
                name,
                theme.unwrap_err());

            process::exit(1);
        }

        theme.unwrap()
    } else {
        set.themes["github"].to_owned()
    }
}


fn make_config(args: &ArgMatches) -> Config {

    let filename = if let Some(file) = args.value_of("FILE") {
        Path::new(file).file_name().unwrap().to_string_lossy().into_owned()
    } else {
        String::from("stdin")
    };

    let title = args.value_of("title")
                    .map(|s| String::from(s));

    let prefix = args.value_of("css-prefix")
                     .map(|s| String::from(s))
                     .unwrap_or(String::from("paint"));

    Config {
        filename:    filename,
        title:       title,
        css_prefix:  prefix,
        inline:      args.is_present("css-inline"),
        numbers:     args.is_present("line-numbers") || args.is_present("gist-like"),
        border:      args.is_present("border") || args.is_present("gist-like"),
        header:      args.is_present("header") || args.is_present("gist-like"),
        footer:      args.is_present("footer"),
        highlighted: parse_highlighted(args.value_of("highlight")),
        selection:   parse_selection(args.value_of("selection")),
    }
}


fn modify_config(mut config: Config, pre: &str) -> Config {
    // match settings with "data-setting=x"
    let gist    = Regex::new(r#"^<pre.*?data-gist-like.*?>"#).unwrap();
    let header  = Regex::new(r#"^<pre.*?data-header.*?>"#).unwrap();
    let footer  = Regex::new(r#"^<pre.*?data-footer.*?>"#).unwrap();
    let border  = Regex::new(r#"^<pre.*?data-border.*?>"#).unwrap();
    let inline  = Regex::new(r#"^<pre.*?data-css-inline.*?>"#).unwrap();
    let numbers = Regex::new(r#"^<pre.*?data-line-numbers.*?>"#).unwrap();
    let title   = Regex::new(r#"^<pre.*?data-title="(.+?)".*?>"#).unwrap();
    let high    = Regex::new(r#"^<pre.*?data-highlight="(.+?)".*?>"#).unwrap();
    let prefix  = Regex::new(r#"^<pre.*?data-css-prefix="(.+?)".*?>"#).unwrap();

    if gist.captures(pre).is_some() {
        config.header = true;
        config.border = true;
        config.numbers = true;
    }

    if header.captures(pre).is_some() { config.header = true; }
    if footer.captures(pre).is_some() { config.footer = true; }
    if border.captures(pre).is_some() { config.border = true; }
    if inline.captures(pre).is_some() { config.inline = true; }
    if numbers.captures(pre).is_some() { config.numbers = true; }

    if let Some(c) = title.captures(pre) {
        config.title = Some(String::from(&c[1]));
    }

    if let Some(c) = high.captures(pre) {
        config.highlighted = parse_highlighted(Some(&c[1]));
    }

    if let Some(c) = prefix.captures(pre) {
        config.css_prefix = String::from(&c[1]);
    }

    config
}


fn make_syntax<'a>(path: &str, temp: &'a mut SyntaxSet) -> &'a SyntaxDefinition {
    let mut file = File::open(path).expect("Can't open syntax file");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let syntax = SyntaxDefinition::load_from_str(&data, false, None).unwrap();
    let scope = syntax.scope;

    temp.add_syntax(syntax);
    temp.link_syntaxes();
    temp.find_syntax_by_scope(scope).unwrap()
}


fn find_syntax<'a>(mut token: &str, set: &'a SyntaxSet) -> &'a SyntaxDefinition {
    // lil manual override
    token = match token.to_lowercase().as_ref() {
        "js" | "jsx" => "JavaScript (Babel)",
        "rs" => "Rust Enhanced",
        _ => token,
    };

    set.find_syntax_by_token(token)
        .unwrap_or_else(|| set.find_syntax_plain_text())
}


fn highlight_string(input: &str,
                    filetype: &str,
                    syntax_path: Option<&str>,
                    theme: &Theme,
                    config: &Config) -> (String, String) {

    // ownership issue, need syntax sets higher in scope so they don't get dropped
    let set = get_syntaxes();
    let mut temp_set = SyntaxSet::new();

    let syntax = match syntax_path {
        Some(path) => make_syntax(path, &mut temp_set),
        None       => find_syntax(filetype, &set),
    };

    paint::highlight(&input, &syntax, theme, &config)
}


fn make_css(args: &ArgMatches) {
    let theme = get_theme(args.value_of("theme"));
    let config = make_config(args);

    println!("{}", paint::css(&theme, &config));
}


fn replace_pre_blocks(input: &str, args: &ArgMatches) -> String {
    // match <pre data-paint="syntax">...</pre>
    let pre = Regex::new(
        r#"<pre.*?data-paint="([\w\d]+)".*?>([\s\S]*?)</pre>"#
    ).unwrap();

    let theme_re = Regex::new(r#"^<pre.*?data-theme="(.+?)".*?>"#).unwrap();
    let html_only = Regex::new(r#"^<pre.*?data-html-only.*?>"#).unwrap();
    let css_inline = Regex::new(r#"^<pre.*?data-css-inline.*?>"#).unwrap();

    let file_contents = pre.replace_all(&input, |cap: &Captures| {
        let outer = &cap[0];
        let inner = &cap[2].trim();
        let filetype = &cap[1];

        // override settings per code block
        let config = modify_config(make_config(&args), &outer);

        // theme could be different per block too
        let theme = match theme_re.captures(outer) {
            Some(c) => get_theme(Some(&c[1])),
            None    => get_theme(args.value_of("theme")),
        };

        let syntax_path = args.value_of("syntax");
        let (html, css) = highlight_string(inner, filetype, syntax_path, &theme, &config);

        let no_css = args.is_present("html-only") ||
                     html_only.captures(outer).is_some() ||
                     css_inline.captures(outer).is_some();

        if no_css {
            html
        } else {
            format!("<style scoped>{}</style>\n{}", &css, &html)
        }
    });

    file_contents.to_string()
}


fn write_to_file(input: &str, path: &str) -> Result<(), io::Error> {
    println!("[\u{001B}[34mpaint\u{001B}[0m] \u{001B}[97mWriting:\u{001B}[0m {}", path);

    let mut file = File::create(path)?;
    file.write(input.as_bytes())?;

    Ok(())
}


fn watch(args: &ArgMatches) -> notify::Result<()> {
    // this wont work with stdin / stdout:
    if args.value_of("FILE").is_none() || args.value_of("out").is_none() {
        eprintln!("Watch mode requires both an input file and an output file.\n
(stdin / stdout won't work here)");
        process::exit(1);
    }

    let watch_path = args.value_of("FILE").unwrap();
    let output_path = args.value_of("out").unwrap();

    let (tx, rx) = channel();
    let debounce = Duration::from_millis(10); // basically 0
    let mut watcher: RecommendedWatcher = Watcher::new(tx, debounce)?;

    watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

    println!("\n[\u{001B}[34mpaint\u{001B}[0m] \u{001B}[96mStarting watcher:\u{001B}[0m {} -> {}",
        watch_path, output_path);

    {
        let input = get_input_from(args);
        let output = replace_pre_blocks(&input, args);

        write_to_file(&output, output_path).unwrap();
    }

    loop {
        match rx.recv() {
            Ok(notify::DebouncedEvent::Write(_)) => {
                println!("[\u{001B}[34mpaint\u{001B}[0m] \u{001B}[97mUpdate: \u{001B}[0m {}",
                    watch_path);

                let input = get_input_from(args);
                let output = replace_pre_blocks(&input, args);

                write_to_file(&output, output_path).unwrap();
            },
            Err(e) => println!("Watch error: {:?}", e),
            _ => (),
        }
    }
}


fn replace(args: &ArgMatches) {
    if args.is_present("css-only") {
        make_css(args);
        return;
    }

    if args.is_present("watch") {
        watch(args).unwrap();
        return;
    }

    let input = get_input_from(args);
    let output = replace_pre_blocks(&input, args);

    match args.value_of("out") {
        Some(path) => write_to_file(&output, path).unwrap(),
        None => println!("{}", output),
    }
}


fn highlight(args: &ArgMatches) {
    let input = get_input_from(&args);

    let filetype = args.value_of("filetype").unwrap_or_else(||
                   args.value_of("FILE").unwrap().split(".").last().unwrap());

    let syntax = args.value_of("syntax");
    let theme  = get_theme(args.value_of("theme"));
    let config = make_config(&args);

    let (html, css) = highlight_string(&input, &filetype, syntax, &theme, &config);

    let output = if args.is_present("css-only") {
        css
    } else if args.is_present("html-only") {
        html
    } else if args.is_present("embed") {
        paint::embed_script(&html, &css)
    } else {
        paint::fullpage(&html, &css, &theme)
    };

    match args.value_of("out") {
        Some(path) => write_to_file(&output, path).unwrap(),
        None => println!("{}", output),
    }
}
