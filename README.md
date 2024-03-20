# DOIT

The DOIT scripting language is a very simple and basic script for doing common tasks in a project folder. It has built in `--help` support that is generated from the script itself. Good for quick and simple tasks that aren't complicated and don't need something more complicated like a Makefile.

## Why?

So I know you can just use something like a Makefile to build things, but I get quickly frustrated with Makefiles and their crazy complexity. Most of the time I just want something I can put my simple CLI commands in that's easy to modify and maintain. I've previously used NPM for this, but it is painfully slow to run. So instead I built my own basic scripting language that gets transpiled to C++ and then compiled by the GCC to a very fast executable file.

## Install

### Self Build

1. Build the project following [Build](#Build)
2. Link the executable to your usr/bin: `sudo ln -s /path/to/project/target/release/doit /usr/local/bin/doit`

### From Release

1. Download executable.
2. Copy to `/usr/local/bin` or place somewhere and link it like above.

## Build

Simple run the following commands in the project folder. Requires no additional crates or dependencies to build.

| Command                         | Result                                                   |
| ------------------------------- | -------------------------------------------------------- |
| Debug build: `cargo build`      | You will find the executable at `./targets/debug/doit`   |
| Release build: `cargo build -r` | You will find the executable at `./targets/release/doit` |

There is also a `pre-commit.sh` hook script that can be added to your local repo. This replicates the checks performed by the GitHub PRs. You can also simply run the script manually. You can add a sym-link to your git hooks directory:

```sh
ln -s -f ../../pre-commit.sh .git/hooks/pre-commit
```

## Contributions

The primary goal of this script is to truly follow the ideology of K.I.S.S. Simplicity and ease of use. Yes this means we lose the advantage of certain advanced features, but that's fine. If your build script needs more complexity, then there are better options out there that you can use like Makefile, NPM, Gulp, etc.

### Formatting

I've included a `rustfmt.toml` file that I follow using VSCode and Rust-Analyser. I also use Cargo-Clippy to check for optimisations and best practices.

## How to Use

In your project folder, create a file called `./do.it`. This file will looked for by the program.

This script contains a very basic set of features which is made up of statements and targets.

| Statements                                                   | Description                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| `exit [int]`                                                 | Creates an exit point that can early exit the program with a specific status integer. |
| `my_var = 42`<br />`my_var = "Hello, world!"`<br />`my_var = 21 * my_other_var` | You can create variables that are assigned from the following expression. All variables are stored as either a `double` or a `string`. They are then converted to strings when injected into script statements.<br />These can be defined inside either a target, or at a global scope. Variables are scoped, so a variable defined in one target will not exist in another. Global variables are shared across all targets. |
| `$ echo "Hello, world!" `                                    | Script statements are denoted by a `$` symbol. Everything following the symbol until the end of the line is inserted into a C/C++ `system()` call. Variables can be referenced and are injected at runtime before passing the string to the system call. |
| `# This is a comment`                                        | Comments are denoted by a `#` symbol. Everything after it until the end of the line is considered part of the comment. Comments are added in-place to the C++ source code before compilation, but otherwise don't do anything. |

### Targets

Targets and scripts are the primary purpose of this. A target is defined as follows and can contain any number of comments, scripts, variables inside them.

```
my_target_name {
	# This is a comment inside a target
	my_var = "This is a variable that is scoped to this target only"
	$ time echo "Here is a script line" | cat
	$ echo "My variable: $my_var"
}
```

You can also create a single line target. When you do, the text following the `:` is expected to be a script as if preceded by `$`. Only a script is allowed on a Single Line Target.

```
my_target: echo "my single line target"
```

You can have multiple targets defined:

```
build {
	...
}
clean {
	...
}
run {
	...
}
```

### Help Statements

You can also provide special help blocks that are included in the generated `--help` print out of your script. Only ONE help block is allowed for the global scope and for each target.

```
@@@
This is a special global help that describes my program
@@@

build {
@@@
This is the build target
@@@
}
clean {
	$ echo "This target does not have a help block"
}
```

The above script will output the following generated help:

```
This is a special global help that describes my program

Usage: doit <target> [args...]

TARGETS
  clean  <No help defined>
  build  This is the build target
```

