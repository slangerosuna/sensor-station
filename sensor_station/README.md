# IOT4AG Sensor Station Challenge
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)

[![github](https://img.shields.io/badge/GitHub-ucmercedrobotics-181717.svg?style=flat&logo=github)](https://github.com/ucmercedrobotics)
[![website](https://img.shields.io/badge/Website-UCMRobotics-5087B2.svg?style=flat&logo=telegram)](https://robotics.ucmerced.edu/)
[![python](https://img.shields.io/badge/Python-3.10.12-3776AB.svg?style=flat&logo=python&logoColor=white)](https://www.python.org)
[![pre-commits](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white)](https://github.com/pre-commit/pre-commit)
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
[![Checked with mypy](http://www.mypy-lang.org/static/mypy_badge.svg)](http://mypy-lang.org/)
<!-- TODO: work to enable pydocstyle -->
<!-- [![pydocstyle](https://img.shields.io/badge/pydocstyle-enabled-AD4CD3)](http://www.pydocstyle.org/en/stable/) -->

<!-- [![arXiv](https://img.shields.io/badge/arXiv-2409.04653-b31b1b.svg)](https://arxiv.org/abs/2409.04653) -->

## How to Start
Build your container:
```bash
make build-image
```

To start the sensor emulator:
```bash
make prod
```
This will begin a `protobuf` stream over TCP port 12347.

## Simulation Format
The message format is fairly simple. It's serialized in `protobuf` with a 4 byte header that contains length of expected packet.

### TCP Message Format - Header and Payload

| Field                     | Type       | Byte Offset | Field Length | Description                                               |
|---------------------------|------------|-------------|--------------|-----------------------------------------------------------|
| **Header (4 bytes)**       | `int32`    | 0           | 4 bytes      | The 4-byte header representing packet length.             |
| **Payload (Serialized)**   | -          | 4           | Varies       | The remaining fields, serialized in Protobuf format.      |

---

### Protobuf Format - Contents (Serialized)

| Field                     | Type       | Field Number | Byte Offset | Field Length | Description                                               |
|---------------------------|------------|--------------|-------------|--------------|-----------------------------------------------------------|
| **timestamp**              | `float`    | 1            | 4           | 4 bytes      | Seconds from epoch (sim) or from boot (target)           |
| **co2**                    | `int32`    | 2            | 8           | 4 bytes      | CO2 concentration (int32)                                |
| **bme_temperature**        | `float`    | 3            | 12          | 4 bytes      | Temperature from the BME sensor (float)                  |
| **bme_pressure**           | `float`    | 4            | 16          | 4 bytes      | Pressure from the BME sensor (float)                     |
| **bme_altitude**           | `float`    | 5            | 20          | 4 bytes      | Altitude from the BME sensor (float)                     |
| **bme_humidity**           | `float`    | 6            | 24          | 4 bytes      | Humidity from the BME sensor (float)                     |
| **thermal (repeated)**     | `float[]`  | 7            | 28          | Varies       | Array of thermal sensor data (list of floats)            |


## Developers
If you are intending on emulating specific behavior for your challenge, feel free to edit the emulators.
The logic behind them is a random uniform distribution of sensor data, so it is very limited and only intended to get you started.
For your own challenge, you can update anything you want.
For example, you might want to test out specific thermal images.
To do this, you can edit the `ThermalCameraSensorEmulator()` with your own custom logic to generate a photo.
However, it is recommended to not touch the `protobuf` as this format will remain consistent on the target platform (LoRa).

## Target Format
When moving to work on target, the message format will be similar.
However, the protocol will no longer be TCP, but instead LoRa.
You will need a hardware device to read this connection, which will be provided.

You will write the receiving LoRa driver on your own.

The sensor station will transmit a packet over LoRa that contains the data. A packet consists of a 4 byte header (sync marker & message length) followed by 
the protobuf encoded sensor data. The sensor station will send the packet in multiple chunks. Each chunk is between 2 - 255 bytes and consists of a 2 byte 
header (chunk index & total chunks) followed by a portion of the packet. The chunks will need to be reassembled to get the whole packet, which can then be 
decoded using the protobuf schema.

To write the receiving LoRa driver in Arduino, you will need the 'LoRa' library and to use the LoRa frequency 915E6.

### Sensor Station Packet Format - Contents
| Field                     | Type         | Byte Offset | Field Length | Description                                                     |
|---------------------------|--------------|-------------|--------------|-----------------------------------------------------------------|
| **Sync Marker**           | `uint8[]`    | 0           | 2 bytes       | Fixed bytes to mark start of a packet.                         |
| **Message Length**        | `uint8[]`    | 2           | 2 bytes       | Total number of bytes in encoded sensor data (little endian).  |
| **Packet Payload**        | `byte[]`     | 4           | ? bytes       | The entire protobuf encoded sensor data.                       |


### LoRa Chunk Format - Contents
| Field                     | Type         | Byte Offset | Field Length | Description                                               |
|---------------------------|--------------|-------------|--------------|-----------------------------------------------------------|
| **Chunk Index**           | `uint8`      | 0           | 1 byte       | Current chunk number (indexed from 0)                     |
| **Total Chunks**          | `uint8`      | 1           | 1 byte       | Total number of chunks in the packet.                     |
| **Chunk Payload**         | `byte[]`     | 2           | 0-253 bytes  | A portion of the full packet                              |

### Protobuf Format - Payload (Serialized)
The `chuck payload` from above is the below message broken up into 253 byte chunks until decomposed.

| Field                     | Type       | Field Number | Byte Offset | Field Length | Description                                               |
|---------------------------|------------|--------------|-------------|--------------|-----------------------------------------------------------|
| **timestamp**              | `float`    | 1            | 4           | 4 bytes      | Seconds from epoch (sim) or from boot (target)           |
| **co2**                    | `int32`    | 2            | 8           | 4 bytes      | CO2 concentration (int32)                                |
| **bme_temperature**        | `float`    | 3            | 12          | 4 bytes      | Temperature from the BME sensor (float)                  |
| **bme_pressure**           | `float`    | 4            | 16          | 4 bytes      | Pressure from the BME sensor (float)                     |
| **bme_altitude**           | `float`    | 5            | 20          | 4 bytes      | Altitude from the BME sensor (float)                     |
| **bme_humidity**           | `float`    | 6            | 24          | 4 bytes      | Humidity from the BME sensor (float)                     |
| **thermal (repeated)**     | `float[]`  | 7            | 28          | Varies       | Array of thermal sensor data (list of floats)            |
