# portools


# Example Workflows

## Start the Server
```
cargo run
```

## Get
```
 curl -v http://localhost:8080/lots 
```

## Upload a CSV
```
curl -v -X PUT --data-binary @resource/example http://localhost:8080/lots
```

