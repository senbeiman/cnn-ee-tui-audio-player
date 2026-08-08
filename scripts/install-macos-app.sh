#!/usr/bin/env bash
set -euo pipefail

APP_NAME="CNN EE Audio Player"
BINARY_NAME="play-cnnee"
APP_PATH="target/release/bundle/osx/${APP_NAME}.app"
RESOURCE_BINARY="${APP_PATH}/Contents/Resources/${BINARY_NAME}-bin"
LAUNCHER_PATH="${APP_PATH}/Contents/MacOS/${BINARY_NAME}"
TERMINAL_SETTINGS_PATH="${APP_PATH}/Contents/Resources/${APP_NAME}.terminal"
START_SCRIPT_PATH="${APP_PATH}/Contents/Resources/start.sh"
APPLICATIONS_PATH="/Applications/${APP_NAME}.app"
INSTALLED_START_SCRIPT="${APPLICATIONS_PATH}/Contents/Resources/start.sh"

cargo bundle --release

if [[ ! -d "${APP_PATH}" ]]; then
  echo "Expected app bundle was not created: ${APP_PATH}" >&2
  exit 1
fi

cp "target/release/${BINARY_NAME}" "${RESOURCE_BINARY}"
cp "/System/Applications/Utilities/Terminal.app/Contents/Resources/Initial Settings/Basic.terminal" "${TERMINAL_SETTINGS_PATH}"

cat >"${START_SCRIPT_PATH}" <<'START'
#!/usr/bin/env bash
set -euo pipefail

APP_CONTENTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
exec "${APP_CONTENTS_DIR}/Resources/play-cnnee-bin"
START

plutil -replace name -string "${APP_NAME}" "${TERMINAL_SETTINGS_PATH}"
plutil -replace CommandString -string "/bin/bash '${INSTALLED_START_SCRIPT}'" "${TERMINAL_SETTINGS_PATH}"
plutil -replace RunCommandAsShell -bool false "${TERMINAL_SETTINGS_PATH}"
plutil -replace WindowTitle -string "${APP_NAME}" "${TERMINAL_SETTINGS_PATH}"
plutil -replace ShowWindowSettingsNameInTitle -bool false "${TERMINAL_SETTINGS_PATH}"
plutil -replace ShowShellCommandInTitle -bool false "${TERMINAL_SETTINGS_PATH}"
plutil -replace ShowActiveProcessInTitle -bool false "${TERMINAL_SETTINGS_PATH}"
plutil -replace ShowTTYNameInTitle -bool false "${TERMINAL_SETTINGS_PATH}"
plutil -replace ShowDimensionsInTitle -bool false "${TERMINAL_SETTINGS_PATH}"
plutil -replace columnCount -integer 60 "${TERMINAL_SETTINGS_PATH}"
plutil -replace rowCount -integer 10 "${TERMINAL_SETTINGS_PATH}"
plutil -replace shellExitAction -integer 1 "${TERMINAL_SETTINGS_PATH}"

cat >"${LAUNCHER_PATH}" <<'LAUNCHER'
#!/usr/bin/env bash
set -euo pipefail

APP_CONTENTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TERMINAL_SETTINGS="${APP_CONTENTS_DIR}/Resources/CNN EE Audio Player.terminal"

exec /usr/bin/open "${TERMINAL_SETTINGS}"
LAUNCHER

chmod +x "${LAUNCHER_PATH}" "${START_SCRIPT_PATH}"

rm -rf "${APPLICATIONS_PATH}"
cp -R "${APP_PATH}" "${APPLICATIONS_PATH}"

echo "Installed ${APPLICATIONS_PATH}"
