"""Read user-selected files without allocating beyond their import limit."""

def read_limited(path, limit, message):
    with path.open('rb') as source:
        data=source.read(limit+1)
    if len(data)>limit:raise ValueError(message)
    return data
