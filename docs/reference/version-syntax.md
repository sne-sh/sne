# Version syntax

Anywhere sne takes a version, on the command line or in a
[version file](../guides/version-files.md), you can write it in these ways.

| You write | Meaning |
|---|---|
| `22` | The newest 22.x.y |
| `22.11` | The newest 22.11.y |
| `22.11.0` or `v22.11.0` | Exactly that version |
| `latest` | The newest release of all |
| `lts` or `lts/*` | The newest LTS release |
| `lts/iron` | The newest release of that LTS line |
| `>=20 <23` | The newest version in the range |
| `work` | An [alias](../guides/aliases.md) you created |

## Which version is picked

* `sne install` and `sne ls-remote` choose from the versions published on nodejs.org.
* `sne use`, `sne exec`, `sne which`, `sne default` and `sne uninstall` choose from the versions you
  have installed.

So `sne use 22` switches to the newest 22 that is installed, even when a newer 22 exists online. Run
`sne install 22` to get the newer one.

## Ranges

Ranges use the same syntax as npm, for example `>=20`, `^20.10`, `~22.4` or `>=20 <23`. Put a range in
quotes on the command line so the shell does not read `<` and `>` as redirection:

```sh
sne install ">=20 <23"
```
