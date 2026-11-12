#set text(
  size: 12pt
)

#align(center)[
  #title[Minecraft MITM Context Tool]
  Dr. Selvan Snow | CS 513 | Worcester Polytechnic Institute\
  By: Harrison DiAmbrosio and Adrian Cervera \
  #link("https://github.com/EggbertFluffle/mc_mitm")
]

= Abstract

The official _Minecraft: Java Edition_ server communicates with game clients using a custom protocol over TCP. Through a community maintained protocol specification, a man-in-the-middle attack tool, a custom protocol parser, and a web server, we have constructed a tool that can sniff packets and accumulate game state context over time. This context is then presented live by means of a bespoke web UI, providing a top-down, interactive map of the game world, including entities, and an action log containing key player interactions. The tool is designed to be used in combination with a man-in-the-middle utility in order to intercept and gain context on game interactions.

= Introduction

== _Minecraft_ Protocol

A simple introduction to the _Minecraft: Java Edition_ protocol will give much needed context to the remainder of the project. Though not an official specification, the community wiki for the game contains a comprehensive spec of the protocol, including the latest official version of the game, `26.3`.

The protocol specifies that all packets must begin with a `length` field, followed by an `id` field. Generally, the protocol defines a large list of types [1], and how they are decoded. Furthermore, the protocol operates in 5 phases, each phase having conditions that signal the client and server to move to another phase. Finally packets are also interpreted differently according to their direction: serverbound or clientbound. The combination of `id`, protocol phase and packet direction allow us to decode packet's entirely according to the community specification.

#figure(caption: [The 5 phases of the _Minecraft: Java Edition_ protocol.])[
  #table(
    columns: 2,
    inset: 2mm,
    table.header([Phase], [Descriptions]),
    [Handshake], [Initiate connection],
    [Status], [Ping and server MOTD, player count, icon, etc.],
    [Login], [Initiates compression, encryption and authentication],
    [Configuration], [Pre-play setup, resource packs and cookies],
    [Play], [All "gameplay" information, chat messages, and world data]
  )
]

== Ettercap

Ettercap describes itself as a "comprehensive suite for man in the middle attacks". It features host analysis, packet sniffing on live connections and the dissection of packet protocols for more nuanced behaviors. Though they provide several methods to MITM, we utilized ARP spoofing for our testing, even if, theoretically, any method that allows the packets to cross the desired network interface would work fine. ARP spoofing is executed when an attacker broadcasts intentionally falsified ARP packets onto a local area network, asking its own MAC address to be associated with a target hosts IP address [2].

With this is mind, one requirement of our tool to behave properly is that the attacker must be on the same LAN as the target. Typically, the ARP spoofing is done bi-directionally such that the default gateway believes the attacker to be the target, and the target believes the attacker to be the default gateway. Often an attacker will simply pass along packets to their original intended destination untouched, as our tool does, but it is possible to manipulate and change packet before forwarding.

Our tool assumes that a MITM attack across the given network interface is already in action. However, if the user wanted to utilize the tool for themselves, because the protocol parser simply capture packets on a network interface, the tool would function identically if the server and client were on the same host, without the need for Ettercap.

= Codebase

Three main facets make up the programming effort for the tool: the protocol parser, the web server, and the frontend. Rust was the chosen backend language, powering the protocol parser and the web server. The frontend was built in standard HTML, CSS and JavaScript.

== Third-party Tooling

The backend relies on a few dependencies that make the protocol parsing and web hosting possible. First, `pcap` [3] is a common UNIX packet capturing library that allows our parser to read bytes going over a network interface of our choosing. It is used in part by _Wireshark_, the modern defacto packet capturing and traffic sniffing tool. The use of `pcap` requires that the binary program either be run with root privileges or given the `CAP_NET_RAW` Linux capability before running.

The `etherparse` [4] library streamlines the parsing of packet based headers for numerous protocols. Our tool utilizes `etherparse` the most when determining packet direction and extracting packet payloads.

