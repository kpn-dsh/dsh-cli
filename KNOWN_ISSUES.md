# Known issues

### Service id duplicated

```bash
> dsh --dry-run --output-format json platform service list --tasks
```

### Usage is rendered strange

```bash
> dsh --dry-run --output-format json platform certificate show broker --usage
```

### Injection fields start with a capital

```bash
> dsh --dry-run --output-format json env find ^info$ --regex
```

### Doesn't show dependants

```bash
> dsh bucket show schema-registry
```

