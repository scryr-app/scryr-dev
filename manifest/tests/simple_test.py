"""Tests for discovery functionality."""

import sys
import tempfile
from pathlib import Path

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from scryr.discovery import discover_pydantic_instances


def test_discover_pydantic_instances_in_temp_directory() -> None:
    """Test that discover_pydantic_instances finds Pydantic instances in a given path."""
    # Create a temporary directory
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)

        # Create a test Python file with a Pydantic instance
        test_file = tmp_path / "test_sample.py"
        test_file.write_text(
            """
from pydantic import BaseModel, Field

class TestModel(BaseModel):
    name: str = Field(..., description="Name field")
    value: int = Field(default=100, description="Value field")

# Create an instance
test_instance = TestModel(name="test_object", value=999)
"""
        )

        # Run discovery
        results = discover_pydantic_instances(tmp_path)

        # Verify results
        assert len(results) > 0, "Should find at least one instance"

        # Find our test instance
        test_instance_data = None
        for item in results:
            if item["name"] == "test_instance":
                test_instance_data = item
                break

        assert test_instance_data is not None, "Should find test_instance"
        assert test_instance_data["file"] == "test_sample.py"
        assert test_instance_data["data"]["name"] == "test_object"
        assert test_instance_data["data"]["value"] == 999


def test_discover_pydantic_instances_current_directory(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that discover_pydantic_instances defaults to current directory."""
    monkeypatch.chdir(tmp_path)

    results = discover_pydantic_instances(None)

    assert results == []


def test_discover_pydantic_instances_nonexistent_path() -> None:
    """Test that discover_pydantic_instances raises error for nonexistent path."""
    with pytest.raises(ValueError, match="Path does not exist"):
        discover_pydantic_instances(Path("/nonexistent/path/12345"))


def test_discover_pydantic_instances_nested_directories() -> None:
    """Test that discover_pydantic_instances finds instances in nested directories."""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)

        # Create nested directory structure
        nested_dir = tmp_path / "subdir" / "nested"
        nested_dir.mkdir(parents=True)

        # Create a file in the nested directory
        nested_file = nested_dir / "nested_sample.py"
        nested_file.write_text(
            """
from pydantic import BaseModel

class NestedModel(BaseModel):
    title: str

nested_obj = NestedModel(title="nested")
"""
        )

        # Run discovery
        results = discover_pydantic_instances(tmp_path)

        # Verify results
        assert len(results) > 0, "Should find instances in nested directories"

        nested_instance = None
        for item in results:
            if item["name"] == "nested_obj":
                nested_instance = item
                break

        assert nested_instance is not None, "Should find nested_obj"
        assert "subdir" in nested_instance["file"]
        assert nested_instance["data"]["title"] == "nested"


def test_discover_pydantic_instances_skips_private_files() -> None:
    """Test that discover_pydantic_instances skips files starting with underscore."""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)

        # Create a private file
        private_file = tmp_path / "_private.py"
        private_file.write_text(
            """
from pydantic import BaseModel

class PrivateModel(BaseModel):
    secret: str

private_instance = PrivateModel(secret="hidden")
"""
        )

        # Run discovery
        results = discover_pydantic_instances(tmp_path)

        # Verify private files are skipped
        for item in results:
            assert not item["file"].startswith("_"), "Should skip private files"
            assert item["name"] != "private_instance", "Should not find private_instance"


def test_discover_pydantic_instances_multiple_instances() -> None:
    """Test finding multiple instances in the same file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)

        # Create a file with multiple instances
        test_file = tmp_path / "multiple.py"
        test_file.write_text(
            """
from pydantic import BaseModel

class Config(BaseModel):
    setting: str

config1 = Config(setting="value1")
config2 = Config(setting="value2")
"""
        )

        # Run discovery
        results = discover_pydantic_instances(tmp_path)

        # Verify we found both instances
        assert len(results) == 2, f"Expected 2 instances, found {len(results)}"

        instance_names = {item["name"] for item in results}
        assert "config1" in instance_names, "Should find config1"
        assert "config2" in instance_names, "Should find config2"