Finally, `zlib-rs` [7] is an implementation of the common `zlib` compression library in Rust. The _Minecraft: Java Edition_ protocol supports compression, and if set to do so during the `Login` phase, any packet exceeding the set threshold length will have its payload compressed. This does not include the mandatory `length` and `id` fields .

To host the frontend and transfer data, we made use of the `axum` [8] library for modular, asynchronous HTTP routing. Similar to Express.js, the library makes setting up a server as a simple as a few lines of code.

== Backend

=== Multithreading

The webserver and protocol parser are packaged within the same binary. To achieve this, we used the multithreading functions built-in to Rust's standard library. The webserver runs on a separate thread, while the protocol parser occupies the main thread. To safely send data between threads, we use an atomically reference counted (ARC) smart pointer to a read/write lock to maintain Rust's memory safety principles. Both the protocol parser and webserver consume the ARC as a parameter before running, so data can be shared between them in a thread safe manner.

=== Protocol parser 

Protocol parsing required extensive back and forth with the community documented protocol spec, and evaluation of the entire protocol to determine what facets were necessary, and which could be ignored. Parsing starts by utilizing `pcap` to read TCP segment bytes from a network interface. Each packet was sliced using `etherparse`, and the direction and payload were copied out.

For most packets, parsing was as easy as immediately decoding the `length` and `id` fields, but some TCP segments were continuations of previously sent segments, and did not include these. For this reason, a buffer system was implemented to reconstruct packets split across segments. When a new segment reaches the protocol parser, the reported length of the payload is compared to the actual length of bytes read. If there are fewer bytes than advertised, the received segment, and subsequent segments, have their bytes buffered until the advertised amount of bytes are received.

Once a fully formed packet is found, it is parsed according to the phase of the protocol, the packet id, and the packet direction [5]. Most packets are simply dropped, as they don't provide any use to building game context. However, when a packet regarding entity movement, player chat, or other noteworthy game interactions come by, the protocol parser will write to a globally accessible `GameState` structure, so that the web server may reflect the changes in the game.

=== Webserver

By default, the webserver is configured to run on port 3001, although this can be changed by modifying the program constants in `main.rs`. The webserver consists of 3 API endpoints. First is a simple health check at `/api/health`. Second is a request to get the server info at `/api/server-info`. This endpoint returns a JSON with 3 items: The IP address of the Minecraft server, the IP of the client being intercepted, and the seed of the Minecraft world. All 3 of these values are program constants in `main.rs`. This endpoint exists to send that data to the frontend for display. The final API endpoint is the most important: `/api/game_state`. This endpoint is what queries the protocol parser for the information it has at the current moment, sending the frontend a concise, readable JSON containing the player's position, as well as any surrounding entities and their positions.

After including all endpoints, the server defaults to serving files. We map the root path `/` to our frontend directory, so that going to `localhost:3001` takes the user directly to the main (and only) page of the tool.

== Frontend

For simplicity, we opted to use plain HTML, CSS and JavaScript to assemble the frontend. In the `src/frontend` directory is a less than 50-line `index.html`. It references a single `style.css` stylesheet. The bulk of the logic in the frontend is done by various JavaScript functions and utilities.

First is the `api.js` script, which is responsible for all fetch requests to the backend. It also makes the initial call to the backend to retrieve the server info.

#figure(
  caption: [Server Info Fields],
  image(width: 75%, "server_info.png")
)

It is worth noting that our protocol parser cannot determine the seed of the world. In Minecraft, a seed is a 64-bit integer that determines how the game generates the terrain and structures in the World. If two players on the same version of the game use the same seed, Minecraft will construct the exact same world. We reasoned that it would be highly unlikely for the server to ever share the seed with the client, since it is a common occurrence for owners of popular servers to hide the seed from players; after all, if players could recreate a world locally, they could use cheats to find the location of rare materials and structures, giving them an unfair advantage. However, we still wanted to use the seed to draw our map, so we settled for manually extracting the seed of our test world and declaring it as a program constant in `main.rs`. The program can run without a seed; it will simply omit drawing the background of the map.

