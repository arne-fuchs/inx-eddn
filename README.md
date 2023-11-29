# Inx-EDDN
Reads from the eddn network and puts the data into blocks, which will be posted into a tangle network.

## Environment Variables

| Env. Variable     | Example                 | Description                                                                                 |
|-------------------|-------------------------|---------------------------------------------------------------------------------------------|
| NODE_URL          | https://api.edcas.de    | URL to your hornet node you want to send data to                                            |
| NUM_OF_WORKERS    | 4                       | How many threads the app is able to occupy for pow                                          |
| WALLET_PASSWORD   | testpassword            | Password for your stronghold wallet                                                         |
| WALLET_PATH       | wallet.stronghold       | Path of your stronghold wallet                                                              |
| ZEROMQ_URL        | tcp://eddn.edcd.io:9500 | Url to the ZeroMQ instance                                                                  |
| KEY_SAVE_LOCATION | publickey               | File location where the public key should be saved to (Only affect with --saveKey argument) |

## Program Arguments

```--saveKey```

Saves the public key to a file. The file location is defined by KEY_SAVE_LOCATION environment variable.


## Docker Compose Example

### For generating and saving key

```dockerfile
  generate_public_key:
    container_name: inx-eddn
    image: frankthefish/inx-eddn:latest
    environment:
      - "NODE_URL=https://api.edcas.de:443"
      - "ZEROMQ_URL=tcp://eddn.edcd.io:9500"
      - "WALLET_PASSWORD=testwallet"
      - "WALLET_PATH=wallet/wallet.stronghold"
      - "NUM_OF_WORKERS=4"
      - "KEY_SAVE_LOCATION=public_key/public_key"
    volumes:
      - ./inx-eddn/wallet:/app/wallet
      - ./inx-eddn/public_key:/app/public_key
```

### For executing normally

```dockerfile
  inx-eddn:
    container_name: inx-eddn
    image: frankthefish/inx-eddn:latest
    environment:
      - "NODE_URL=https://api.edcas.de:443"
      - "ZEROMQ_URL=tcp://eddn.edcd.io:9500"
      - "WALLET_PASSWORD=testwallet"
      - "WALLET_PATH=wallet/wallet.stronghold"
      - "NUM_OF_WORKERS=4"
    volumes:
      - ./inx-eddn/wallet:/app/wallet
```