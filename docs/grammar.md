# Doit Grammar

```ini
               Node Definitions
         Root: [SOF]         [...]     [EOF]
         Exit: [EXIT]        [EXPR]    [EOL]
        Yield: [YIELD]       [EOL]
       Assign: [NOMEN]       [ASSIGN]  [EXPR]   [EOL]
       Target: [NOMEN]       [TGT_BEG] [...]    [EOL]  [TGT_END]
   SLE Target: [NOMEN]       [TGT_SLE] [SYMBOL] [EXPR] [EOL]
         Help: [HELP]        [EXPR]    [EOL]
       Script: [SCRIPT]      [EXPR]    [EOL]
      Comment: [COMMENT]     [EXPR]    [EOL]
   Help Block: [HELP_BEG]    [...]     [HELP_END]
 Script Block: [SCRIPT_BEG]  [...]     [SCRIPT_END]
Comment Block: [COMMENT_BEG] [...]     [COMMENT_END]
 Required Arg: [ARG_REQ]     [NOMEN]   [HELP]   [EXPR] [EOL]
 Optional Arg: [ARG_OPT]     [NOMEN]   [HELP]   [EXPR] [EOL]
```

## Artifacts

### Expression (`expr`)

An expression is any chain of symbols, literals, and/or variables that create a result. It can be a single literal value, or a variable name, or an arithmetic.

```python
1
my_var + 1
var1 - var2
```

### Literal Number or String (`lit_num`, `lit_str`)

These are bare numbers or quotes strings. All numbers are stored as doubles, and all strings must use double-quotes.

```python
42
21.23
"my string of text"
```

## Grammar

### Exit

The exit command can take an optional integer value. Any value passed will be converted to a `uint8` value that defaults to `0`. This command will immediately terminate the program and set the program error code to the value provided.

```sh
exit [lit_num|expr]\n
# For example
exit 0
exit 1
exit my_result
exit 1+my_result
```

### Yield

The yield command checks the current `EXIT_CODE` state and performs an early exit if it is non-zero. The `EXIT_CODE` value is overrwritten by each invocation of a script. If the script exits with a non-zero code, you can early return and DOIT will also exit that same code. This allows you to propogate errors from internal scripts to outside callers.

```sh
my_target {
	$ exit 0
	yield
	$ echo "This will be printed"

	$ exit 42
	yield
	$ echo "This will not get printed"
}
```

If the above example were to be executed with this command: `doit my_target`, the first `echo` will print and the second will not. The doit program will also exit with the code `42`.

### Help Statement

Within a target, you can provide a help block that can be parsed as a special singleline or multiline block of text that can be printed for the entire program when a user runs the program without any sub-commands. There must only be one Help block per scope or you will get an error.

```
@@@
This is a root help that will be displayed if no target is provided.
It is displayed immediately after the usage statement and before the targets statements.
@@@

my_target {
	@@@
	This help block will be displayed after this target in the help message.
	You can use multiple lines as they will all be padded to the right.
	@@@
}

my_target2 {
  @ You can also make single line helps
}
```

Additionally, help blocks support a small subset of escape characters. You can escape the backslash with a double backslash `\\`.

| character | Description                                                  |
| --------- | ------------------------------------------------------------ |
| `\e`      | Inserts the escape character `0x1B, 033, 27` into the help block string. This allows you to print out ANSI Colours to the terminal.<br />For example: `\e[31m I am a block of red text \e[0m` |
| `\n`      | Standard newline character. NOTE: All separate lines will be indented to align together in the help for individual targets. |
| `\r`      | Standard carriage-return character. NOTE: All separate lines will be indented to align together in the help for individual targets. |

Example:

```
my_target {
	@ This text is normal, \e[31mbut this text becomes red\e[0m
}
```

### Variable Assignment

Variables are simply declared be a single nomen symbol as the first token, followed by the `=` symbol and then an expression. Variables are also scoped and can be defined at any time.

