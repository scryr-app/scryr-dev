"""Exercise the browser runner's Python boundary with the real SDK."""

import json
import os
import sys
from pathlib import Path
from typing import Any


def test_browser_preview_preserves_plan_without_running_commands(tmp_path, monkeypatch) -> None:
    """The emitted plan bypasses manifest line mapping and remains declarative."""
    repository = Path(__file__).resolve().parents[2]
    runner = repository / "map/src/pyodide/runDocument.py"
    sample = repository / "manifest/tests/samples/collectors/index.scry"
    marker = tmp_path / "command-was-run"
    command = [sys.executable, "-c", f"from pathlib import Path; Path({str(marker)!r}).touch()"]
    source = sample.read_text().replace(
        'argv=["printf", "%s", \'{"version":"demo-v1","state":"healthy","ready_instances":3}\']',
        f"argv={command!r}",
    )
    assert str(marker) in source
    (tmp_path / "index.scry").write_text(source)
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(sys, "path", sys.path.copy())

    def browser_chdir(path: str) -> None:
        """Map the browser's project directory onto pytest's isolated directory."""
        assert path == "/project"

    monkeypatch.setattr(os, "chdir", browser_chdir)
    files = [{"path": "index.scry", "content": source}]
    namespace: dict[str, Any] = {
        "_scryr_document": json.dumps({"entrypoint": "index.scry", "files": files})
    }
    exec(compile(runner.read_text(), str(runner), "exec"), namespace)
    envelope = namespace["envelope"]
    assert envelope["files"] == files
    assert len(envelope["manifests"]) == 1
    assert envelope["manifests"][0]["line_number"] > 0
    assert len(envelope["collection_plan"]["collectors"]) == 1
    assert len(envelope["collection_plan"]["cards"]) == 3
    assert str(marker) in envelope["collection_plan"]["tasks"][0]["command"]["argv"][2]
    assert not marker.exists()
