#!/bin/sh
# SPDX-License-Identifier: MIT OR MulanPSL-2.0

set -eu
set -f

prefix=${PREFIX:-/usr/local}
destdir=${DESTDIR:-}
profile=${PROFILE:-debug}
admin_group=${ADMIN_GROUP:-sudo}
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

if [ -z "$destdir" ] && [ "$(id -u)" -ne 0 ]; then
    die "run as root, or set DESTDIR for a staged installation"
fi

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$script_dir"

binary=$target_dir/$profile/lssbi
locale_root=$target_dir/$profile/locale
pam_source=pam/lssbi
linguas=po/LINGUAS

[ -f "$binary" ] || die "missing $binary; run cargo build first"
[ -f "$pam_source" ] || die "missing $pam_source"
[ -f "$linguas" ] || die "missing $linguas"

languages=$(awk '{ sub(/#.*/, ""); for (i = 1; i <= NF; i++) print $i }' "$linguas")
for language in $languages; do
    case $language in
        *[!A-Za-z0-9_.@-]*) die "invalid locale in $linguas: $language" ;;
    esac
    catalog=$locale_root/$language/LC_MESSAGES/lssbi.mo
    [ -f "$catalog" ] || die "missing $catalog; run cargo build first"
done

install_dir() {
    mode=$1
    path=$2
    if [ -d "$path" ]; then
        return
    fi
    if [ -z "$destdir" ]; then
        install -d -o root -g root -m "$mode" "$path"
    else
        install -d -m "$mode" "$path"
    fi
}

install_file() {
    mode=$1
    group=$2
    source=$3
    destination=$4
    if [ -z "$destdir" ]; then
        install -o root -g "$group" -m "$mode" "$source" "$destination"
    else
        install -m "$mode" "$source" "$destination"
    fi
}

sbin_dir=$destdir$prefix/sbin
locale_destination=$destdir$prefix/share/locale
pam_destination=$destdir/etc/pam.d

if [ -z "$destdir" ] && [ ! -d "$pam_destination" ]; then
    die "$pam_destination does not exist"
fi

install_dir 0755 "$sbin_dir"
if [ -n "$destdir" ]; then
    install_dir 0755 "$pam_destination"
fi

install_file 0644 root "$pam_source" "$pam_destination/lssbi"

for language in $languages; do
    source=$locale_root/$language/LC_MESSAGES/lssbi.mo
    destination=$locale_destination/$language/LC_MESSAGES
    install_dir 0755 "$destination"
    install_file 0644 root "$source" "$destination/lssbi.mo"
done

install_file 0750 "$admin_group" "$binary" "$sbin_dir/lssbi"
chmod 4750 "$sbin_dir/lssbi"
printf 'Installed %s\n' "$sbin_dir/lssbi"
