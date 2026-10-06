"""Read build-time product version from Cargo's authoritative workspace metadata."""
from pathlib import Path
import re
import tomllib


def application_version(root):
    metadata = tomllib.loads((Path(root) / 'Cargo.toml').read_text())
    version = metadata['workspace']['package']['version']
    if not isinstance(version, str) or not re.fullmatch(
            r'(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)'
            r'(?:-[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?', version):
        raise ValueError('Invalid workspace application version')
    return version
