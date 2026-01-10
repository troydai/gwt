use anyhow::{Result, anyhow};

pub fn handle(shell: &str) -> Result<()> {
    println!("{}", generate_init(shell)?);
    Ok(())
}

fn generate_init(shell: &str) -> Result<String> {
    match shell {
        "bash" => Ok(r#"gwt() {
    if [ "$1" = "switch" ] || [ "$1" = "sw" ]; then
        for arg in "$@"; do
            if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
                command gwtree "$@"
                return
            fi
        done
        local result
        result=$(command gwtree sw "${@:2}")
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            else
                printf "%s\n" "$result"
            fi
        else
            printf "%s\n" "$result" >&2
            return $exit_code
        fi
    elif [ "$1" = "remove" ] || [ "$1" = "rm" ]; then
        for arg in "$@"; do
            if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
                command gwtree "$@"
                return
            fi
        done
        local result
        result=$(command gwtree rm "${@:2}")
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            fi
        else
            printf "%s\n" "$result" >&2
            return $exit_code
        fi
    elif [ "$1" = "home" ]; then
        for arg in "$@"; do
            if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
                command gwtree "$@"
                return
            fi
        done
        local result
        result=$(command gwtree home)
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            else
                printf "%s\n" "$result"
            fi
        else
            printf "%s\n" "$result" >&2
            return $exit_code
        fi
    else
        command gwtree "$@"
    fi
}

# Tab completion for gwt
_gwt_completions() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Top-level commands
    local commands="config ls sw rm init current completion home"

    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return
    fi

    # Complete branch names for 'sw' and 'rm' commands
    case "${COMP_WORDS[1]}" in
        sw|switch)
            local branches
            branches=$(command gwtree ls --raw 2>/dev/null)
            COMPREPLY=($(compgen -W "$branches" -- "$cur"))
            ;;
        rm|remove)
            local branches
            branches=$(command gwtree ls --raw 2>/dev/null)
            COMPREPLY=($(compgen -W "$branches" -- "$cur"))
            ;;
        init|completion)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur"))
            ;;
        config)
            COMPREPLY=($(compgen -W "view setup" -- "$cur"))
            ;;
    esac
}

complete -F _gwt_completions gwt
"#
        .to_string()),
        "zsh" => Ok(r#"gwt() {
    if [ "$1" = "switch" ] || [ "$1" = "sw" ]; then
        for arg in "$@"; do
            if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
                command gwtree "$@"
                return
            fi
        done
        local result
        result=$(command gwtree sw "${@:2}")
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            else
                printf "%s\n" "$result"
            fi
        else
            printf "%s\n" "$result" >&2
            return $exit_code
        fi
    elif [ "$1" = "remove" ] || [ "$1" = "rm" ]; then
        for arg in "$@"; do
            if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
                command gwtree "$@"
                return
            fi
        done
        local result
        result=$(command gwtree rm "${@:2}")
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            fi
        else
            printf "%s\n" "$result" >&2
            return $exit_code
        fi
    elif [ "$1" = "home" ]; then
        for arg in "$@"; do
            if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
                command gwtree "$@"
                return
            fi
        done
        local result
        result=$(command gwtree home)
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            else
                printf "%s\n" "$result"
            fi
        else
            printf "%s\n" "$result" >&2
            return $exit_code
        fi
    else
        command gwtree "$@"
    fi
}

# Tab completion for gwt (zsh)
_gwt() {
    local -a commands branches shells config_commands
    commands=(
        'config:Configure gwt'
        'ls:List all worktrees'
        'sw:Switch to an existing worktree'
        'rm:Remove a worktree by branch name'
        'init:Output shell integration code'
        'current:Print current worktree and branch information'
        'completion:Generate shell completion scripts'
        'home:Switch to the home worktree (original repository)'
    )
    shells=('bash' 'zsh' 'fish')
    config_commands=('view' 'setup')

    if (( CURRENT == 2 )); then
        _describe 'command' commands
    else
        case "${words[2]}" in
            sw|switch|rm|remove)
                branches=(${(f)"$(command gwtree ls --raw 2>/dev/null)"})
                _describe 'branch' branches
                ;;
            init|completion)
                _describe 'shell' shells
                ;;
            config)
                _describe 'subcommand' config_commands
                ;;
        esac
    fi
}

