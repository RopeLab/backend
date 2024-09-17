# backend

## Dev environment 

### Nix
If you want to use nix shell to install all needed programs
```shell
nix-shell ./shell.nix
```

#### Without Nix 
Install these programs:
- rust (cargo) https://www.rust-lang.org/tools/install
- clang (cpp compilation of some rust libraries) https://clang.llvm.org/
- diesel-cli (The ORM we use)
- dbeaver (To view the database content) https://dbeaver.io/


### Database
To use the postgres database you need an ssh tunnel from userspace.
You ssh public key needs to be known by userspace to connect via ssh.
```shell
ssh -L 5432:127.0.0.1:5432 ropelab@betelgeuse.uberspace.de
```

To debug the database you can use [dbeaver](https://dbeaver.io/)
```shell
dbeaver
```

### Backend
The backend is written in rust. 
Just run it compiles everything and start the backend on http://localhost:3001/swagger-ui/
```shell
cargo run
```

### ORM with diesel

#### Create new migrations
```shell
diesel migration generate add_user_new_slot
```

#### Apply migrations
```shell
diesel migration run
```

#### Recreate all migrations
```shell
diesel migration redo
```

## Notes
### Debugging Handler Compile Problems 
add to function `#[debug_handler]`

### Remove unused imports and other stuff 
You need to commit before running fix
```shell
cargo fix
```