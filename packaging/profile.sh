# Installed as /etc/profile.d/sne.sh by the sne package.
# Puts the default Node.js version managed by sne on PATH for login shells, so `node` works
# without any setup. For per-terminal switching, see `sne env --help`.
sne_default="${SNE_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/sne}/aliases/default/bin"
case ":$PATH:" in
    *":$sne_default:"*) ;;
    *) PATH="$sne_default:$PATH"; export PATH ;;
esac
unset sne_default
