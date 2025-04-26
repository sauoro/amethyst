# These lines are like telling Python which toy boxes (libraries) we need to use.
import socket  # This box has tools for talking over the internet (like a walkie-talkie).
import time    # This box has tools for waiting (like counting seconds).
import struct  # This box has tools to pack and unpack secret computer codes (like a decoder ring).
import random  # This box has tools for picking random numbers (like rolling dice).
from threading import Thread # This box lets the robot do two things at once (like patting its head and rubbing its tummy).

# Let's build our friendly Minecraft Server Robot!
class MinecraftBedrockServer:
    # This is the instruction manual for building the robot (__init__ means "initialize" or "set up")
    def __init__(self, host='0.0.0.0', port=19132):
        """
        When we build the robot, we tell it where to listen for messages.
        Imagine the robot has an address (host) and a special door number (port).
        '0.0.0.0' means the robot will listen at *all* its doors on its street.
        19132 is the special door number Minecraft usually knocks on.
        """
        self.host = host  # The robot remembers its address.
        self.port = port  # The robot remembers its special door number.
        self.socket = None # The robot doesn't have its walkie-talkie turned on yet.
        self.running = False # The robot is not switched on yet.
        # The robot needs a unique secret name tag (GUID) so Minecraft knows who it is.
        # We use the 'random' toy box to give it a super big, random number name tag.
        self.server_guid = random.randint(0, 2**64 - 1)
        # This is like a little sign the robot holds up for Minecraft players to see.
        # It tells them the server name, rules, and how many friends are playing.
        self.motd = {
            'motd1': '§bPython §eRakNet §aServer', # First line of the welcome sign (with colors!)
            'motd2': '§dSimple MOTD Example',    # Second line of the welcome sign (with colors!)
            'game_mode': 'Survival',           # What game mode are we playing?
            'game_version': '1.21.73',         # Which version of Minecraft is this for?
            'protocol_version': '786',         # A secret code number for the Minecraft version.
            'max_players': 50,                 # How many friends can play at once?
            'current_players': 0               # How many friends are playing right now? (Starts at 0)
        }

    # This tells the robot how to start working.
    def start(self):
        """
        Flip the switch ON for the robot!
        It gets its walkie-talkie ready and starts listening.
        """
        # Get the walkie-talkie ready from the 'socket' toy box.
        # AF_INET means internet addresses (like house numbers).
        # SOCK_DGRAM means using quick messages (like UDP postcards, not phone calls).
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        # Tell the walkie-talkie to use the robot's address and door number.
        self.socket.bind((self.host, self.port))
        # Flip the "running" switch to ON!
        self.running = True
        # Print a message saying the robot is ready!
        print(f"Robot started listening at {self.host} on door {self.port}")

        # The robot needs help! It asks a helper (Thread) to do the listening part.
        # This way, the main robot can wait patiently without being busy listening all the time.
        listen_thread = Thread(target=self.listen) # Tell the helper which job to do (listen).
        listen_thread.daemon = True # If the main robot stops, the helper stops too.
        listen_thread.start() # Tell the helper to start working!

        # The main robot now just waits. It will keep waiting until someone tells it to stop.
        try:
            while self.running:
                # Just wait for one second, then check again if it should still be running.
                time.sleep(1)
        # If someone presses Ctrl+C on the keyboard, it's like a signal to stop.
        except KeyboardInterrupt:
            self.stop() # Call the stop instructions.

    # This tells the robot how to stop working.
    def stop(self):
        """Flip the switch OFF for the robot."""
        # Flip the "running" switch to OFF.
        self.running = False
        # If the walkie-talkie is on...
        if self.socket:
            self.socket.close() # ...turn it off.
        # Print a message saying the robot stopped.
        print("Robot stopped.")

    # This is the helper's job: listening for messages.
    def listen(self):
        """The helper robot listens carefully for incoming messages (packets)."""
        # Keep listening as long as the main robot's switch is ON.
        while self.running:
            try:
                # Wait here until a message (data) arrives on the walkie-talkie.
                # Also find out who sent it (addr - their address and door number).
                # 1024 is like saying "don't listen for messages longer than this".
                data, addr = self.socket.recvfrom(1024)
                # Once a message is received, give it to the main robot to handle.
                self.handle_packet(data, addr)
            # If something goes wrong with the walkie-talkie (like it gets turned off)...
            except socket.error:
                break # ...stop listening.

    # This tells the robot what to do when it gets a message (packet).
    def handle_packet(self, data, addr):
        """
        The robot looks at the message it received.
        It's like opening a postcard and reading it.
        """
        # If the message is empty, do nothing.
        if not data:
            return

        # Let's print some information so we can see what's happening.
        print("===== Robot received a Message! =====")
        # Show who sent the message.
        print(f" - From Minecraft game at: {addr}")
        # Show the message itself in computer code (hexadecimal).
        print(f" - Message code (hex): {data.hex()}")

        # Look at the VERY FIRST byte (number) of the message.
        # This first byte tells us what kind of message it is.
        # If the first byte is 0x01 (which is 1 in normal numbers), it's a "ping" message.
        # A "ping" is like the Minecraft game shouting "Hello? Is anyone there?".
        if data[0] == 0x01:

            # --- These prints help us understand the message code ---
            raw_hex = data.hex() # The computer code as one long string.
            formatted_hex = ""   # We'll make a version that's easier to read.
            for i in range(0, len(raw_hex), 2): # Look at the code two letters at a time.
                byte_hex = raw_hex[i:i+2]
                formatted_hex += "\\x" + byte_hex # Put "\x" before each pair.
            print(f"===== It's a 'Hello?' (Ping) Message! =====")
            print(f"From Game: {addr}")
            print(f" - Original message code (hex): {raw_hex}")
            print(f" - Easier-to-read code: {formatted_hex}")
            # --- End of explanation prints ---

            # Since it's a "ping", let's give it to the special part of the robot
            # that knows how to answer pings.
            self.handle_unconnected_ping(data, addr)

    # This is the special part that answers "Hello?" (ping) messages.
    def handle_unconnected_ping(self, data, addr):
        """
        The robot answers the "Hello?" message with "Yes, I'm here!".
        This is called a "pong" response.
        """
        # A proper "ping" message should have at least 33 bytes (letters/numbers in code).
        # If it's too short, it might be a broken message.
        if len(data) < 33:
            print(f"Uh oh, the 'Hello?' message seems too short ({len(data)} bytes). Ignoring it.")
            return

        # Use the 'struct' decoder ring to read parts of the message.
        # '>Q' means read a big number (64 bits) in a standard way.

        # Bytes 1 to 8: The time the game sent the message. We need to send this back.
        ping_time = struct.unpack('>Q', data[1:9])[0]
        print(f"Game's clock time: {ping_time}")

        # Bytes 9 to 24: A secret handshake code (RakNet magic).
        # This proves the message is really from a Minecraft game using the RakNet language.
        magic = data[9:25]
        print(f"Secret Handshake received: {magic.hex()}")

        # Bytes 25 to 32: The game's unique secret name tag (Client GUID).
        client_guid = struct.unpack('>Q', data[25:33])[0]
        print(f"Game's secret name tag: {client_guid}")

        # Check if the secret handshake code is correct.
        expected_magic = b'\x00\xff\xff\x00\xfe\xfe\xfe\xfe\xfd\xfd\xfd\xfd\x12\x34\x56\x78'
        if magic != expected_magic:
            print("Wrong secret handshake! Ignoring this message.")
            return

        # Now, prepare the answer message (the "pong").
        # First, create the message that goes on the server list sign.
        # It needs to be in a VERY specific order, with semicolons (;) separating parts.
        motd_data = ';'.join([
            'MCPE',                      # Always start with MCPE
            self.motd['motd1'],          # The first line of the server name
            self.motd['protocol_version'],# The secret code version number
            self.motd['game_version'],   # The Minecraft version name
            str(self.motd['current_players']), # How many friends are playing (as text)
            str(self.motd['max_players']),     # How many friends can play (as text)
            str(self.server_guid),       # The robot's secret name tag (as text)
            self.motd['motd2'],          # The second line of the server name
            self.motd['game_mode'],      # The game mode name
            '1',                         # Game mode number (1 is Survival)
            '19132',                     # The door number for normal internet (IPv4)
            '19133',                     # The door number for newer internet (IPv6)
        ])

        # Now build the actual "pong" reply message byte-by-byte.
        # It's like writing a postcard in computer code.
        pong_packet = bytearray() # Start with an empty message.

        # 1. Add the message type code: 0x1c means "Unconnected Pong".
        pong_packet.append(0x1c)

        # 2. Add the game's clock time (we read this earlier). Use the decoder ring to pack it.
        pong_packet.extend(struct.pack('>Q', ping_time))

        # 3. Add the robot's secret name tag (Server GUID). Pack it.
        pong_packet.extend(struct.pack('>Q', self.server_guid))

        # 4. Add the secret handshake code (Magic).
        pong_packet.extend(magic)

        # 5. Add the length (how long?) of the server list sign message.
        #    '>H' means a smaller number (16 bits). Pack it.
        pong_packet.extend(struct.pack('>H', len(motd_data)))

        # 6. Add the actual server list sign message (convert it to bytes).
        pong_packet.extend(motd_data.encode('utf-8'))

        # Send the finished "pong" message back to the game that sent the "ping".
        self.socket.sendto(pong_packet, addr)
        print(f"Sent 'I'm here!' (Pong) message back to {addr}")
        print(f"The message on the sign was: {motd_data}")

    # This lets you change the server's welcome sign information before starting the robot.
    def set_motd(self, motd1=None, motd2=None, game_mode=None, max_players=None, current_players=None, game_version=None, protocol_version=None):
        """
        Update the information on the robot's welcome sign.
        Like changing the writing on a whiteboard.
        """
        # If you provide a new value for motd1, update it.
        if motd1:
            self.motd['motd1'] = motd1
        # If you provide a new value for motd2, update it.
        if motd2:
            self.motd['motd2'] = motd2
        # If you provide a new game_mode, update it.
        if game_mode:
            self.motd['game_mode'] = game_mode
        # If you provide a new max_players number, update it.
        if max_players is not None: # Check it's not empty
            self.motd['max_players'] = max_players
        # If you provide a new current_players number, update it.
        if current_players is not None: # Check it's not empty
            self.motd['current_players'] = current_players
        # If you provide a new game_version, update it.
        if game_version:
            self.motd['game_version'] = game_version
        # If you provide a new protocol_version, update it.
        if protocol_version:
            self.motd['protocol_version'] = protocol_version

# This part only runs if you start *this* Python file directly.
# It's like the main "Play" button for our robot program.
if __name__ == "__main__":
    # Let's build the robot! Tell it which door number to use (19132).
    server = MinecraftBedrockServer(port=19132)

    # Optional: You can change the welcome sign details here if you want.
    # If you don't change them, it will use the defaults we set up earlier.
    server.set_motd(
        motd1="§cFun §ePython §aServer!", # A new first line for the sign
        motd2="§bCome Play!",            # A new second line
        game_mode="Creative",          # Let's make it Creative mode
        max_players=20,                # Only 20 players allowed
        game_version="1.21.73",        # Make sure the version is right
        protocol_version="786"         # Make sure the protocol code is right
    )

    # Now, tell the robot to start working!
    try:
        server.start() # Call the start instructions
    # If you press Ctrl+C while the robot is running...
    except KeyboardInterrupt:
        server.stop() # ...tell the robot to stop nicely.