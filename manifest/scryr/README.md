# Scryr

Scryr is a Python package for defining architecture manifests as typed Pydantic
models. Applications can use it as a library, and tools can execute manifest
files through `python -m scryr.cli`.

```bash
pip install scryr
```

```python
from scryr import Manifest, ProgrammingLanguage, WebFramework

api = Manifest(
    name="Public API",
    language=ProgrammingLanguage.python,
    frameworks=[WebFramework.fastapi],
    description="Customer-facing HTTP API",
)

print(api.to_dict())
```

The public `scryr` command is the Rust CLI. The Python SDK entrypoint remains
available as a module for tooling:

```bash
python -m scryr.cli architecture.py --json
python -m scryr.cli architecture.py --types
python -m scryr.cli architecture.py --schema
```
