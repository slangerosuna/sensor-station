import serial
import socket
import threading

SERIAL_PORT = "COM5"  # Arduino's port
BAUD_RATE = 9600 #matches serial.begin on arduino code

TCP_HOST = "0.0.0.0" #Listen on all interfaces
TCP_PORT = 5005 #port clients will connect to

clients = [] #list of TCP client sockets
clients_lock = threading.Lock()  #prevent concurrent access to clients list

def broadcast(data):
    #sends data to all connected clients, removing any that have disconnected
    with clients_lock:
        dead = []
        for c in clients:
            try:
                c.sendall(data)
            except:
                dead.append(c) #mark clients that have disconnected
        for d in dead:
            clients.remove(d) #remove dead clients from list

def serial_reader(ser):
    #continuously read from serial and broadcast to clients
    while True:
        data = ser.read(ser.in_waiting or 1) #read available bytes, or block until at least 1 byte is received
        if data:
            broadcast(data)

def handle_client(conn):
    #keep client thread alive until client disconnects
    print("Client connected")
    try:
        while True:
            conn.recv(1)  # keep alive
    except:
        pass
    finally:
        with clients_lock:
            if conn in clients:
                clients.remove(conn)
        conn.close()

def main():
    #open serial connection to Arduino
    ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=1)
    print(f"Opened serial: {SERIAL_PORT}")

    #start serial reader thread
    threading.Thread(target=serial_reader, args=(ser,), daemon=True).start()


#start TCP server
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((TCP_HOST, TCP_PORT))
    server.listen()

    print(f"TCP server running on {TCP_PORT}")
    #accept incoming client connections
    while True:
        conn, addr = server.accept()
        print("Client:", addr)
        with clients_lock:
            clients.append(conn)
        threading.Thread(target=handle_client, args=(conn,), daemon=True).start()

if __name__ == "__main__":
    main()