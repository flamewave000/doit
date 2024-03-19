# Doit Grammar

```ini
               Node Definitions
         Root: [SOF]      [...]     [EOF]
         Exit: [EXIT]     [EXPR]    [EOL]
         Help: [HELP_BEG] [...]     [HELP_END]
       Assign: [NOMEN]    [ASSIGN]  [EXPR] [EOL]
       Target: [NOMEN]    [TGT_BEG] [...]  [EOL] [TGT_END]
   SLE Target: [NOMEN]    [TGT_SLE] [EXPR] [EOL]
       Script: [SCRIPT]   [EXPR]    [EOL]
      Comment: [COMMENT]  [EXPR]    [EOL]
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

Within a target, you can provide a help block that can be parsed as a special multiline block of text that can be printed for the entire program when a user passes the `-h` or `--help` flags to the program. Help blocks must be the first thing defined, otherwise they will be ignored, and only one will be accepted per scope.

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

Script commands are denoted by a single `$` at the start and are run on the system via the C/C++ `system()` call. These are essentially converted to strings. They can contain variables which are used by prefacing the variable name with a single `$` or for better distinction, can be contained within `$(...)`. This is useful if the tail of the variable is beside an alphanumeric character. Arguments passed in from the console can also be accessed using the `$1` style variables.

```python
$ echo "Hello, World!" | cat
$ echo "My var is: $my_var" > temp.txt
$ echo "WAZZUP!$(var1)DUDE!"
$ echo "Args: $1 $2 $3"
```

### Comment

Comments are denoted by a `#` character and then followed by the comment info. Everything after the `#` to the end of the line will be ignored.

```sh
build { # Comments can occur at the end of any line
	# Or on their own lines
	$ echo "This runs in the shell" # Or at the end of script lines
}
```

