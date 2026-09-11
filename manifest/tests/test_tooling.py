"""Source tooling must preserve user files and follow local Scryr imports."""

import os
import sys
from pathlib import Path

import pytest

from scryr.tooling import run, sources


@pytest.fixture(autouse=True)
def tool_path(monkeypatch) -> None:
    """Use the test environment's pinned development tools."""
    monkeypatch.setenv("PATH", str(Path(sys.executable).parent) + os.pathsep + os.environ["PATH"])


def test_only_reachable_sources_are_formatted(tmp_path, capsys) -> None:
    """Exercise tooling against actual files and installed checkers."""
    entry = tmp_path / "index.scry"
    imported = tmp_path / "service.scry"
    unrelated = tmp_path / "unrelated.py"
    entry.write_text("from service import api\n")
    imported.write_text("api={ 'name':'test'}\n")
    unrelated.write_text("unrelated={ 'leave':'alone'}\n")
    assert sources(entry) == [entry, imported]
    assert run(entry, "--format-check") == 1
    diagnostics = capsys.readouterr().out
    assert f"Would reformat {imported}" in diagnostics
    assert str(unrelated) not in diagnostics
    assert imported.read_text() == "api={ 'name':'test'}\n"
    assert run(entry, "--format") == 0
    assert run(entry, "--format-check") == 0
    assert unrelated.read_text() == "unrelated={ 'leave':'alone'}\n"


def test_format_does_not_execute_user_code(tmp_path) -> None:
    """Exercise tooling against actual files and installed checkers."""
    entry = tmp_path / "index.scry"
    entry.write_text("raise RuntimeError('must not execute')\n")
    assert run(entry, "--format") == 0


def test_hyphenated_declarations_round_trip(tmp_path) -> None:
    """Exercise tooling against actual files and installed checkers."""
    entry = tmp_path / "index.scry"
    entry.write_text("from scryr import Manifest\nmy-service = Manifest(name='API')\n")
    assert run(entry, "--format") == 0
    assert "my-service = Manifest" in entry.read_text()
    assert "scryr_scry" not in entry.read_text()
    assert run(entry, "--format-check") == 0


def test_multiline_hyphenated_declaration_format_is_idempotent(tmp_path) -> None:
    """Formatting must converge when a Scryr declaration opens on its own line."""
    entry = tmp_path / "index.scry"
    entry.write_text(
        'from scryr import Manifest\n\nnorthwind-postgres = Manifest(\nname="Database",\n)\n'
    )

    assert run(entry, "--format") == 0
    formatted = entry.read_text()
    assert "Manifest(\n    name=" in formatted
    assert run(entry, "--format-check") == 0
    assert entry.read_text() == formatted


def test_types_resolve_imported_scry_and_report_original_path(tmp_path, capsys) -> None:
    """Exercise tooling against actual files and installed checkers."""
    entry = tmp_path / "index.scry"
    service = tmp_path / "service.scry"
    entry.write_text("from service import value\ncount: int = value\n")
    service.write_text('value: str = "wrong"\n')
    assert run(entry, "--typecheck") == 1
    assert str(entry) in capsys.readouterr().out
    service.write_text("value: int = 1\n")
    assert run(entry, "--typecheck") == 0


def test_lint_fixes_preserve_imported_public_manifest_objects(tmp_path) -> None:
    """Public imports are exports even when Python does not reference them."""
    entry = tmp_path / "index.scry"
    entry.write_text("from service import api\n")
    (tmp_path / "service.scry").write_text('api = "exported"\n')
    assert run(entry, "--lint-fix") == 0
    assert entry.read_text() == "from service import api\n"


def test_format_preserves_strings_that_resemble_internal_aliases(tmp_path) -> None:
    """Restoring declaration names must not change user string values."""
    entry = tmp_path / "index.scry"
    entry.write_text(
        'from scryr import Manifest\nmy-service = Manifest(name="scryr_scry_2_my_service")\n'
    )
    assert run(entry, "--format") == 0
    assert 'name="scryr_scry_2_my_service"' in entry.read_text()
