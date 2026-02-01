# Chat shell integration for zsh
# Add to ~/.zshrc: eval "$(chat init zsh)"

_chat_search() {
    emulate -L zsh
    zle -I

    local output
    output=$(command chat 2>/dev/null)

    if [[ -n "$output" ]]; then
        LBUFFER="$output"
        RBUFFER=""
    fi

    zle reset-prompt
}

zle -N _chat_search

# Bind to Ctrl+G by default (can be customized)
bindkey '^G' _chat_search
