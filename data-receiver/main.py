import serial
import serial.tools.list_ports
import sys
import time
import os
import threading

BAUD_RATE = 115200
LOG_DIR = os.path.dirname(os.path.abspath(__file__))

stop_event = threading.Event()

def find_port():
    ports = serial.tools.list_ports.comports()
    for p in ports:
        if "USB" in p.description or "tty" in p.device or "COM" in p.device:
            return p.device
    return None

def stdout_open():
    try:
        os.fstat(sys.stdout.fileno())
        return True
    except (OSError, ValueError):
        return False

def monitor(port: str, log_path: str):
    with serial.Serial(port, BAUD_RATE, timeout=0) as ser, open(log_path, "w", buffering=1) as log:
        time.sleep(2)
        ser.write(b"START")
        log.write(">> START\n")
        if stdout_open():
            print(">> START", flush=True)

        buffer = bytearray()
        while not stop_event.is_set():
            data = ser.read(4096)
            if data:
                buffer.extend(data)
                while b"\n" in buffer:
                    line, buffer = buffer.split(b"\n", 1)
                    decoded = line.decode("utf-8", errors="replace").rstrip()
                    log.write(decoded + "\n")
                    if stdout_open():
                        print(decoded, flush=True)
                    if decoded == "=== Experiment Complete ===":
                        stop_event.set()
                        return

def main():
    port = find_port()
    if port is None:
        if len(sys.argv) > 1:
            port = sys.argv[1]
        else:
            sys.exit(1)

    if len(sys.argv) > 1:
        port = sys.argv[1]

    log_path = os.path.join(LOG_DIR, "data.log")

    print(f"pid={os.getpid()}", flush=True)

    t = threading.Thread(target=monitor, args=(port, log_path), daemon=True)
    t.start()

    try:
        while t.is_alive():
            t.join(timeout=1)
    except KeyboardInterrupt:
        stop_event.set()
        t.join()

    # After monitor thread ends, run the parser
    from parser import parse
    parse('./data.log')

if __name__ == "__main__":
    main()