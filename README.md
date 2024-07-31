# test_soramitsu

Test task on vacancy Trainee Rust Developer.

## Content

<ol>
<li>
    <a href="#content">Content</a>
</li>
<li>
    <a href="#launch-guide">Launch guide</a>
</li>
<li>
    <a href="#task">Task</a>
</li>
<li>
    <a href="#application-behavior">Application behavior</a>
</li>
</ol>

## Launch guide 

To start a peer use 

```shell
cargo run --bin test_soramitsu -- --period=<period> --port=<port> --connect=<peer_address>
```

This will run a peer on port `port` that will send gossip messages each `period` seconds. `connect` is optional, if specified, then the peer will ask other peer with specified address about other peers before sending and listening.

## Task

Your task is to design a simple p2p gossiping application in Rust. The peer should have a cli interface to start it and connect itself to the other peers. Once connected, the peer should send a random gossip message to all the other peers every N seconds. The messaging period should also be specifiable in the command line. When a peer receives a message from the other peers, it should print it in the console.

## Application behavior

Each peer sends and accepts messages to other peers using UDP sockets, there are five types of messages:

<ol> 
<li> "new" - add the sender of the message to the list of known peers in order to send gossips to it in the future. </li>
<li> "msg" - message that contains gossip. </li>
<li> "list" - request for a list of peers </li>
<li> "list_response" - response on "list" message, after receiving this type of message, the peer sends "new" message to the peer specified in the message. </li>
<li> "stop" - indicates that the sender is closed, so there is no need to send gossips to it. </li>
</ol>