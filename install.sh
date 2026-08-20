#!/bin/sh
# SPDX-License-Identifier: MIT OR MulanPSL-2.0

set -eu
set -f

prefix=${PREFIX:-/usr/local}
destdir=${DESTDIR:-}
profile=${PROFILE:-debug}
target_dir=${CARGO_TARGET_DIR:-target}

die() {
    printf '%s\n' "lssbi: $*" >&2
    exit 1
}

case $prefix in
    /*) ;;
    *) die "PREFIX must be an absolute path" ;;
esac

case $destdir in
    "" | /*) ;;
    *) die "DESTDIR must be empty or an absolute path" ;;
esac

case $profile in
    "" | *[!A-Za-z0-9_.-]*) die "invalid PROFILE: $profile" ;;
esac

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$script_dir"

binary=$target_dir/$profile/lssbi
locale_root=$target_dir/$profile/locale
linguas=po/LINGUAS

[ -f "$binary" ] || die "missing $binary; run cargo build first"
[ -f "$linguas" ] || die "missing $linguas"

languages=$(awk '{ sub(/\r$/, ""); sub(/#.*/, ""); for (i = 1; i <= NF; i++) print $i }' "$linguas")
for language in $languages; do
    case $language in
        *[!A-Za-z0-9_.@-]*) die "invalid locale in $linguas: $language" ;;
    esac
    catalog=$locale_root/$language/LC_MESSAGES/lssbi.mo
    [ -f "$catalog" ] || die "missing $catalog; run cargo build first"
done

install_dir() {
    install -d -m "$1" "$2"
}

install_file() {
    install -m "$1" "$2" "$3"
}

bin_dir=$destdir$prefix/bin
locale_destination=$destdir$prefix/share/locale

install_dir 0755 "$bin_dir"
install_file 0755 "$binary" "$bin_dir/lssbi"

for language in $languages; do
    source=$locale_root/$language/LC_MESSAGES/lssbi.mo
    destination=$locale_destination/$language/LC_MESSAGES
    install_dir 0755 "$destination"
    install_file 0644 "$source" "$destination/lssbi.mo"
done

printf 'Installed %s\n' "$bin_dir/lssbi"
