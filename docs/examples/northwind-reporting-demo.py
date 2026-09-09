"""Exercise local reporting against Northwind's existing JUnit and coverage artifacts.

Run Northwind's tests first as described in docs/cli-reporting.md. Dependabot and
staging deployment observations in this demo are explicitly synthetic.
"""

import datetime
import json
from pathlib import Path
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
PROJECT = ROOT.parent / "ex-northwind-commerce"
OUTPUT = ROOT / ".scryr/local-reports"
CLI = ROOT / "crystal/target/debug/scryr"
ENDPOINT = "http://127.0.0.1:8001/graphql"
NOW = datetime.datetime.now(datetime.timezone.utc).isoformat()
RUN = "northwind-local-" + NOW


def command(*args):
    result = subprocess.run([str(CLI), *args], cwd=ROOT, check=True, capture_output=True, text=True)
    return result.stdout


def query(text):
    request = urllib.request.Request(
        ENDPOINT, data=json.dumps({"query": text}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    assert not result.get("errors"), result
    return result["data"]


def upload():
    # Explicit environment only for the upload command's legacy endpoint setting.
    import os
    environment = dict(os.environ, SCRYR_GRAPHQL_URL=ENDPOINT)
    subprocess.run([
        str(CLI), "generate", "upload", "--path", str(PROJECT / "index.scry"),
        "--manifest-dir", str(PROJECT), "--scryr-dir", str(ROOT / ".scryr"),
    ], cwd=ROOT, env=environment, check=True)


def report(kind, *extra, observed=NOW):
    return json.loads(command(
        "report", kind, "--manifest-id", "northwind-commerce/api",
        "--endpoint", ENDPOINT, "--run-id", RUN, "--observed-at", observed,
        "--branch", "main", "--json", *extra,
    ))


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    upload()
    tests = ("--file", str(OUTPUT / "northwind-unit.xml"), "--suite", "unit")
    assert report("tests", *tests)["recorded"]
    assert not report("tests", *tests)["recorded"]
    assert report("coverage", "--file", str(OUTPUT / "northwind-coverage.xml"),
                  "--format", "cobertura", "--suite", "unit")["recorded"]
    fixture = [{
        "number": 900001, "state": "open",
        "dependency": {"manifest_path": "api/uv.lock", "package": {"name": "synthetic-demo-package", "ecosystem": "pip"}},
        "security_advisory": {"severity": "high"},
        "html_url": "https://github.com/scryr-app/ex-northwind-commerce",
    }]
    path = OUTPUT / "dependabot-synthetic.json"
    path.write_text(json.dumps(fixture))
    assert report("dependencies", "--file", str(path), "--repository",
                  "scryr-app/ex-northwind-commerce", "--manifest-path", "api/uv.lock")["recorded"]
    assert report("deployment", "--environment", "staging", "--status", "success",
                  "--version", "synthetic-local-demo")["recorded"]
    old = OUTPUT / "older-failing.xml"
    old.write_text('<testsuite tests="1"><testcase name="old"><failure/></testcase></testsuite>')
    assert report("tests", "--file", str(old), "--suite", "unit",
                  observed="2020-01-01T00:00:00Z")["recorded"]
    upload()
    blocks = query('{ blocks(scryIdentifier: "northwind_commerce_diagram") { name rawJsonString } }')["blocks"]
    api = next(json.loads(b["rawJsonString"]) for b in blocks if b["name"] == "northwind-api")
    assert api["tests"]["total"] == 16, api["tests"]
    assert api["tests"]["passing"] == 16 and api["tests"]["failing"] == 0
    assert api["tests"]["coverage"] > 0
    assert api["dependencies"]["vulnerableDeps"] == 1
    assert api["dependencies"].get("totalDeps") is None
    assert api["cicd"]["deployStatusStaging"] == "deployed"
    summary = {"tests": api["tests"], "dependencies": api["dependencies"], "cicd": api["cicd"]}
    (OUTPUT / "verified-summary.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps({"passing": api["tests"]["passing"], "coverage": api["tests"]["coverage"],
                      "syntheticVulnerableDependencies": 1, "deduplication": "verified",
                      "delayedDelivery": "verified", "regeneration": "verified"}, indent=2))


if __name__ == "__main__":
    main()
