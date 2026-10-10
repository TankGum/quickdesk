# QuickDesk: make version changes reach terminals that are already open.
# Loaded from the QuickDesk block in ~/.bashrc or ~/.zshrc; turn it off in
# QuickDesk (Runtimes → "Apply right away in open terminals") or delete that line.
#
# At each prompt, with no extra process unless something changed:
# - after QuickDesk changed a version, forget remembered command paths
#   (`hash -r`), so a newly added python3, go… is found;
# - with nvm, use the nearest .nvmrc, else nvm's default alias.

__quickdesk_nvm() {
  command -v nvm >/dev/null 2>&1 || return 0
  local dir="$PWD" rc="" want=""
  while [ -n "$dir" ]; do
    if [ -f "$dir/.nvmrc" ]; then rc="$dir/.nvmrc"; break; fi
    dir="${dir%/*}"
  done
  if [ -n "$rc" ]; then
    want="$(<"$rc")"
  elif [ -f "${NVM_DIR:-$HOME/.nvm}/alias/default" ]; then
    want="$(<"${NVM_DIR:-$HOME/.nvm}/alias/default")"
  fi
  # A manual `nvm use` stays until the folder's pin or the default changes.
  [ "$rc|$want" = "${__QUICKDESK_NVM_KEY-}" ] && return 0
  __QUICKDESK_NVM_KEY="$rc|$want"
  [ -n "$want" ] || return 0
  if ! nvm use --silent "$want" >/dev/null 2>&1 && [ -n "$rc" ]; then
    echo "QuickDesk: Node $want from $rc is not installed; run: nvm install $want" >&2
  fi
}

__quickdesk_prompt() {
  local changed="" file="${XDG_CONFIG_HOME:-$HOME/.config}/quickdesk/shell/changed"
  [ -f "$file" ] && changed="$(<"$file")"
  if [ "$changed" != "${__QUICKDESK_CHANGED-}" ]; then
    __QUICKDESK_CHANGED="$changed"
    hash -r 2>/dev/null
  fi
  __quickdesk_nvm
}

if [ -n "${ZSH_VERSION-}" ]; then
  autoload -Uz add-zsh-hook && add-zsh-hook precmd __quickdesk_prompt
elif [ -n "${BASH_VERSION-}" ]; then
  case ";${PROMPT_COMMAND-};" in
    *";__quickdesk_prompt;"*) ;;
    *) PROMPT_COMMAND="__quickdesk_prompt${PROMPT_COMMAND:+;$PROMPT_COMMAND}" ;;
  esac
fi
