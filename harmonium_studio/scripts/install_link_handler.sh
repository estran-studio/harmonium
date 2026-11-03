#!/bin/sh
# Enregistre le schéma `harmonium://` : un clic sur un lien harmonium://open?… appelle
#   <binaire du studio> open '<lien>'
# qui transmet le lien au studio qui tourne, ou le lance.
#
#   install_link_handler.sh [--dest <dossier>] [--no-register] <binaire du studio>
#   install_link_handler.sh --uninstall [--dest <dossier>]
#
# macOS : une petite appli (`Harmonium Link Handler.app`, AppleScript `on open location`, compilée
#         par osacompile) dans ~/Applications. macOS remet l'URL par Apple Event, pas par argv :
#         l'appli la reçoit et appelle la commande. Pas d'icône dans le Dock (LSUIElement).
# Linux : un `harmonium-studio-link.desktop` dans ~/.local/share/applications, déclaré gestionnaire
#         de x-scheme-handler/harmonium par xdg-mime.
# Windows : voir harmonium_studio/README.md (clé de registre à la main).
#
# --dest : autre dossier d'installation ; --no-register : n'enregistre rien auprès du système
# (pour contrôler le résultat sans changer les réglages de l'utilisateur).

set -eu

dest=""
register=1
uninstall=0
binary=""
while [ $# -gt 0 ]; do
    case "$1" in
        --dest) dest="${2:?--dest needs a folder}"; shift 2 ;;
        --no-register) register=0; shift ;;
        --uninstall) uninstall=1; shift ;;
        -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
        -*) echo "unknown option $1" >&2; exit 2 ;;
        *) binary="$1"; shift ;;
    esac
done

if [ "$uninstall" -eq 0 ]; then
    [ -n "$binary" ] || { echo "usage: $0 [--dest <dir>] [--no-register] <studio binary>" >&2; exit 2; }
    [ -x "$binary" ] || { echo "$binary is not an executable (build it: cargo build --release -p harmonium_studio)" >&2; exit 1; }
    case "$binary" in
        /*) ;;
        *) binary="$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")" ;;
    esac
    # Le chemin est écrit dans un script AppleScript ou une ligne Exec : pas de caractère à échapper.
    case "$binary" in
        *\"*|*\\*|*\`*|*\$*|*%*) echo "unsupported character in $binary (one of \" \\ \` \$ %)" >&2; exit 1 ;;
    esac
fi

install_macos() {
    dest="${dest:-$HOME/Applications}"
    app="$dest/Harmonium Link Handler.app"
    lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
    if [ "$uninstall" -eq 1 ]; then
        [ -d "$app" ] && [ "$register" -eq 1 ] && "$lsregister" -u "$app" 2>/dev/null || true
        rm -rf "$app"
        echo "removed $app"
        return
    fi
    mkdir -p "$dest"
    rm -rf "$app"
    script="$(mktemp -t harmonium-link-handler).applescript"
    # `quoted form of` protège les espaces et les apostrophes du chemin et du lien.
    cat > "$script" <<EOF
on open location theURL
	set studio to "$binary"
	do shell script (quoted form of studio) & " open " & (quoted form of theURL) & " >/dev/null 2>&1 &"
end open location
EOF
    osacompile -o "$app" "$script"
    rm -f "$script"
    plist="$app/Contents/Info.plist"
    pb=/usr/libexec/PlistBuddy
    "$pb" -c "Delete :CFBundleURLTypes" "$plist" 2>/dev/null || true
    "$pb" -c "Add :CFBundleURLTypes array" "$plist"
    "$pb" -c "Add :CFBundleURLTypes:0 dict" "$plist"
    "$pb" -c "Add :CFBundleURLTypes:0:CFBundleURLName string 'Harmonium link'" "$plist"
    "$pb" -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes array" "$plist"
    "$pb" -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string harmonium" "$plist"
    "$pb" -c "Delete :LSUIElement" "$plist" 2>/dev/null || true
    "$pb" -c "Add :LSUIElement bool true" "$plist"
    # Modifier Info.plist invalide la signature ad hoc de l'applet : la refaire.
    codesign --force --deep --sign - "$app" 2>/dev/null || true
    if [ "$register" -eq 1 ]; then
        "$lsregister" -f "$app"
    fi
    echo "installed $app -> $binary"
}

install_linux() {
    dest="${dest:-${XDG_DATA_HOME:-$HOME/.local/share}/applications}"
    desktop="$dest/harmonium-studio-link.desktop"
    if [ "$uninstall" -eq 1 ]; then
        rm -f "$desktop"
        [ "$register" -eq 1 ] && command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$dest" || true
        echo "removed $desktop"
        return
    fi
    mkdir -p "$dest"
    # Exec : le chemin entre guillemets (il peut contenir des espaces), %u : l'URL.
    cat > "$desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Harmonium Studio link handler
Comment=Opens harmonium:// links in the studio
Exec="$binary" open %u
Terminal=false
NoDisplay=true
MimeType=x-scheme-handler/harmonium;
EOF
    if [ "$register" -eq 1 ]; then
        command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$dest" || true
        xdg-mime default harmonium-studio-link.desktop x-scheme-handler/harmonium
    fi
    echo "installed $desktop -> $binary"
}

case "$(uname -s)" in
    Darwin) install_macos ;;
    Linux) install_linux ;;
    *) echo "unsupported system $(uname -s): see harmonium_studio/README.md (Windows)" >&2; exit 1 ;;
esac
