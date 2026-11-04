#!/usr/bin/env python3
"""Check relative links in maintained docs, offline; preserve historical evidence."""
import re
import sys
import unicodedata
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r'!?\[[^\]\n]*\]\(([^)\n]+)\)|<(?:img|a)\b[^>]*?\b(?:src|href)=["\']([^"\']+)["\']')


def without_fences(text):
    """Ignore example code while retaining line positions for readable errors."""
    result = []
    fence = None
    for line in text.splitlines(keepends=True):
        marker = re.match(r'^\s{0,3}(`{3,}|~{3,})', line)
        if marker:
            value = marker.group(1)
            if fence is None:
                fence = value
            elif value[0] == fence[0] and len(value) >= len(fence):
                fence = None
            result.append('\n')
        else:
            result.append('\n' if fence else line)
    return ''.join(result)


def anchors(path):
    """GitHub-style IDs for the ATX headings used by this repository."""
    found = set()
    counts = {}
    for line in without_fences(path.read_text()).splitlines():
        match = re.match(r'^#{1,6}\s+(.+?)\s*#*\s*$', line)
        if not match:
            continue
        title = re.sub(r'<[^>]+>', '', match.group(1)).lower()
        title = re.sub(r'\[([^]]+)\]\([^)]+\)', r'\1', title)
        slug = ''.join(
            char for char in title
            if char in ' -_' or unicodedata.category(char)[0] in ('L', 'N', 'M')
        ).replace(' ', '-')
        duplicate = counts.get(slug, 0)
        counts[slug] = duplicate + 1
        found.add(f'{slug}-{duplicate}' if duplicate else slug)
    return found


def main():
    files = [ROOT / 'README.md', ROOT / 'AGENTS.md', *sorted((ROOT / 'docs').glob('*.md'))]
    for folder in ('host', 'portable', 'assets', 'config'):
        files.extend(p for p in (ROOT / folder).rglob('*.md') if 'target' not in p.parts)
    errors = []
    checked = 0
    cache = {}
    indexed = set()
    for source in files:
        text = without_fences(source.read_text())
        for match in LINK.finditer(text):
            raw = match.group(1) or match.group(2)
            target = raw.strip().split(' "', 1)[0].strip('<>')
            parts = urlsplit(target)
            if parts.scheme or parts.netloc:
                continue
            checked += 1
            path = (source.parent / unquote(parts.path)).resolve() if parts.path else source
            if source == ROOT / "docs/README.md":
                indexed.add(path)
            problem = None
            if not path.is_relative_to(ROOT):
                problem = 'path leaves repository'
            elif not path.exists():
                problem = 'missing path'
            elif parts.fragment and path.suffix == '.md':
                if path not in cache:
                    cache[path] = anchors(path)
                if unquote(parts.fragment) not in cache[path]:
                    problem = 'missing heading'
            if problem:
                line = text.count('\n', 0, match.start()) + 1
                errors.append(f'{source.relative_to(ROOT)}:{line}: {problem}: {target}')
    for document in sorted((ROOT / 'docs').glob('*.md')):
        if document.name != 'README.md' and document not in indexed:
            errors.append(f'docs/README.md: missing index entry: {document.name}')
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print(f'PASS: {checked} relative links across {len(files)} documentation files')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
