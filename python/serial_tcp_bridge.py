import serial
import socket
import threading
import struct
from sensor_pb2 import SensorData

SERIAL_PORT = "COM5"       # change to your port, e.g. /dev/ttyACM0 on Linux/Mac
BAUD_RATE   = 115200
TCP_HOST    = "0.0.0.0"
TCP_PORT    = 12347        # matches simulation port so clients need no changes

clients      = []
clients_lock = threading.Lock()


def broadcast(data: bytes):
    """Send length-prefixed protobuf to all TCP clients (matches sim format)."""
    # Wrap in 4-byte header just like the simulation: int32 length + payload
    framed = struct.pack("<I", len(data)) + data
    with clients_lock:
        dead = []
        for c in clients:
            try:
                c.sendall(framed)
            except OSError:
                dead.append(c)
        for d in dead:
            clients.remove(d)


def read_exact(ser: serial.Serial, n: int) -> bytes:
    buf = b""
    while len(buf) < n:
        chunk = ser.read(n - len(buf))
        if not chunk:
            raise IOError("Serial read timeout — connection lost?")
        buf += chunk
    return buf


def serial_reader(ser: serial.Serial):
    while True:
        try:
            # 2-byte length prefix from Arduino
            header = read_exact(ser, 2)
            (pkt_len,) = struct.unpack("<H", header)

            if pkt_len == 0 or pkt_len > 4096:
                print(f"[WARN] Suspicious packet length {pkt_len}, skipping")
                continue

            full_packet = read_exact(ser, pkt_len)

            

            # Validate message length field
            msg_length = struct.unpack_from("<H", full_packet, 2)[0]
            if msg_length != pkt_len - 4:
                print(f"[WARN] Length mismatch: header says {msg_length}, got {pkt_len - 4}")
                continue

            # Protobuf payload starts at byte 4
            proto_bytes = full_packet[4:]

            try:
                sensor = SensorData()
                sensor.ParseFromString(proto_bytes)
                print(
                    f"[SENSOR] ts={sensor.timestamp:.2f} "
                    f"CO2={sensor.co2} "
                    f"Temp={sensor.bme_temperature:.1f}C "
                    f"Humidity={sensor.bme_humidity:.1f}% "
                    f"Pressure={sensor.bme_pressure:.1f}hPa "
                    f"Altitude={sensor.bme_altitude:.1f}m"
                )
                for i, row in enumerate(sensor.row):
                    temps = ", ".join(f"{t:.1f}" for t in row.pixel_temp)
                    print(f"  thermal row[{i}]: {temps}")
            except Exception as e:
                print(f"[ERROR] Protobuf decode: {e}")
                continue

            # Broadcast to TCP clients in simulation-compatible format
            broadcast(proto_bytes)

        except IOError as e:
            print(f"[ERROR] Serial: {e}")
            break
        except Exception as e:
            print(f"[ERROR] Unexpected in serial_reader: {e}")


def handle_client(conn: socket.socket, addr):
    print(f"[TCP] Client connected: {addr}")
    try:
        while True:
            data = conn.recv(1024)
            if not data:
                break
    except OSError:
        pass
    finally:
        with clients_lock:
            if conn in clients:
                clients.remove(conn)
        conn.close()
        print(f"[TCP] Client disconnected: {addr}")


def main():
    try:
        ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=1)
        print(f"[SERIAL] Opened: {SERIAL_PORT}")
    except serial.SerialException as e:
        print(f"[ERROR] Could not open serial port: {e}")
        return

    threading.Thread(target=serial_reader, args=(ser,), daemon=True).start()

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((TCP_HOST, TCP_PORT))
    server.listen()
    print(f"[TCP] Listening on port {TCP_PORT} (simulation-compatible)")

    while True:
        conn, addr = server.accept()
        with clients_lock:
            clients.append(conn)
        threading.Thread(target=handle_client, args=(conn, addr), daemon=True).start()


if __name__ == "__main__":
    main()