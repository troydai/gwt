use super::{Cli, ShellType};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Generator, Shell, generate};
use std::io;

pub fn handle<W: io::Write>(shell: ShellType, writer: &mut W) -> Result<()> {
    let clap_shell = match shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
        ShellType::Fish => Shell::Fish,
    };

    print_completions(clap_shell, &mut Cli::command(), writer);

    // Print additional dynamic completion functions for branch suggestions
    print_dynamic_completions(shell, writer);

    Ok(())
}

fn print_completions<G: Generator, W: io::Write>(
    generator: G,
    cmd: &mut clap::Command,
    writer: &mut W,
) {
    generate(generator, cmd, cmd.get_name().to_string(), writer);
}

fn print_dynamic_completions<W: io::Write>(shell: ShellType, writer: &mut W) {
    match shell {
        ShellType::Bash => write!(
            writer,
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
        )
        .unwrap(),
        ShellType::Zsh => write!(
            writer,
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
        )
        .unwrap(),
        ShellType::Fish => write!(
            writer,
            r#"
# Dynamic completion for gwt sw command (branch names)
function __gwt_branches
    gwtree ls --raw 2>/dev/null
end

# Complete branch names after 'gwt sw'
complete -c gwt -n '__fish_seen_subcommand_from sw switch' -a '(__gwt_branches)' -d 'branch'
"#
        )
        .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_bash() {
        let mut buf = Vec::new();
        assert!(handle(ShellType::Bash, &mut buf).is_ok());
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_handle_zsh() {
        let mut buf = Vec::new();
        assert!(handle(ShellType::Zsh, &mut buf).is_ok());
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_handle_fish() {
        let mut buf = Vec::new();
        assert!(handle(ShellType::Fish, &mut buf).is_ok());
        assert!(!buf.is_empty());
    }
}
