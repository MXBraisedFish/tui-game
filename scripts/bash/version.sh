#!/bin/sh
set -eu
"$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)/version" "$@"