```python
var1 = 42
var2 = 10 + 12
var3 = var1
var4 = var1 + var2 * var3 / 23
```

### Target

A target is essentially a function that can be called from the command line. It is defined by a optional whitespace and then a single `{`. It then captures various additional tokens and is terminated by a single `}`.

You can also define a single line target that contains only ONE single line expression by declaring the name followed by a `:`, and then the expression

```
build {
	[...]
}
run
{
	[...]
}
clean: $ [EXPR]
test: % [EXPR]
```

### Script

Script commands are denoted by a single `$` for single line shell scripts, or can be surrounded by `$$$` for a shell script block. Python scripts are declared with `%` and python script block with `%%%`.

These scripts are run on the system via the C/C++ `system()` call. These are essentially converted to raw-strings. They can contain variables which are used by prefacing the variable name with a single `$` or for better distinction, can be contained within `$(...)`. This is useful if the tail of the variable is beside an alphanumeric character. Arguments passed in from the console can also be accessed using the `$1` style variables. If you wish to reference an environment variable, you can use a double `$$` for the variable reference. The double `$$` will be converted to a single `$` when the script is run.

```sh
my_target1 {
	$ echo "Single line shell script"
	$$$
	echo "shell script block"
	echo "can utilize multiline"
	$$$

	% print('single line python script')
	%%%
	print('shell script block')
	print('can utilize multiline')
	%%%
}
my_target1: $ echo "Single line target using shell script"
my_target1: % print('Single line target using python script')
```

#### CLI Args Expansion

Variable insertion can performa CLI Argument expansion using the following patterns: `$@`, `$(beg:end)`, `$(beg:)`. The `$@` will be expanded to all CLI arguments starting at index `1` (excludes `$0` with is the target name). The second and third describe argument index ranges inclusively. So `$(2:5)` will select 4 arguments at indexes 2-5 inclusively. By ommitting the `end` parameter, it will automatically select all remaining arguments after the `beg` index inclusive. So if 5 arguments are passed `$(2:)` will also select the 4 args at indexes 2-5.

IMPORTANT! Separate script executions do not share environments, so an environment variable set in one is not accessible in another. Please use multiline scripts for such cases.

```sh
my_target {
	$ echo "Hello, World!" | cat
	$ echo "My var is: $my_var" > temp.txt

	$ my_shell_var="not accessible!!"
	$ echo "this will be blank: $my_shell_var"

	$$$
	my_shell_var = "accessible!"
	echo "this will work: $my_shell_var"
	$$$

	# Note: $0 will be the name of the target being run
	$ echo "Access command line args: $0 $1 $2 $3"

	# Expands all CLI args into the script
	$ echo "$@"

	# Injects the number of arguments passed in
	$ echo "$#"

	# Expands all CLI args from index 2, to index 5 inclusively
	$ echo "$(2:5)"
	# Expands all CLI args from index 2, to the last index inclusively
	$ echo "$(2:)"
}
```

### Comment

Comments are denoted by a `#` character and then followed by the comment info. Everything after the `#` to the end of the line will be ignored.

```sh
build { # Comments can occur at the end of any line
	# Or on their own lines
	$ echo "This runs in the shell" # Or at the end of script lines
}
```

### Req/Opt Arguments

Arguments that are required for a specific target can be declared using the `req` and `opt` keywords. These are fairly simple and are just semantics. They provide additional documentation in the DOIT help message generation, and we can check to make sure a minimum number of arguments are provided based on the number of required args (optionals are ignored).

```
build {
	@ Build the project
	opt -r @ Creates an optimized release build
	$ cargo build $@
}

> doit

Usage: doit <target> [args...]
       Project builder

TARGETS
     lint  Test project
     test  Test project
    clean  Clean Project
    build  Build the project

ARGS
    build [-r]
        -r  Creates an optimized release build
```

