1. Config .env:
For details, please refer to .env file

2. Run 
```cargo build ```
to generate proto

3. SetUp Postgre Database:
```
cargo run --release --bin migration
```

4. Start restful server:
```
cargo run --release --bin restful-server 
```


# Gateway

## Overview

The gateway is for collecting P2P VLC date periodically and providing APIs for the browser and Chrome extension to query relative data.


## Dependences

### PostgreDB

The gateway depends on postgre db for data persistence, so please install postgre and setup a pg instance.

### Environment Variables Config  
The .env file contains the database configuration and http restful server port and seed node infos.
Please config it before you deploy the gateway with your requirements, for more details please refer [.env](.env).

### Seed Node  
With seed node, gateway can use bfs_traverse to acquire the whole P2P network nodes info and data from VLC.


### Restful APIs

Gateway provides restful apis:  
- /gateway    
  - /overview (provide P2P network nodes brief infos. ex. node ids)
  - /node/:id  (provide single node detailed info. ex. is-alive,clock,message ids )
  - /message/:id (provide single message details info.)
  - /merge_log_by_message_id/:id (provide the relative merge logs of the message)

### Acquire vlc data from P2P

The gateway uses protobuf proto3 as serialization compression algorithm and communication protocol for querying from P2P, More messages body details, please see [src/proto](src/proto/) for check it.

## Compile

### Build from source

```bash
git clone https://github.com/NagaraTech/gateway.git

cargo build

```

## Run a node

```bash

# 1.init db & pg tables
./target/debug/migration

# 3.setup gateway
./target/debug/restful-server 

```

## How to test


Python sctipt: 
[test.py](test.py) 


