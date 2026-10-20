# Scryr

Scryr is a Python package for defining architecture manifests as typed Pydantic
models. Applications can use it as a library.

Use `Classification` from `scryr.types` for architectural roles and interfaces:

```python
from scryr import Manifest
from scryr.types import Classification

api = Manifest(name="API", classification=Classification.rest)
```

`Classification` includes existing roles such as `public_api`, `database`, and
`worker`, plus interface choices such as `rest`, `graphql`, `grpc`, and `sql`.
Values serialize as strings. `InterfaceType` remains an import alias for
`Classification`; its former tuple values and interface metadata properties
(`category`, `id`, and `description`) are no longer supported.
