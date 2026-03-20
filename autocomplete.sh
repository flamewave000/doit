#!/bin/bash
_doit_completions() {
    # Current word being completed
    local cur="${COMP_WORDS[COMP_CWORD]}"
    # Ask program for list of valid targets
    local opts=$(doit -f --targets)
    # Generate completions
    COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
}
complete -F _doit_completions doit