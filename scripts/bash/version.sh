#!/bin/sh
set +x
set +v
set -eu
"$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)/version" "$@"
