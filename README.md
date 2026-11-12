# `mc_mitm`: Minecraft MITM Context Tool 

Dr. Selvan Snow | CS 513: Computer Networks
By: Harrison DiAmbrosio

A tool designed to be used with a MITM utility in order to build context for client connections to Minecraft: Java Edition servers. The tool utilized `pcap` to capture packets over a network interface, parse the data according to the community documented protocol spec and build game context overtime presented to the attacker via a web UI. 

The project is further detailed in the submission report, found [here](https://github.com/EggbertFluffle/mc_mitm/blob/main/report/CS513_Minecraft_MITM.pdf).

## Features

We support many features in the tool, such as:

- Real-time logging for several in game events
- Live top-down map of the game world, including player and game entity positions
- Handling of communications compressed with `zlib`

Some things are not included in the project, such as:
- Support for decryption of encrypted communications with servers using `online-mode` (the default)
- Dynamic seed retrieval for drawing biome boundaries
- Storing chunk data for reference in packets that require context from block coordinates
