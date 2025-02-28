# Known issues

```bash
> dsh --dry-run --output-format json platform show --app kafdrop --platform prodlz --tenant greenbox-dev
```

Wrong/incomplete results

```bash
> dsh --dry-run --output-format json platform service list --tasks
```

Service id duplicated

```bash
> dsh --dry-run --output-format json platform certificate show broker --usage
```

Usage is rendered strange

```bash
> dsh --dry-run --output-format json env find ^info$ --regex
```

Injection fields start with a capital
