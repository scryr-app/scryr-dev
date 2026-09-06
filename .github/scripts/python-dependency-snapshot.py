"""Describe every third-party package in uv.lock for GitHub's dependency graph.

No dependency code is installed or executed. Include all locked platforms and
both development and runtime packages so transitive vulnerabilities are visible.
"""

import datetime
import json
import os
import tomllib
from pathlib import Path
from urllib.parse import quote

lock = tomllib.loads(Path("manifest/uv.lock").read_text())
resolved = {}
for package in lock["package"]:
    if "registry" not in package["source"]:
        continue  # Workspace members are first-party code, not PyPI packages.
    name = package["name"].lower().replace("_", "-")
    purl = f"pkg:pypi/{quote(name, safe='-')}@{quote(package['version'], safe='')}"
    resolved[purl] = {"package_url": purl}

print(json.dumps({
    "version": 0,
    "sha": os.environ["GITHUB_SHA"],
    "ref": os.environ["GITHUB_REF"],
    "job": {"id": os.environ["GITHUB_RUN_ID"], "correlator": "scryr-uv-lock"},
    "detector": {
        "name": "scryr-uv-lock",
        "version": "1.0.0",
        "url": "https://github.com/scryr-app/scryr-dev",
    },
    "scanned": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "manifests": {
        "manifest/uv.lock": {
            "name": "manifest/uv.lock",
            "file": {"source_location": "manifest/uv.lock"},
            "resolved": resolved,
        },
    },
}))
