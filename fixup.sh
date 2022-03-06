#!/usr/bin/env bash

cat >dist/cjs/package.json <<!EOF
{
    "type": "commonjs"
}
!EOF

cat >dist/mjs/package.json <<!EOF
{
    "type": "module"
}
!EOF

cp -r .npmignore README.md package.json LICENSE bin-package dist