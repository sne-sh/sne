# Project version files

A version file records which Node.js version a project needs, so everyone on the project uses the
same one.

## Files sne reads

When you run `sne install` or `sne use` without a version, sne looks in the current folder and then in
each parent folder. In every folder it checks, in this order:

1. `.node-version`
2. `.nvmrc`
3. `package.json`, if it has an `engines.node` field

The first one found wins.

## What to write in them

`.node-version` and `.nvmrc` hold one version on one line. Anything from the
[version syntax](../reference/version-syntax.md) works:

```
22
```

```
lts/iron
```

```
v22.11.0
```

In `package.json`, sne reads the standard `engines` field:

```json
{
  "engines": {
    "node": ">=20 <23"
  }
}
```

## Create one

```sh
echo "22" > .nvmrc
```

Commit the file so your teammates and your CI get the same version.

## Use it

```sh
sne install   # install what the project asks for
sne use       # switch this terminal to it
```

To have this happen when you enter the folder, see [Switching automatically](auto-switch.md).
