# Doit Grammar

```ini
               Node Definitions
         Root: [SOF]         [...]     [EOF]
         Exit: [EXIT]        [EXPR]    [EOL]
       Assign: [NOMEN]       [ASSIGN]  [EXPR] [EOL]
       Target: [NOMEN]       [TGT_BEG] [...]  [EOL] [TGT_END]
   SLE Target: [NOMEN]       [TGT_SLE] [EXPR] [EOL]
         Help: [HELP]        [EXPR]    [EOL]
       Script: [SCRIPT]      [EXPR]    [EOL]
      Comment: [COMMENT]     [EXPR]    [EOL]
   Help Block: [HELP_BEG]    [...]     [HELP_END]
 Script Block: [SCRIPT_BEG]  [...]     [SCRIPT_END]
Comment Block: [COMMENT_BEG] [...]     [COMMENT_END]
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

### Help Statement

Within a target, you can provide a help block that can be parsed as a special singleline or multiline block of text that can be printed for the entire program when a user passes the `-h` or `--help` flags to the program. The must only be one Help block per scope or you will get an error.

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
clean: [EXPR]
```

### Script

Script commands are denoted by a single `$` for single line scripts, or can be surrounded by `$$$` for a script block. These scripts are run on the system via the C/C++ `system()` call. These are essentially converted to raw-strings. They can contain variables which are used by prefacing the variable name with a single `$` or for better distinction, can be contained within `$(...)`. This is useful if the tail of the variable is beside an alphanumeric character. Arguments passed in from the console can also be accessed using the `$1` style variables. If you wish to reference an environment variable, you can use a double `$$` for the variable reference. The double `$$` will be converted to a single `$` when the script is run.

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

