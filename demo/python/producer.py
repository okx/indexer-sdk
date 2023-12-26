import zmq
import time

context = zmq.Context()
publisher = context.socket(zmq.PUB)
publisher.bind("tcp://127.0.0.1:5555")

while True:
    message = "Hello, World!"
    print(f"Publishing: {message}")
    publisher.send_string(message)
    time.sleep(5)
