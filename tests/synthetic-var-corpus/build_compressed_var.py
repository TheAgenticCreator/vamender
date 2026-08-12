# SPDX-License-Identifier: MIT

import json
import pathlib
import sys
import zipfile


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit("usage: build_compressed_var.py <path> <bzip2|lzma> <creator.package.version>")

    destination = pathlib.Path(sys.argv[1])
    compression_name = sys.argv[2]
    package_id = sys.argv[3]
    compression = {
        "bzip2": zipfile.ZIP_BZIP2,
        "lzma": zipfile.ZIP_LZMA,
    }.get(compression_name)
    if compression is None:
        raise SystemExit(f"unsupported compression fixture: {compression_name}")

    parts = package_id.split(".")
    if len(parts) < 3:
        raise SystemExit(f"invalid synthetic package id: {package_id}")

    destination.parent.mkdir(parents=True, exist_ok=True)
    metadata = {
        "name": package_id,
        "creatorName": parts[0],
        "packageName": parts[1],
        "licenseType": "CC BY",
        "description": "Synthetic VaMender compression fixture",
        "dependencies": {},
    }
    with zipfile.ZipFile(destination, "w", compression=compression) as archive:
        archive.writestr("meta.json", json.dumps(metadata, separators=(",", ":")))
        archive.writestr("Custom/Assets/Demo/compressed.txt", compression_name)
    print(destination)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