#figure(
  caption: [The Worldmap as a blank canvas with no seed.],
  image(width: 75%, "map_no_seed.png")
)

The next and largest script is `worldmap.js`, which is responsible for drawing the map and updating the log. The map uses an HTML5 `<canvas>` element and draws everything from scratch. First, the script draws a Cartesian graph. The graph adheres to Minecraft's coordinate standards: `+x` is east, `-x` is west, `+z` is south, and `-z` is north. You can zoom in and out of the graph. As you zoom out further, the graph collapses the distance between cells, allowing the map to concisely capture ranges up to 20,000 blocks in any direction#footnote[This is a manually imposed limitation. Minecraft worlds are procedurally generated, and thus can be infinite in size. But we assumed for the domain of our project that our player would not wander beyond 20,000 blocks]. After drawing the graph, the script calls on helpers to draw the biome map using the seed. Then the graph renders a marker for the player. For simplicity, we chose to use the face of Steve to represent the player, regardless of what skin the player may be using.

The player's marker starts at (0, 0) when the web server first initializes. After initial setup, the worldmap script polls the `/api/game_state` endpoint every 0.5 seconds (a configurable variable). It parses the x and z input, moving the player marker to the specified coordinates. The player also has an orange arrow signifying what direction they are facing.

The game state poll also includes information about entities that are near the player. In Minecraft, an entity is any dynamic, moving object that is not fixed to the standard block grid. This can include animals, monsters, vehicles like minecarts or boats, projectiles, dropped items, etc.

The API sends a list of JSON objects for each entity near the player. Each JSON object contains the entity's unique ID as the key. This ID can be used to differentiate two of the same entity (although we don't actually use this in the frontend). The object contains a number for the entity type, the x and z positions, and the yaw. Whereas the yaw of a player is a 32-bit float, the yaw of an entity is an unsigned 8-bit integer. This prevented us from applying the same Euclidean modulo technique we used to clamp the player's yaw, so we opted to omit it. We matched the entity type in a lookup table that we created to identify the type of entity and map it to its respective image, before placing it on the map. The map redraws every frame without caching, so entities that are removed from the poll between iterations (either because of death or exiting render distance) are no longer rendered.

The worldmap script also contains the function to populate the log, which provides a list of actions taken by the player. Minecraft includes a variety of different packet signals for interacting with various menus and entities, but we decided to limit our packets of interest to opening containers, engaging in combat with entities, dying, sending chat messages, and changing the contents of signs.

Lastly, the frontend contains an assortment of scripts to build a biome map of the world from the seed. One challenge was finding a library that supported Minecraft 26.3, the latest version of the game. Most JavaScript libraries only supported older versions, and we did not want to downgrade and add another constraint. We settled on a fork of Cubiomes [9] that was updated to support 26.3. We compiled the C code to WebAssembly, then invoked the binary with several JavaScript helpers. The binary draws a colorful list of all the biomes in the world. Because this addition is more cosmetic and less oriented around networks, we made use of AI to accelerate compilation and debugging of the WebAssembly.

= Future Work

There were some shortcomings within the project that were not anticipated. First and foremost, we intended to maintain compatibility with entirely unmodified servers. The problem we came across is that the stock _Minecraft: Java Edition_ server software comes with encryption enabled by default. This is a problem because during the `login` phase, the server can request an encrypted connection, after which the client and server do an RSA key exchange and have all subsequent communications encrypted using AES. This is not something we can overcome easily, and generally, undermining the gold standard for asymmetric cryptographic key generation is designed to be impossible. Therefore, for our tool to work, the communications either need to be decrypted prematurely or the server must have `online-mode` disabled, available within the server configuration file, `server.properties`. This server property forgoes new client connections needing to verify with official Mojang account servers, and doesn't initiate an encrypted connection during the `login`.

