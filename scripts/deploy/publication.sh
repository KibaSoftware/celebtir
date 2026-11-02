#!/usr/bin/env bash
# Source this with `root` set by the caller while holding update.lock.
# Journal before changing any pointer; recover it before the next run.
publication_link() {
  local name=$1 target=$2 temporary="$1.restore.$$"
  if [ "$target" = - ]; then
    if [ -L "$name" ]; then rm -- "$name"; elif [ -e "$name" ]; then return 1; fi
  else
    if [ -L "$temporary" ]; then rm -- "$temporary"; elif [ -e "$temporary" ]; then return 1; fi
    ln -s "$target" "$temporary" || return 1
    mv -Tf "$temporary" "$name" || return 1
  fi
}
publication_validate() {
  local value=$1 parent=$2
  [ "$value" = - ] && return 0
  [ -d "$value" ] && [ "$(dirname "$(readlink -f "$value")")" = "$parent" ]
}
publication_recover() {
  local journal="$root/shared/publication.pending"
  [ -f "$journal" ] || return 0
  local saved=()
  mapfile -t saved < "$journal"
  [ "${#saved[@]}" -eq 6 ] || return 1
  local i
  for i in 0 3; do publication_validate "${saved[$i]}" "$root/shared/generations" || return 1; done
  for i in 1 4; do publication_validate "${saved[$i]}" "$root/web" || return 1; done
  for i in 2 5; do publication_validate "${saved[$i]}" "$root/releases" || return 1; done
  publication_link "$root/shared/data" "${saved[0]}" || return 1
  publication_link "$root/current" "${saved[1]}" || return 1
  publication_link "$root/source" "${saved[2]}" || return 1
  rm -- "$journal"
}
publication_begin() {
  local data=$1 web=$2 src=$3 old_data=- old_web=- old_source=-
  [ ! -e "$root/shared/publication.pending" ] || return 1
  for name in "$root/shared/data" "$root/current" "$root/source"; do
    if [ -e "$name" ] && [ ! -L "$name" ]; then echo "Unmanaged pointer: $name" >&2; return 1; fi
  done
  if [ -L "$root/shared/data" ]; then old_data=$(readlink -f "$root/shared/data"); fi
  if [ -L "$root/current" ]; then old_web=$(readlink -f "$root/current"); fi
  if [ -L "$root/source" ]; then old_source=$(readlink -f "$root/source"); fi
  publication_validate "$old_data" "$root/shared/generations" || return 1
  publication_validate "$old_web" "$root/web" || return 1
  publication_validate "$old_source" "$root/releases" || return 1
  publication_validate "$data" "$root/shared/generations" || return 1
  publication_validate "$web" "$root/web" || return 1
  publication_validate "$src" "$root/releases" || return 1
  printf '%s\n' "$old_data" "$old_web" "$old_source" "$data" "$web" "$src" > "$root/shared/publication.pending.new"
  mv -T "$root/shared/publication.pending.new" "$root/shared/publication.pending"
  publication_link "$root/shared/data" "$data"
  publication_link "$root/current" "$web"
  publication_link "$root/source" "$src"
}
publication_commit() {
  rm -- "$root/shared/publication.pending"
}
