/// Generate shell integration function for supported shells
pub fn generate_init(shell: &str) -> Result<String, String> {
    match shell {
        "bash" | "zsh" => Ok(r#"gwt() {
    if [ "$1" = "switch" ]; then
        local result
        result=$(command gwtree switch "${@:2}")
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            if [ -d "$result" ]; then
                cd "$result" || return 1
            fi
        else
            echo "$result" >&2
            return $exit_code
        fi
    else
        command gwtree "$@"
    fi
}
"#
        .to_string()),
        "fish" => Ok(r#"function gwt
    if test "$argv[1]" = "switch"
        set result (command gwtree switch (echo $argv | sed 's/^switch //'))
        set exit_code $status
        if test $exit_code -eq 0
            if test -d "$result"
                cd "$result" || return 1
            end
        else
            echo $result >&2
            return $exit_code
        end
    else
        command gwtree $argv
    end
end
"#
        .to_string()),
        _ => Err(format!(
            "Unsupported shell '{}'. Supported: bash, zsh, fish",
            shell
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
    }

    #[test]
    fn generate_fish_init_contains_function() {
        let s = generate_init("fish").unwrap();
        assert!(s.contains("function gwt"));
        assert!(s.contains("command gwtree"));
    }
}
