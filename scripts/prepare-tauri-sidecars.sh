#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
host_triple="$(rustc -vV | awk '/^host: / { print $2 }')"

if [[ -z "${host_triple}" ]]; then
  echo "failed to detect Rust host triple" >&2
  exit 1
fi

profile="${FRAMKEY_SIGNER_HELPER_PROFILE:-}"
if [[ -z "${profile}" ]]; then
  case "${TAURI_ENV_DEBUG:-}" in
    1|true|TRUE|yes|YES)
      profile="debug"
      ;;
    *)
      profile="release"
      ;;
  esac
fi

case "${profile}" in
  debug)
    cargo_args=(build -p framkey-signer-helper -p framkey-ch347-helper)
    target_dir="${repo_root}/target/debug"
    ;;
  release)
    cargo_args=(build --release -p framkey-signer-helper -p framkey-ch347-helper)
    target_dir="${repo_root}/target/release"
    ;;
  *)
    echo "unsupported FRAMKEY_SIGNER_HELPER_PROFILE=${profile}; expected debug or release" >&2
    exit 1
    ;;
esac

(cd "${repo_root}" && cargo "${cargo_args[@]}")

sidecar_dir="${repo_root}/apps/framkey-desktop/src-tauri/binaries"
mkdir -p "${sidecar_dir}"

for helper_name in framkey-signer-helper framkey-ch347-helper; do
  helper_path="${target_dir}/${helper_name}"
  sidecar_path="${sidecar_dir}/${helper_name}-${host_triple}"
  cp "${helper_path}" "${sidecar_path}"
  chmod 755 "${sidecar_path}"
  echo "prepared ${sidecar_path}"
done
