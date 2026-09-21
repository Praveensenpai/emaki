use crate::error::{EmakiError, Result};

pub struct CompletionGenerator;

impl CompletionGenerator {
    pub fn generate(shell: &str) -> Result<String> {
        match shell {
            "bash" => Ok(Self::bash()),
            "zsh" => Ok(Self::zsh()),
            "fish" => Ok(Self::fish()),
            unknown => Err(EmakiError::Config(format!(
                "Unsupported shell: '{unknown}'. Supported: bash, zsh, fish"
            ))),
        }
    }

    fn bash() -> String {
        r#"_emaki() {
    local cur prev words cword
    _init_completion 2>/dev/null || {
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    }

    local commands="login daemon run stop logs status config whitelist service completion help"
    local config_actions="show set list"
    local config_keys="max_file_size_mb max_cache_size_gb caption_prefix temp_dir cache_dir session_db"
    local whitelist_actions="list show add remove rm"
    local service_actions="install uninstall remove status start stop restart logs"
    local shells="bash zsh fish"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
        return 0
    fi

    case "${words[1]}" in
        config)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$config_actions" -- "$cur") )
            elif [[ $cword -eq 3 && "${words[2]}" == "set" ]]; then
                COMPREPLY=( $(compgen -W "$config_keys" -- "$cur") )
            fi
            ;;
        whitelist)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$whitelist_actions" -- "$cur") )
            fi
            ;;
        service)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$service_actions" -- "$cur") )
            fi
            ;;
        daemon)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--foreground -f" -- "$cur") )
            fi
            ;;
        completion)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$shells" -- "$cur") )
            fi
            ;;
    esac
}
complete -F _emaki emaki
"#
        .to_string()
    }

    fn zsh() -> String {
        r#"#compdef emaki

_emaki() {
    local -a commands
    commands=(
        'login:Start interactive QR pairing to authenticate with WhatsApp'
        'daemon:Start the reel daemon in background (use -f for foreground)'
        'stop:Stop the background daemon'
        'logs:Follow live background daemon logs'
        'status:Check login credentials, background daemon state, and cache'
        'config:View or update configuration settings'
        'whitelist:Manage whitelisted WhatsApp group JIDs'
        'service:Manage 24/7 background systemd service'
        'completion:Generate shell autocompletion script'
        'help:Display help menu'
    )

    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'emaki command' commands
            ;;
        args)
            case $words[1] in
                config)
                    if (( CURRENT == 2 )); then
                        local -a config_cmds
                        config_cmds=('show:Display current config' 'set:Update a config value' 'list:List config')
                        _describe -t config_cmds 'config action' config_cmds
                    elif (( CURRENT == 3 )) && [[ $words[2] == "set" ]]; then
                        local -a keys
                        keys=('max_file_size_mb:Max download size in MB' 'max_cache_size_gb:Max cache size in GB' 'caption_prefix:Custom caption' 'temp_dir:Temp directory' 'cache_dir:Cache directory' 'session_db:Session DB path')
                        _describe -t keys 'config key' keys
                    fi
                    ;;
                whitelist)
                    local -a wl_cmds
                    wl_cmds=('list:List whitelisted groups' 'add:Add a group JID' 'remove:Remove a group JID')
                    _describe -t wl_cmds 'whitelist action' wl_cmds
                    ;;
                service)
                    local -a srv_cmds
                    srv_cmds=('install:Install and start systemd user service' 'status:Show service status' 'logs:Follow service logs' 'restart:Restart service' 'start:Start service' 'stop:Stop service' 'uninstall:Uninstall service')
                    _describe -t srv_cmds 'service action' srv_cmds
                    ;;
                daemon)
                    _arguments '--foreground[Run in foreground]' '-f[Run in foreground]'
                    ;;
                completion)
                    local -a shells
                    shells=('bash:Generate Bash completion' 'zsh:Generate Zsh completion' 'fish:Generate Fish completion')
                    _describe -t shells 'shell' shells
                    ;;
            esac
            ;;
    esac
}

_emaki "$@"
"#
        .to_string()
    }

    fn fish() -> String {
        r#"complete -c emaki -f

complete -c emaki -n "__fish_use_subcommand" -a login -d "Start interactive QR pairing to authenticate with WhatsApp"
complete -c emaki -n "__fish_use_subcommand" -a daemon -d "Start the reel daemon in background"
complete -c emaki -n "__fish_use_subcommand" -a stop -d "Stop the background daemon"
complete -c emaki -n "__fish_use_subcommand" -a logs -d "Follow live background daemon logs"
complete -c emaki -n "__fish_use_subcommand" -a status -d "Check login credentials, daemon state, and cache"
complete -c emaki -n "__fish_use_subcommand" -a config -d "View or update configuration settings"
complete -c emaki -n "__fish_use_subcommand" -a whitelist -d "Manage whitelisted WhatsApp group JIDs"
complete -c emaki -n "__fish_use_subcommand" -a service -d "Manage 24/7 background systemd service"
complete -c emaki -n "__fish_use_subcommand" -a completion -d "Generate shell completion script"
complete -c emaki -n "__fish_use_subcommand" -a help -d "Display help menu"

complete -c emaki -n "__fish_seen_subcommand_from daemon" -l foreground -s f -d "Run in foreground"
complete -c emaki -n "__fish_seen_subcommand_from config" -a "show set list"
complete -c emaki -n "__fish_seen_subcommand_from whitelist" -a "list add remove"
complete -c emaki -n "__fish_seen_subcommand_from service" -a "install status logs restart stop start uninstall"
complete -c emaki -n "__fish_seen_subcommand_from completion" -a "bash zsh fish"
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_generation() {
        let bash = CompletionGenerator::generate("bash").unwrap();
        assert!(bash.contains("_emaki"));
        assert!(bash.contains("complete -F _emaki emaki"));

        let zsh = CompletionGenerator::generate("zsh").unwrap();
        assert!(zsh.contains("#compdef emaki"));
        assert!(zsh.contains("_emaki"));

        let fish = CompletionGenerator::generate("fish").unwrap();
        assert!(fish.contains("complete -c emaki"));

        assert!(CompletionGenerator::generate("unknown_shell").is_err());
    }
}
