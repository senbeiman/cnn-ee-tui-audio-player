#!/usr/bin/env bash
set -euo pipefail

APP_NAME="CNN EE Audio Player"
BINARY_NAME="play-cnnee"
APP_PATH="target/release/bundle/osx/${APP_NAME}.app"
RESOURCE_BINARY="${APP_PATH}/Contents/Resources/${BINARY_NAME}-bin"
LAUNCHER_PATH="${APP_PATH}/Contents/MacOS/${BINARY_NAME}"
TERMINAL_COMMAND_PATH="${APP_PATH}/Contents/Resources/start.command"
APPLICATIONS_PATH="/Applications/${APP_NAME}.app"

cargo bundle --release

if [[ ! -d "${APP_PATH}" ]]; then
  echo "Expected app bundle was not created: ${APP_PATH}" >&2
  exit 1
fi

cp "target/release/${BINARY_NAME}" "${RESOURCE_BINARY}"

cat >"${TERMINAL_COMMAND_PATH}" <<'COMMAND'
#!/usr/bin/env bash
set -euo pipefail

APP_CONTENTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
exec "${APP_CONTENTS_DIR}/Resources/play-cnnee-bin"
COMMAND

cat >"${LAUNCHER_PATH}" <<'LAUNCHER'
#!/usr/bin/env bash
set -euo pipefail

APP_CONTENTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMMAND_FILE="${APP_CONTENTS_DIR}/Resources/start.command"

exec /usr/bin/open "${COMMAND_FILE}"
LAUNCHER

chmod +x "${LAUNCHER_PATH}" "${TERMINAL_COMMAND_PATH}"

rm -rf "${APPLICATIONS_PATH}"
cp -R "${APP_PATH}" "${APPLICATIONS_PATH}"

echo "Installed ${APPLICATIONS_PATH}"
