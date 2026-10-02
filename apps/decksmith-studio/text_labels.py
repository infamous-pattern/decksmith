"""Validation and compact defaults for user-visible device labels."""
import unicodedata

MAX_LABEL_CHARACTERS = 24


def valid_label(value):
    """Accept short printable names, including emoji and joined emoji sequences."""
    if not isinstance(value, str) or not 1 <= len(value) <= MAX_LABEL_CHARACTERS:
        return False
    if not any(not char.isspace() and char != '\u200d' and not _emoji_tag(char) for char in value):
        return False
    for char in value:
        if char == ' ' or char.isascii() and char.isalnum():
            continue
        if char.isascii():
            return False
        category = unicodedata.category(char)
        if category in ('Cc', 'Cs', 'Zl', 'Zp') or char.isspace():
            return False
        if category == 'Cf' and char != '\u200d' and not _emoji_tag(char):
            return False
    return True


def _emoji_tag(char):
    return 0xE0020 <= ord(char) <= 0xE007F


def generated_label(value, fallback='Label'):
    """Normalize discovered names while retaining emoji, then fit the label limit."""
    output = []
    for char in unicodedata.normalize('NFKD', str(value)):
        category = unicodedata.category(char)
        if char.isascii():
            output.append(char if char.isalnum() else ' ')
        elif category[0] in ('L', 'N', 'S') or char in ('\u200d', '\ufe0f', '\u20e3') or _emoji_tag(char):
            output.append(char)
        elif category[0] == 'M':
            # NFKD marks on letters are stripped; keep emoji presentation/keycap marks.
            if char == '\ufe0f' or char == '\u20e3':
                output.append(char)
        else:
            output.append(' ')
    result = ' '.join(''.join(output).split())[:MAX_LABEL_CHARACTERS].rstrip()
    while result and (result[-1] == '\u200d' or _emoji_tag(result[-1])):
        result = result[:-1]
    return result or fallback
