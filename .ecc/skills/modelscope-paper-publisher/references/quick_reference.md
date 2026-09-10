# Quick Reference

## Create Draft

```bash
uv run scripts/paper_manager.py create \
  --title "Your Paper Title" \
  --authors "Author A,Author B" \
  --template standard \
  --output ./papers/your-paper.md
```

## Upload Draft

```bash
uv run scripts/paper_manager.py upload \
  --repo-id your-org/your-model \
  --input ./papers/your-paper.md \
  --path-in-repo reports/papers/your-paper.md
```

## List Templates

```bash
uv run scripts/paper_manager.py list-templates
```
