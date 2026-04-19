import time
import socket
import argparse

import yaml

from sensors.co2 import CO2SensorEmulator
from sensors.temperature import TempSensorEmulator
from sensors.pressure import PressureSensorEmulator
from sensors.altitude import AltitudeSensorEmulator
from sensors.humidity import HumiditySensorEmulator
from sensors.thermalcamera import ThermalCameraSensorEmulator

import protobuf.sensor_pb2 as sensor_pb2

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Sensor Station Argparser")
    parser.add_argument(
        "--config",
        type=str,
        help="Sensor Station YAML config file",
        default="/sensor-station/app/config/config.yaml",
    )
    args = parser.parse_args()

    with open(args.config, "r") as file:
        config = yaml.safe_load(file)

    # initialize TCP server socket
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server_socket.bind((config["host"], config["port"]))
    server_socket.listen()

    temp_sensor = TempSensorEmulator()
    co2_sensor = CO2SensorEmulator()
    pressure_sensor = PressureSensorEmulator()
    altitude_sensor = AltitudeSensorEmulator()
    humidity_sensor = HumiditySensorEmulator()
    thermal_camera_sensor = ThermalCameraSensorEmulator()

    try:
        while True:
            connection, _ = server_socket.accept()
            print("Client connected to Sensor Station")
            with connection:
                while True:
                    sensor_data = sensor_pb2.SensorData()

                    time.sleep(1)
                    temp = temp_sensor.read_temp()
                    co2 = co2_sensor.read_co2()
                    pressure = pressure_sensor.read_pa()
                    alt = altitude_sensor.read_alt()
                    humidity = humidity_sensor.read_rh()
                    thermal_image = thermal_camera_sensor.read_image()
                    # protobuf construction
                    sensor_data.timestamp = time.time()
                    sensor_data.co2 = co2
                    sensor_data.bme_temperature = temp
                    sensor_data.bme_pressure = pressure
                    sensor_data.bme_altitude = alt
                    sensor_data.bme_humidity = humidity
                    for row in thermal_image:
                        new_row = sensor_data.row.add()
                        new_row.pixel_temp.extend(row)

                    encoded_data = sensor_data.SerializeToString()

                    # header data of packet length
                    msg_length = len(encoded_data).to_bytes(4, byteorder="big")

                    try:
                        connection.sendall(msg_length + encoded_data)
                    except (BrokenPipeError, ConnectionResetError, OSError):
                        break
            print("Client disconnected from Sensor Station")
    finally:
        server_socket.close()
