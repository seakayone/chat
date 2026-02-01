# Chat shell integration for fish
# Add to ~/.config/fish/config.fish: chat init fish | source

function _chat_search
    set -l output (command chat 2>/dev/null)

    if test -n "$output"
        commandline -r "$output"
        commandline -f repaint
    end
end

# Bind to Ctrl+G by default (can be customized)
bind \cg _chat_search
