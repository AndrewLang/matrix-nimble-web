import pathlib, re
root = pathlib.Path('examples/photos')
for path in root.rglob('*.rs'):
    text = path.read_text()
    text_no_block = re.sub(r'/\*.*?\*/', '', text, flags=re.S)
    lines = []
    for line in text_no_block.splitlines():
        parts = line.split('"')
        new_parts = []
        for i, part in enumerate(parts):
            if i % 2 == 0:
                idx = part.find('//')
                if idx != -1:
                    part = part[:idx]
            new_parts.append(part)
        replaced = '"'.join(new_parts)
        lines.append(replaced.rstrip())
    new_text = '\n'.join(lines).rstrip()
    if new_text:
        new_text += '\n'
    path.write_text(new_text)