Another hiccup of note is the potential delay and packet loss through ettercap. When doing simple web browsing or idling on the desktop, man-in-the-middle attacks are nearly unnoticeable. But if large amounts of fast moving traffic now need to take an extra hop, as well as deal with queuing delay from an attacking host, the longer RTT and packet loss become noticeable, especially from a gameplay perspective. In our testing, this degradation of performance was inconsistent, leaving some targets with suspiciously poor performance, and others none the wiser.

Finally, the protocol parser must read all the communicated packets starting from the `handshake` phase. If the tool starts to listen after the target has already joined a server and is in active communication, the parser has no context on compression status, encryption status, and various game state properties. This limitation could be overcome through trial and error methods, such as incrementally finding the compression threshold, or decoding a packet with the same `id` and direction in multiple phases until it parses correctly.

= Sample Results

Here, we provide a small demonstration of the tool, using a simple topology. In ettercap, the attacker will search for available hosts, revealing the server and client IPv4 addresses. The attacker then marks them as targets A and B respectively, and starts the ARP spoofing attack.

#figure(caption: [Diagram of network topology for the demonstration])[
  #grid(
    columns: 5,
    column-gutter: 2mm,
    align: horizon,
    rect(inset: 5mm)[
      Client \
      `18.42.101.184` \
      `c0:a8:10:57:40:7c`
    ],
    [$arrow$ \ $arrow.l$],
    rect(inset: 5mm)[
      Attacker \
      `18.42.101.187` \
      `dc:97:ba:df:ca:41`
    ],
    [$arrow$ \ $arrow.l$],
    rect(inset: 5mm)[
      Server \
      `18.42.101.94` \
      `2c:cf:67:2c:20:c7`
    ]
  )
]

Then our tool can be started. The program serves the frontend at `localhost:3001` and idles until game packets from the `handshake` phase roll through, after which it begins to step through the phases as it reads packets, until the player establishes a connection to a game server.

#figure(
  caption: [Open game client],
  image(width: 75%, "client.png")
) <client>

Once the client connects, the protocol parser begins to decode all the packets it can, filtering what is given to the web server, and what is ignored. The frontend continually makes HTTP GET requests to the web server for updated data, and visualizes changes in game state in real time.

The map reveals the position of the player and all in-game entities [6]. For the player specifically, the yaw is displayed as a small orange arrow in the direction the player is looking. For other entities, if we included an image representing it, that image will be positioned appropriately, otherwise the entity will be represented by a simple white square.

On the right is the action log, an updating list of important in game events, paired with their timestamps. The events we chose to log are opening containers, closing containers, chat messages, player death, item renaming and updating signs, many of which are seen in @attacker.

#figure(
  caption: [Tool web UI, with map on the left and action log on the right],
  image(width: 75%, "attacker.png")
) <attacker>

= Contributions

*Harrison DiAmbrosio* - Protocol parser, ettercap man-in-the-middle

*Adrian Cervera* - Webserver, multithreaded architecture, frontend JavaScript + styling

= References

1. Minecraft Wiki. "Java Edition protocol: Data types." \
  #link("https://minecraft.wiki/w/Java_Edition_protocol/Data_types")

2. Wikipedia. "ARP spoofing." \
  #link("https://en.wikipedia.org/wiki/ARP_spoofing")

3. Rust pcap. "pcap: A Rust library for packet capture." \
  #link("https://github.com/rust-pcap/pcap")

4. Julian Schmid. "etherparse." \
  #link("https://github.com/JulianSchmid/etherparse")

5. Minecraft Wiki. "Java Edition protocol: Packets." \
  #link("https://minecraft.wiki/w/Java_Edition_protocol/Packets")

6. Minecraft Wiki. "Java Edition protocol." \
  #link("https://minecraft.wiki/w/Java_Edition_protocol")

7. Trifecta Tech Foundation. "zlib-rs." \
  #link("https://github.com/trifectatechfoundation/zlib-rs")

8. Axum: HTTP routing and request-handling library for Rust that focuses on ergonomics and modularity \
  #link("https://github.com/tokio-rs/axum")

9. Conflux-Union. "cubiomes": C library that mimics the Minecraft biome generation. \
  #link("https://github.com/Conflux-Union/cubiomes")
