import socket
import time
import struct
import random
from threading import Thread

class MinecraftBedrockServer:
    def __init__(self, host='0.0.0.0', port=19132):
        self.host = host
        self.port = port
        self.socket = None
        self.running = False
        self.server_guid = random.randint(0, 2**64 - 1)
        self.motd = {
            'motd1': '§bPython §eRakNet §aServer',
            'motd2': '§dSimple MOTD Example',
            'game_mode': 'Survival',
            'game_version': '1.21.73',  # Updated version
            'protocol_version': '786',  # Protocol version for latest Bedrock
            'max_players': 50,
            'current_players': 0
        }

    def start(self):
        """Start the server"""
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind((self.host, self.port))
        self.running = True
        print(f"Server started on {self.host}:{self.port}")

        # Start listening thread
        listen_thread = Thread(target=self.listen)
        listen_thread.daemon = True
        listen_thread.start()

        try:
            while self.running:
                time.sleep(1)
        except KeyboardInterrupt:
            self.stop()

    def stop(self):
        """Stop the server"""
        self.running = False
        if self.socket:
            self.socket.close()
        print("Server stopped")

    def listen(self):
        """Listen for incoming packets"""
        while self.running:
            try:
                data, addr = self.socket.recvfrom(1024)
                self.handle_packet(data, addr)
            except socket.error:
                break

    def handle_packet(self, data, addr):
        # This doc is provided by sauoro for a better understanding of RakNet
        # If you d0n't understand, remember to read slowly and check step-by-step how we handle it

        if not data:
            return

        print("===== Received a Packet =====")

         # Receive Client's IP & Address
        print(f" - From MC Client: {addr}")

         # Bytes sent by Minecraft Client automatically (could be any)
        print(f" - Packet Bytes (hex): {data.hex()}")

        # Unconnected ping detection (0x01 = ping)
        if data[0] == 0x01:

            # We convert the data into a hex
            raw_hex = data.hex()

            # Not important, this is to explain something
            formatted_hex = ""
            for i in range(0, len(raw_hex), 2):
                byte_hex = raw_hex[i:i+2]
                formatted_hex += "\\x" + byte_hex

            print(f"===== Received Unconnected Ping =====")

            # Receive Client's IP & Address
            print(f"From Client: {addr}")

            # 01000000001852088900ffff00fefefefefdfdfdfd12345678b4ea50e8f062a462
            # This is sent by Minecraft Client
            print(f" - Original raw packet (hex): {raw_hex}")

            # This is how we would type it here in python with (b'\x01')
            #
            # This is an example in Python:
            # Example: expected_magic = b'\x00\xff\xff\x00\xfe\xfe\xfe\xfe\xfd\xfd\xfd\xfd\x12\x34\x56\x78'
            #
            # This is what we receive from this print
            #  - Formatted hex packet: \x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\xfe\xfe\xfe\xfe\xfd\xfd\xfd\xfd\x12\x34\x56\x78\xb4\xea\x50\xe8\xf0\x62\xa4\x62
            # This is sent by Minecraft Client
            print(f" - Formatted hex packet: {formatted_hex}")

             # Handle Unconnected Ping to Handle
            self.handle_unconnected_ping(data, addr)

    def handle_unconnected_ping(self, data, addr):
        """Handle unconnected ping packet"""
        if len(data) < 33:  # Minimum valid ping packet length
            print(f"Invalid ping packet, length: {len(data)}")
            return

        # Extract client time and check magic
        ping_time = struct.unpack('>Q', data[1:9])[0]
        print(f"Client ping time: {ping_time}")
        magic = data[9:25]  # RakNet magic
        print(f"Received magic: {magic.hex()}")
        client_guid = struct.unpack('>Q', data[25:33])[0]
        print(f"Client GUID: {client_guid}")

        # Verify this is a valid RakNet packet by checking magic
        expected_magic = b'\x00\xff\xff\x00\xfe\xfe\xfe\xfe\xfd\xfd\xfd\xfd\x12\x34\x56\x78'
        if magic != expected_magic:
            print("Invalid magic bytes, ignoring packet")
            return

        # Create the MOTD string - EXACT order and format matters!
        motd_data = ';'.join([
            'MCPE',  # Must start with MCPE
            self.motd['motd1'],
            self.motd['protocol_version'],
            self.motd['game_version'],
            str(self.motd['current_players']),
            str(self.motd['max_players']),
            str(self.server_guid),
            self.motd['motd2'],
            self.motd['game_mode'],
            '1',  # GameMode ID (1 = Survival)
            '19132',  # IPv4 port
            '19133',  # IPv6 port
        ])

        # Create unconnected pong packet
        pong_packet = bytearray()
        pong_packet.append(0x1c)  # Unconnected Pong packet ID
        pong_packet.extend(struct.pack('>Q', ping_time))  # Client ping time
        pong_packet.extend(struct.pack('>Q', self.server_guid))  # Server GUID
        pong_packet.extend(magic)  # RakNet magic
        pong_packet.extend(struct.pack('>H', len(motd_data)))  # String length
        pong_packet.extend(motd_data.encode('utf-8'))  # MOTD string

        # Send the pong packet
        self.socket.sendto(pong_packet, addr)
        print(f"Sent pong response to {addr}")
        print(f"MOTD data: {motd_data}")

    def set_motd(self, motd1=None, motd2=None, game_mode=None, max_players=None, current_players=None, game_version=None, protocol_version=None):
        """Update server MOTD information"""
        if motd1:
            self.motd['motd1'] = motd1
        if motd2:
            self.motd['motd2'] = motd2
        if game_mode:
            self.motd['game_mode'] = game_mode
        if max_players is not None:
            self.motd['max_players'] = max_players
        if current_players is not None:
            self.motd['current_players'] = current_players
        if game_version:
            self.motd['game_version'] = game_version
        if protocol_version:
            self.motd['protocol_version'] = protocol_version

if __name__ == "__main__":
    # Example usage
    server = MinecraftBedrockServer(port=19132)

    # Customize your MOTD (optional - defaults are already set)
    server.set_motd(
        motd1="§bPython §eRakNet §aServer",
        motd2="§dSimple MOTD Example",
        game_mode="Survival",
        max_players=50,
        game_version="1.21.73",
        protocol_version="786"
    )

    # Start the server
    try:
        server.start()
    except KeyboardInterrupt:
        server.stop()