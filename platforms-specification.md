## Platforms specification

[&#x2190; Settings and targets](settings_targets.md)

If you need a different set of platform configurations,
you can use the `DSH_API_PLATFORMS_FILE` environment variable to provide
the name of a file with an alternative list of platform specifications.
It can either be an absolute file name
or a relative file name from the working directory.
When this environment variable is set, the normal list of default platforms
will <em>not</em> be included. If you need these too, make sure that you also
include the default platforms in your platforms file.

The default platforms file is defined in the `dsh_api` library crate.
When tou want to create your own platforms file you can use the default file as a starting point.

```bash
> dsh platform export > my-platforms.json
```

Open the file in your favorite editor and make the required changes,
Then set the environment variable to point to the file and see the new platform configuration:

```bash
> export DSH_API_PLATFORMS_FILE=my-platforms.json
> dsh platform list
```

See the github for the [`dsh_api`](https://github.com/kpn-dsh/dsh-api) for more information
about the platforms specifications.

[Set up autocompletion &#x2192;](autocompletion.md)
