# Autocompletion

[&#x2190; Platforms specification](platforms-specification.md)

The `dsh` tool has a hidden option `--generate-autocomplete-file`, which can be used
to generate an autocompletion file for the following shells:

* `bash` - Bourne-again shell
* `elvish` - Elvish shell
* `fish` - Fish shell
* `powershell` - Microsoft Powershell
* `zsh` - Z shell

For the `zsh` shell see below. For the other shell types, please consult your shell's documentation
how to install the autocompletion file.

## Bourne-again shell

```shell
> dsh --generate-autocomplete-file bash
```

## Elvish shell

```shell
> dsh --generate-autocomplete-file elvish
```

## Fish shell

```shell
> dsh --generate-autocomplete-file fish
```

## Microsoft Powershell

```shell
> dsh --generate-autocomplete-file powershell
```

## Z shell

Run `dsh` with the `--generate-autocomplete-file zsh` flag and redirect the result
to an autocomplete file name `_dsh`.

```shell
> dsh --generate-autocomplete-file zsh > _dsh
```

Then copy the autocomplete file to your `zsh` autocomplete directory.
Note that this most likely will require `sudo`.

```shell
> mv _dsh /usr/local/share/zsh/site-functions/_dsh
```

Finally, add the following two lines to your `~/.zshrc` file.

```
# enable autocomplete
autoload -Uz compinit
compinit
```

[Developers &#x2192;](developers.md)
