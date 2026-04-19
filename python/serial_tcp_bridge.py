import serial
import socket
import threading
import struct
from sensor_pb2 import SensorData

SERIAL_PORT = "COM5"
BAUD_RATE   = 115200
TCP_HOST    = "0.0.0.0"
TCP_PORT    = 5005

clients      = []
clients_lock = threading.Lock()

def broadcast(data):
    with clients_lock:
        dead = []
        for c in clients:
            try:
                c.sendall(data)
            except:
                dead.append(c)
        for d in dead:
            clients.remove(d)

def read_exact(ser, n):
    buf = b""
    while len(buf) < n:
        chunk = ser.read(n - len(buf))
        if chunk:
            buf += chunk
    return buf

def serial_reader(ser):
    while True:
        # Read 2-byte length prefix from Arduino
        header = read_exact(ser, 2)
        (pkt_len,) = struct.unpack("<H", header)

        if pkt_len == 0 or pkt_len > 4096:
            continue

        # Read full packet
        full_packet = read_exact(ser, pkt_len)

        # Validate 4-byte packet header
        msg_length = struct.unpack_from("<H", full_packet, 2)[0]
        if msg_length != pkt_len - 4:
            print(f"Length mismatch: expected {msg_length}, got {pkt_len - 4}")
            continue

        # Protobuf payload starts at byte 4
        proto_bytes = full_packet[4:]

        # Decode protobuf
        try:
            sensor = SensorData()
            sensor.ParseFromString(proto_bytes)
            print(f"[SENSOR] ts={sensor.timestamp:.2f} "
                  f"CO2={sensor.co2} "
                  f"Temp={sensor.bme_temperature:.1f}C "
                  f"Humidity={sensor.bme_humidity:.1f}% "
                  f"Pressure={sensor.bme_pressure:.1f}hPa "
                  f"Altitude={sensor.bme_altitude:.1f}m")

            for i, row in enumerate(sensor.row):
                temps = ", ".join(f"{t:.1f}" for t in row.pixel_temp)
                print(f"  thermal row[{i}]: {temps}")

        except Exception as e:
            print(f"Protobuf decode error: {e}")
            continue

        # Broadcast raw proto bytes to TCP clients
        broadcast(proto_bytes)

def handle_client(conn):
    print("Client connected")
    try:
        while True:
            if not conn.recv(1024):
                break
    except:
        pass
    finally:
        with clients_lock:
            if conn in clients:
                clients.remove(conn)
        conn.close()

def main():
    try:
        ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=1)
        print(f"Opened serial: {SERIAL_PORT}")
    except serial.SerialException as e:
        print(f"Failed to open serial port: {e}")
        return

    threading.Thread(target=serial_reader, args=(ser,), daemon=True).start()

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((TCP_HOST, TCP_PORT))
    server.listen()
    print(f"TCP server on port {TCP_PORT}")

    while True:
        conn, addr = server.accept()
        print("Client:", addr)
        with clients_lock:
            clients.append(conn)
        threading.Thread(target=handle_client, args=(conn,), daemon=True).start()

if __name__ == "__main__":
    main()