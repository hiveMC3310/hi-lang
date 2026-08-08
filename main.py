"""
Рекурсивный обход и вывод дерева папок и файлов в текущей директории.
Игнорирует содержимое папок .git и target (сами папки видны, но не раскрываются).
"""

import sys
from pathlib import Path

# Папки, внутренность которых не нужно показывать
IGNORE_DIRS = {'.git', 'target', '.idea'}

def print_tree(directory: Path, prefix: str = '', is_last_global: bool = True) -> None:
    """
    Рекурсивно выводит содержимое директории в виде дерева.
    Папки из IGNORE_DIRS выводятся, но их содержимое не обходится.
    """
    items = list(directory.iterdir())
    dirs = sorted([p for p in items if p.is_dir()], key=lambda p: p.name)
    files = sorted([p for p in items if p.is_file()], key=lambda p: p.name)
    all_items = dirs + files

    for idx, item in enumerate(all_items):
        is_last = (idx == len(all_items) - 1)
        connector = '└── ' if is_last else '├── '
        print(prefix + connector + item.name)

        # Рекурсивный обход только для папок, не входящих в игнорируемый список
        if item.is_dir() and item.name not in IGNORE_DIRS:
            extension = '    ' if is_last else '│   '
            print_tree(item, prefix + extension, is_last)

def main():
    if len(sys.argv) > 1:
        start_path = Path(sys.argv[1])
    else:
        start_path = Path('.')

    if not start_path.exists() or not start_path.is_dir():
        print(f"Ошибка: '{start_path}' не является существующей папкой.", file=sys.stderr)
        sys.exit(1)
    
    print(start_path.resolve().name if start_path.is_dir() else start_path.name)
    print_tree(start_path)

if __name__ == '__main__':
    main()
