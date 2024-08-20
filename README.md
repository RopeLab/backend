# backend

## Setup 

shh tunnel postgres database
```shell
ssh -L 5432:127.0.0.1:5432 ropelab@betelgeuse.uberspace.de
```

Diesel init
```shell
diesel migration run
```

## Run
```shell
cargo run
```

## Debugging Database
```shell
nix-shell shell.nix
dbeaver
```