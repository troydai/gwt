use anyhow::{Result, anyhow};

pub fn handle(shell: &str) -> Result<()> {
    println!("{}", generate_init(shell)?);
    Ok(())
}

fn generate_init(shell: &str) -> Result<String> {
    match shell {
        "bash" | "zsh" => Ok(r#"gwt() {
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
    else
        command gwtree "$@"
    fi
}
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
    else
        command gwtree $argv
    end
end
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
    fn generate_zsh_init_contains_function() {
        let s = generate_init("zsh").unwrap();
        assert!(s.contains("gwt() {"));
        assert!(s.contains("command gwtree"));
        assert!(s.contains(r#"[ "$1" = "switch" ] || [ "$1" = "sw" ]"#));
        assert!(s.contains(r#"[ "$1" = "remove" ] || [ "$1" = "rm" ]"#));
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
    fn generate_init_unsupported_shell() {
        let result = generate_init("powershell");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unsupported shell 'powershell'. Supported: bash, zsh, fish"
        );
    }
}
