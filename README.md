# portools


# Example Workflows

## Upload a CSV
```
cargo run
curl -v -X PUT --data-binary @resource/example http://localhost:8080/lots
```

## Get 
```
 curl -v http://localhost:8080/lots 
```