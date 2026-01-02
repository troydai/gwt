use super::{Cli, ShellType};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Generator, Shell, generate};
use std::io;

pub fn handle(shell: ShellType) -> Result<()> {
    let clap_shell = match shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
        ShellType::Fish => Shell::Fish,
    };

    print_completions(clap_shell, &mut Cli::command());

    // Print additional dynamic completion functions for branch suggestions
    print_dynamic_completions(shell);

    Ok(())
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn print_dynamic_completions(shell: ShellType) {
    match shell {
        ShellType::Bash => print!(
            r#"
# Dynamic completion for gwt sw command (branch names)
_gwt_sw_completions() {{
    local branches
    branches=$(gwtree ls --raw 2>/dev/null)
    COMPREPLY=($(compgen -W "$branches" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

# Override the default completion for 'sw' subcommand
_gwt_custom() {{
    local cur prev words cword
    _init_completion || return

    if [[ ${{cword}} -ge 2 && "${{words[1]}}" == "sw" ]]; then
        # Complete branch names for 'gwt sw <branch>'
        _gwt_sw_completions
        return
    fi

    # Fall back to default gwtree completions
    _gwtree "$@"
}}

complete -F _gwt_custom gwt
"#
        ),
        ShellType::Zsh => print!(
            r#"
# Dynamic completion for gwt sw command (branch names)
_gwt_branches() {{
    local branches
    branches=(${{(f)"$(gwtree ls --raw 2>/dev/null)"}})
    _describe 'branch' branches
}}

# Custom completion for gwt wrapper function
compdef _gwt_wrapper gwt

_gwt_wrapper() {{
    local line state

    _arguments -C \
        '1: :->command' \
        '*: :->args'

    case $state in
        command)
            _gwtree
            ;;
        args)
            case $line[1] in
                sw|switch)
                    _gwt_branches
                    ;;
                *)
                    _gwtree
                    ;;
            esac
            ;;
    esac
}}
"#
        ),
        ShellType::Fish => print!(
            r#"
# Dynamic completion for gwt sw command (branch names)
function __gwt_branches
    gwtree ls --raw 2>/dev/null
end

# Complete branch names after 'gwt sw'
complete -c gwt -n '__fish_seen_subcommand_from sw switch' -a '(__gwt_branches)' -d 'branch'
"#
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_bash() {
        // Just ensure it doesn't panic
        assert!(handle(ShellType::Bash).is_ok());
    }

    #[test]
    fn test_handle_zsh() {
        assert!(handle(ShellType::Zsh).is_ok());
    }

    #[test]
    fn test_handle_fish() {
        assert!(handle(ShellType::Fish).is_ok());
    }
}
