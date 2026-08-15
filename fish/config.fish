
# Starship prompt
if status --is-interactive
   source ("/usr/bin/starship" init fish --print-full-init | psub)

   # Rtfetch
   rtfetch
end

alias desktop='cd ~/Desktop'
desktop

alias delete-screenshots='rm ~/Screenshots/*'
alias delete-downloads='rm -r ~/Downloads/*'
alias delete-trash='gio trash --empty'

alias record='wf-recorder -g "$(slurp)"'

alias ls='exa -al --color=always --group-directories-first' # preferred listing
alias la='exa -a --color=always --group-directories-first'  # all files and dirs
alias ll='exa -l --color=always --group-directories-first'  # long format
alias lt='exa -aT --color=always --group-directories-first' # tree listing

alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'
alias ......='cd ../../../../..'

function clone
        git clone https://github.com/$argv
end
