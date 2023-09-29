# relay-rank

This is a tool to rank relays, using the output of the gossip nostr client.

First, after having used gossip for a while, create a relays.json file like this:

````
gossip print_relays > relays.json
````

Then edit that file to prune the junk from the top.

Then run the `run.sh` script from this directory, with the relays.json file in
the parent directory.
