import serial
import socket
import threading
import struct
from sensor_pb2 import SensorData #generated from sensor.proto using protoc compiler, make sure to have the generated file in the same directory or adjust import path accordingly

SERIAL_PORT = "COM5"       # port
BAUD_RATE   = 115200 #must match Arduino baud rate
TCP_HOST    = "0.0.0.0"
TCP_PORT    = 12347        # matches simulation port so clients need no changes

#list of connected TCP clients, protected by a lock for thread safety
clients      = []
clients_lock = threading.Lock()



def broadcast(data: bytes):
    """Send length-prefixed protobuf to all TCP clients (matches sim format)."""
    # Wrap in 4-byte header: int32 length + payload
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

            if full_packet[0] != 0xAA or full_packet[1] != 0x55:
                print(f"[WARN] Bad sync marker: {full_packet[0]:02X} {full_packet[1]:02X}")
                continue
  
            # Validate message length field
            msg_length = struct.unpack_from("<H", full_packet, 2)[0]
            if msg_length != pkt_len - 4:
                print(f"[WARN] Length mismatch: header says {msg_length}, got {pkt_len - 4}")
                continue

            # Protobuf payload starts at byte 4
            proto_bytes = full_packet[4:]
            #decode protobuf for logging, but ignore errors since we still want to broadcast raw bytes to clients even if decoding fails
            try:
                sensor = SensorData()
                sensor.ParseFromString(proto_bytes)
                print(
                    f"[SENSOR] ts={sensor.timestamp:.2f} "
                    f"CO2={sensor.co2} "
                    f"Temp={sensor.bme_temperature:.1f}C "
                    f"Pressure={sensor.bme_pressure:.1f}hPa "
                    f"Altitude={sensor.bme_altitude:.1f}m "
                    f"Humidity={sensor.bme_humidity:.1f}% "
                )
                #print thermal rows
                for i, row in enumerate(sensor.row):
                    temps = ", ".join(f"{t:.1f}" for t in row.pixel_temp)
                    print(f"  thermal row[{i}]: {temps}")
            except Exception as e:
                print(f"[ERROR] Protobuf decode: {e}")
                continue

            # Broadcast to TCP clients in simulation-compatible format
            broadcast(proto_bytes)

        except IOError as e:
            #serial read timeout or disconnect
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
                break # client disconnected
    except OSError:
        pass #client disconnected
    finally:
        with clients_lock:
            if conn in clients:
                clients.remove(conn)
        conn.close()
        print(f"[TCP] Client disconnected: {addr}")


def main():
        # open serial port
    try:
        ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=1)
        print(f"[SERIAL] Opened: {SERIAL_PORT}")
    except serial.SerialException as e:
        print(f"[ERROR] Could not open serial port: {e}")
        return


    #start serial reader thread
    threading.Thread(target=serial_reader, args=(ser,), daemon=True).start()

    #start TCP server
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((TCP_HOST, TCP_PORT))
    server.listen()
    print(f"[TCP] Listening on port {TCP_PORT} (simulation-compatible)")

    #accept clients in main thread, spawn handler threads for each
    while True:
        conn, addr = server.accept()
        with clients_lock:
            clients.append(conn)
        threading.Thread(target=handle_client, args=(conn, addr), daemon=True).start()


if __name__ == "__main__":
    main()