compdef _gwt gwt
"#
        .to_string()),
        "fish" => Ok(r#"function gwt
    if test "$argv[1]" = "switch" -o "$argv[1]" = "sw"
        for arg in $argv
            if test "$arg" = "--help" -o "$arg" = "-h"
                command gwtree $argv
                return
            end
        end
        set result (command gwtree sw $argv[2..-1])
        set exit_code $status
        if test $exit_code -eq 0
            if test -d "$result"
                cd "$result" || return 1
            else
                printf "%s\n" $result
            end
        else
            printf "%s\n" $result >&2
            return $exit_code
        end
    else if test "$argv[1]" = "remove" -o "$argv[1]" = "rm"
        for arg in $argv
            if test "$arg" = "--help" -o "$arg" = "-h"
                command gwtree $argv
                return
            end
        end
        set result (command gwtree rm $argv[2..-1])
        set exit_code $status
        if test $exit_code -eq 0
            if test -d "$result"
                cd "$result" || return 1
            end
        else
            printf "%s\n" $result >&2
            return $exit_code
        end
    else if test "$argv[1]" = "home"
        for arg in $argv
            if test "$arg" = "--help" -o "$arg" = "-h"
                command gwtree $argv
                return
            end
        end
        set result (command gwtree home)
        set exit_code $status
        if test $exit_code -eq 0
            if test -d "$result"
                cd "$result" || return 1
            else
                printf "%s\n" $result
            end
        else
            printf "%s\n" $result >&2
            return $exit_code
        end
    else
        command gwtree $argv
    end
end

# Tab completion for gwt (fish)
function __gwt_branches
    command gwtree ls --raw 2>/dev/null
end

function __gwt_needs_command
    set -l cmd (commandline -opc)
    test (count $cmd) -eq 1
end

function __gwt_using_command
    set -l cmd (commandline -opc)
    test (count $cmd) -gt 1; and test "$cmd[2]" = "$argv[1]"
end

# Disable file completions for gwt
complete -c gwt -f

# Top-level commands
complete -c gwt -n '__gwt_needs_command' -a 'config' -d 'Configure gwt'
complete -c gwt -n '__gwt_needs_command' -a 'ls' -d 'List all worktrees'
complete -c gwt -n '__gwt_needs_command' -a 'sw' -d 'Switch to an existing worktree'
complete -c gwt -n '__gwt_needs_command' -a 'rm' -d 'Remove a worktree by branch name'
complete -c gwt -n '__gwt_needs_command' -a 'init' -d 'Output shell integration code'
complete -c gwt -n '__gwt_needs_command' -a 'current' -d 'Print current worktree and branch information'
complete -c gwt -n '__gwt_needs_command' -a 'completion' -d 'Generate shell completion scripts'
complete -c gwt -n '__gwt_needs_command' -a 'home' -d 'Switch to the home worktree (original repository)'

# Branch completions for sw and rm
complete -c gwt -n '__gwt_using_command sw' -a '(__gwt_branches)' -d 'branch'
complete -c gwt -n '__gwt_using_command rm' -a '(__gwt_branches)' -d 'branch'

# Shell completions for init and completion
complete -c gwt -n '__gwt_using_command init' -a 'bash zsh fish'
complete -c gwt -n '__gwt_using_command completion' -a 'bash zsh fish'

# Config subcommands
complete -c gwt -n '__gwt_using_command config' -a 'view setup'
"#
        .to_string()),
        _ => Err(anyhow!(
            "Unsupported shell '{shell}'. Supported: bash, zsh, fish",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_bash_init_contains_function() {
        let s = generate_init("bash").unwrap();
        assert!(s.contains("gwt() {"));
        assert!(s.contains("command gwtree"));
        assert!(s.contains(r#"[ "$1" = "switch" ] || [ "$1" = "sw" ]"#));
        assert!(s.contains(r#"[ "$1" = "remove" ] || [ "$1" = "rm" ]"#));
    }

    #[test]
    fn generate_bash_init_contains_completion() {
        let s = generate_init("bash").unwrap();
        assert!(s.contains("_gwt_completions"));
        assert!(s.contains("complete -F _gwt_completions gwt"));
        assert!(s.contains("gwtree ls --raw"));
    }

    #[test]
    fn generate_zsh_init_contains_function() {
        let s = generate_init("zsh").unwrap();
        assert!(s.contains("gwt() {"));
        assert!(s.contains("command gwtree"));
        assert!(s.contains(r#"[ "$1" = "switch" ] || [ "$1" = "sw" ]"#));
        assert!(s.contains(r#"[ "$1" = "remove" ] || [ "$1" = "rm" ]"#));
    }

    #[test]
    fn generate_zsh_init_contains_completion() {
        let s = generate_init("zsh").unwrap();
        assert!(s.contains("_gwt()"));
        assert!(s.contains("compdef _gwt gwt"));
        assert!(s.contains("gwtree ls --raw"));
    }

    #[test]
    fn generate_fish_init_contains_function() {
        let s = generate_init("fish").unwrap();
        assert!(s.contains("function gwt"));
        assert!(s.contains("command gwtree"));
        assert!(s.contains(r#"test "$argv[1]" = "switch" -o "$argv[1]" = "sw""#));
        assert!(s.contains(r#"test "$argv[1]" = "remove" -o "$argv[1]" = "rm""#));
    }

    #[test]
    fn generate_fish_init_contains_completion() {
        let s = generate_init("fish").unwrap();
        assert!(s.contains("__gwt_branches"));
        assert!(s.contains("complete -c gwt"));
        assert!(s.contains("gwtree ls --raw"));
    }

    #[test]
    fn generate_init_unsupported_shell() {
        let result = generate_init("powershell");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unsupported shell 'powershell'. Supported: bash, zsh, fish"
        );
    }
}
