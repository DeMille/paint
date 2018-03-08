# paint
**A sublime-like syntax highlighter**

```sh
# highlight a file
paint ./file.xx --theme="oceanic next" > index.html

# replace <pre> blocks with highlighted code
paint replace --watch ./plain.html -o highlighted.html
```

👉 [Output Demo](https://demille.github.io/paint)
💾 [Precompiled Binaries](http://link)

Written in Rust, built with @trishume's fantastic [syntect](https://github.com/trishume/syntect) library.

- [Why](#why-not-pygments)
- [How to use](#how-to-use)
- [Install](#install)
- [Complete Usage & Features](#complete-usage-features)


## Why (not pygments)
I want html code snippets that look like my text editor.

For some languages, pygments (or highlight.js) is kinda bland:

<img align="center" src="https://demille.github.io/paint/split.jpg">

Sublime & VS Code have so much more detail! Why? They use more complex parsing grammars than pygments does. Compare the [~70 line](https://bitbucket.org/birkenfeld/pygments-main/src/default/pygments/lexers/javascript.py?fileviewer=file-view-default#javascript.py-33:110) rule set that pygments uses for JavaScript with this fatty [1500 line grammar](https://github.com/babel/babel-sublime/blob/master/JavaScript%20\(Babel\).YAML-tmLanguage) in Sublime. These grammars allow for more sophisticated styling in color schemes.

Now this example is a bit contrived. JavaScript is usually the worst offender with pygments, and not all themes even take advantage of the extra information Sublime provides. But generally, I think the added detail makes a difference in visually parsing text.

To scratch my own itch, I wanted something that:
- [x] looked like what I see in Sublime / VS Code / Github
- [x] had some good defaults out of the box
- [x] could fit into some sort of workflow


## How to use
I thought about a few ways I might use this and broke it into some workflows:

#### • Get the css once, then highlight individual snippets:

1- Get the css
```sh
paint css "github" >> main.css
```

2- Get highlighted html, paste somewhere in `<body>`

```sh
# Get contents of whole file, or parts with --selection=X-Y
paint ./file.xx --html-only | clip

# Or, copy the text you want to add to a page:
paste | paint --filetype="xx" --html-only | clip
```

Piping into [clip](https://blogs.msdn.microsoft.com/oldnewthing/20091110-00/?p=16093) (or pbcopy or xclip) is the bees knees.

#### • Automatically highlight all code blocks within a document:
```sh
paint replace ./raw.html > highlighted.html

# also with watch mode, will re-highlight on file save
paint replace --watch ./raw.html --out highlighted.html
```

Looks for `<pre data-paint="xx"></pre>` blocks within a document and highlights everything inside them, where `xx` is the filetype to use (like using code fences in markdown: ` ```rust `). Add other data attributes to enable other settings.

👉 [Example](https://demille.github.io/paint/before)

#### • Package everything into a script, embed like a gist:
Emulates the functionality of github gists. Outputs a small script that you can load from another page.
Could be useful to keep a page clean from the noisy markup of highlighted examples.
```sh
paint ./file.xx --embed > example.js
```

```html
<div class="embedded">
    <script src="./example.js"></script>
</div>
```

👉 [Example](https://demille.github.io/paint/after)


## Install
Grab precompiled binaries from the [latest release](...) or install from source:

```
cargo install paint
```

*Notes:*
- On windows you'll need to run `vcvarsall.bat x64` first in order to compile `syntect`. </br>My vcvarsall (VS2015) was at: `\Program Files (x86)\Microsoft Visual Studio 14.0\VC\vcvarsall.bat`


## Complete Usage & Features
Noteworthy features not weren't mentioned yet:
- line numbers
- selecting lines X-Y of a file
- adding a highlight to certain lines

<br/>

```
USAGE:
    paint.exe [FLAGS] [OPTIONS] [FILE] [SUBCOMMAND]

FLAGS:
    -b, --border          Wrap output in a border
        --css-inline      Put styles inline instead of using classes
        --css-only        Output css only
        --embed           Emit a js embed script instead of html
    -f, --footer          Adds footer
    -g, --gist-like       Adds line numbers, border, and header
        --help            Prints help information
    -h, --header          Adds header
        --html-only       Output html only
    -n, --line-numbers    Include line numbers
    -V, --version         Prints version information

OPTIONS:
        --css-prefix <prefix>    CSS style prefix, defaults to ".paint"
        --filetype <type>        Specify the filetype when using stdin
        --highlight <lines>      Highlight lines: X[-Y][,...]
    -o, --out <file>             Save result to file instead of stdout
        --selection <lines>      Only include range of lines: N-M
        --syntax <file>          Use given .sublime-syntax for syntax parsing
    -t, --theme <name/path>      Theme name or .tmTheme path, (defaults to "github")
        --title <string>         Title to use for the header or footer

ARGS:
    <FILE>    File to highlight

SUBCOMMANDS:
    css              Generate css for a theme
    dump-syntaxes
    dump-themes
    help             Prints this message or the help of the given subcommand(s)
    list-syntaxes
    list-themes
    replace          Replaces html <pre> blocks in <FILE> with a highlighted version.
                     You need to specify language type w/: <pre data-paint="xx">
                     Enable watch mode with --watch

```


## License
MIT
