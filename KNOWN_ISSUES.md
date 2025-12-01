# Known issues

### Usage is rendered strange

```bash
> dsh --dry-run --output-format json certificate show broker --usage
```

### Injection fields start with a capital

```bash
> dsh --dry-run --output-format json env find ^info$ --regex
```
