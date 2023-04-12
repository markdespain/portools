# portools


# Example Workflows

## Start the Server
For local development and testing, the application can be started up directly using either
Cargo or by building a running a Docker container. 

Notes:
* For rapid developing, running Cargo directly will likely be the quicker option.
* Both examples below will result in the application running and available on localhost:8080.
### Using Cargo
```
cargo run
```

### Using Docker
#### Build The Container (As Needed)
```
./scripts/build_container.sh
```
#### Run The Container
```
./scripts/run_container.sh
```

### Interact with the Server

## Upload a CSV containing Lots
```
curl -v -X PUT --data-binary @resource/example.csv http://localhost:8080/portfolio/1
```

## Get Lots
```
curl -v http://localhost:8080/portfolio/1 
```



