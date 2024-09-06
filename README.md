# backend

## Setup 

shh tunnel postgres database
```shell
ssh -L 5432:127.0.0.1:5432 ropelab@betelgeuse.uberspace.de
```

## Run
```shell
cargo run
```

## Tools
### Debug database 
```shell
dbeaver
```

### Create new migrations
```shell
diesel migration generate add_deadline_data_to_events
```

### Apply migrations
```shell
diesel migration run
```

### Recreate all migrations
```shell
diesel migration redo
```

### Fixing Handler Compile Problems 
`#[debug_handler]`