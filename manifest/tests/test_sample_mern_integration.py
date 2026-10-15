"""Integration tests for Open SaaS sample execution through discovery."""

import json
import subprocess
import sys
from pathlib import Path
from typing import cast

import pytest


@pytest.fixture
def manifest_dir() -> Path:
    """Get the manifest directory path."""
    return Path(__file__).parent.parent


def test_open_saas_output_via_discovery(manifest_dir: Path) -> None:
    """Test that the Open SaaS sample matches discovery output."""
    result = subprocess.run(  # noqa: S603
        [sys.executable, "-c", _get_open_saas_collection_script(manifest_dir)],
        cwd=manifest_dir,
        capture_output=True,
        text=True,
        check=True,
    )

    discovery_output = result.stdout.strip()

    # Run the Open SaaS sample directly and capture output.
    sample_result = subprocess.run(  # noqa: S603
        [sys.executable, "-c", _get_open_saas_sample_script(manifest_dir)],
        cwd=manifest_dir,
        capture_output=True,
        text=True,
        check=True,
    )

    sample_output = sample_result.stdout.strip()

    # Parse both outputs as JSON to compare semantically
    discovery_json = json.loads(discovery_output)
    sample_json = json.loads(sample_output)

    # Verify the outputs are identical
    assert discovery_json == sample_json, (
        "Discovery output differs from the Open SaaS sample output.\n"
        f"Discovery:\n{json.dumps(discovery_json, indent=2)}\n\n"
        f"Sample_open_saas.py:\n{json.dumps(sample_json, indent=2)}"
    )


def test_open_saas_output_structure(manifest_dir: Path) -> None:
    """Test that Open SaaS output has the correct structure."""
    result = subprocess.run(  # noqa: S603
        [sys.executable, "-c", _get_open_saas_collection_script(manifest_dir)],
        cwd=manifest_dir,
        capture_output=True,
        text=True,
        check=True,
    )

    output = json.loads(result.stdout)

    # Discovery currently emits 8 Open SaaS blocks
    assert isinstance(output, list), "Output should be a list"
    assert len(output) == 8, f"Expected 8 blocks, got {len(output)}"

    # Verify each block has expected fields
    for i, block in enumerate(output):
        assert isinstance(block, dict), f"Block {i} should be a dict"
        block_data = cast("dict[str, object]", block)
        assert "name" in block_data, f"Block {i} missing 'name' field"
        assert isinstance(block_data["name"], str), f"Block {i} name should be a string"


def test_open_saas_blocks_have_correct_names(manifest_dir: Path) -> None:
    """Test that discovery includes expected Open SaaS blocks."""
    result = subprocess.run(  # noqa: S603
        [sys.executable, "-c", _get_open_saas_collection_script(manifest_dir)],
        cwd=manifest_dir,
        capture_output=True,
        text=True,
        check=True,
    )

    output = json.loads(result.stdout)
    block_names = [cast("dict[str, object]", block)["name"] for block in output]

    # Should contain key Open SaaS blocks
    assert "PostgreSQL" in block_names, "Missing PostgreSQL block"
    assert "Wasp Server" in block_names, "Missing Wasp Server block"
    assert "Wasp Client" in block_names, "Missing Wasp Client block"
    assert "Blog & Docs" in block_names, "Missing Blog & Docs block"
    assert "Stripe" in block_names, "Missing Stripe block"
    assert "Email" in block_names, "Missing Email block"
    assert "AWS S3" in block_names, "Missing AWS S3 block"
    assert "Analytics" in block_names, "Missing Analytics block"


def test_scryr_types_module_reexports_manifest_types(manifest_dir: Path) -> None:
    """Integration check: scryr.types is importable from sample runtime context."""
    result = subprocess.run(  # noqa: S603
        [sys.executable, "-c", _get_types_module_script(manifest_dir)],
        cwd=manifest_dir,
        capture_output=True,
        text=True,
        check=True,
    )

    output = json.loads(result.stdout)
    assert output["name_type"] == "Label"
    assert output["description_type"] == "Markdown"
    assert output["version_type"] == "SemVer"


def _get_open_saas_sample_script(manifest_dir: Path) -> str:
    """Get a script that runs the Open SaaS sample and outputs its blocks."""
    return f"""
import sys
import json
sys.path.insert(0, '{manifest_dir}')

from scryr.runtime import load_module_from_path

sample = load_module_from_path('{manifest_dir / "tests" / "samples" / "open_saas" / "index.scry"}')

blocks = [
    json.loads(sample.postgres_block.model_dump_json()),
    json.loads(sample.server_block.model_dump_json()),
    json.loads(sample.client_block.model_dump_json()),
    json.loads(sample.blog_block.model_dump_json()),
    json.loads(sample.stripe_block.model_dump_json()),
    json.loads(sample.email_block.model_dump_json()),
    json.loads(sample.s3_block.model_dump_json()),
    json.loads(sample.analytics_block.model_dump_json()),
]
print(json.dumps(blocks, indent=2))
"""


def _get_open_saas_collection_script(manifest_dir: Path) -> str:
    """Get a script that collects Open SaaS blocks through scryr discovery."""
    return f"""
import sys
import json
from pathlib import Path
sys.path.insert(0, '{manifest_dir}')

from scryr.discovery import collect_manifest_blocks

blocks = collect_manifest_blocks(Path('{manifest_dir}') / 'tests' / 'samples' / 'open_saas')
print(json.dumps(blocks, indent=2))
"""


def _get_types_module_script(manifest_dir: Path) -> str:
    """Get a script that validates `scryr.types` imports and runtime typing."""
    return f"""
import sys
import json
sys.path.insert(0, '{manifest_dir}')

from scryr.runtime import load_module_from_path
from scryr.types import Label, Markdown, SemVer

sample = load_module_from_path('{manifest_dir / "tests" / "samples" / "mern" / "index.scry"}')

output = {{
    "name_type": type(sample.react_block.name).__name__,
    "description_type": type(sample.react_block.info.description).__name__,
    "version_type": type(sample.react_block.info.version).__name__,
    "label_isinstance": isinstance(sample.react_block.name, (Label, str)),
    "markdown_isinstance": isinstance(sample.react_block.info.description, (Markdown, str)),
    "semver_isinstance": isinstance(sample.react_block.info.version, SemVer),
}}
print(json.dumps(output))
"""
