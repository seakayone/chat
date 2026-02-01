# Chat shell integration for bash
# Add to ~/.bashrc: eval "$(chat init bash)"

_chat_search() {
    local output
    output=$(command chat 2>/dev/null)

    if [[ -n "$output" ]]; then
        READLINE_LINE="$output"
        READLINE_POINT=${#output}
    fi
}

# Bind to Ctrl+G by default (can be customized)
bind -x '"\C-g": _chat_search'
