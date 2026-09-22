#!/bin/sh
# Fedora preview bootstrap. Installs per user; never enables login startup.
set -eu

version=${DECKSMITH_VERSION:-v0.1.0-preview.1}
case "$version" in ''|*[!A-Za-z0-9._-]*) echo 'Invalid release version.' >&2; exit 1;; esac
if [ "$(id -u)" -eq 0 ]; then
    echo 'Run this as your regular desktop user, not with sudo.' >&2
    exit 1
fi
if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
    echo 'This preview requires Linux x86_64.' >&2
    exit 1
fi
. /etc/os-release
if [ "${ID:-}" != fedora ]; then
    echo 'This installer supports Fedora. See the source-build guide for other distributions.' >&2
    exit 1
fi
case "${VERSION_ID:-}" in
    44) ;;
    45) echo 'Fedora 45: this preview is primarily validated on Fedora 44.';;
    *) echo 'This preview requires Fedora 44 or 45.' >&2; exit 1;;
esac
for tool in curl sha256sum rpm dnf; do
    command -v "$tool" >/dev/null 2>&1 || { echo "Required command is missing: $tool" >&2; exit 1; }
done

echo "Installing Decksmith $version for the current user."
echo 'Saved layouts are preserved. Background controls and login startup are not enabled.'
# DNF requests its own confirmation; the Python installer is never run as root.
missing=''
for package in python3 python3-gobject python3-pillow python3-cairo gtk4 libadwaita librsvg2 glib2 systemd systemd-libs pulseaudio-libs pulseaudio-utils wireplumber; do
    if ! rpm -q "$package" >/dev/null 2>&1; then missing="$missing $package"; fi
done
if [ -n "$missing" ]; then
    echo "Fedora dependencies to install:$missing"
    if [ ! -r /dev/tty ]; then
        echo "Run sudo dnf install$missing, then retry this installer." >&2
        exit 1
    fi
    # Package names come only from the fixed list above.
    # shellcheck disable=SC2086
    sudo dnf install $missing </dev/tty
fi

umask 077
temporary=$(mktemp -d "${TMPDIR:-/tmp}/decksmith-install.XXXXXXXX")
trap 'rm -rf -- "$temporary"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP
cd "$temporary"
base="https://github.com/infamous-pattern/decksmith/releases/download/$version"
for file in decksmith-linux-x86_64.tar.gz decksmith-install.py package_io.py INSTALL.md SHA256SUMS; do
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
        --connect-timeout 10 --max-time 180 --retry 3 "$base/$file" --output "$file"
done
sha256sum --check --strict SHA256SUMS
python3 -B decksmith-install.py install "$temporary/decksmith-linux-x86_64.tar.gz"
echo 'Decksmith is installed. Open it from your application menu.'
echo 'Close other Stream Deck controllers, then choose Start under Background controls.'
