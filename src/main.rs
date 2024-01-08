use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{BufRead, Write};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn main() {
    let mut nodes: HashMap<String, Node> = HashMap::new();
    for line in io::stdin().lock().lines() {
        match line {
            Ok(line) => {
                let msg = Msg::from_json(&line);
                match &msg {
                    Ok(msg) => {
                        match &msg.body {
                            MsgBody::Init(body) => {
                                nodes.insert(body.id.clone(), Node::init(body.id.clone(), body.ids.clone()));
                                nodes.get_mut(&msg.dest).unwrap().handle(&msg);
                            }
                            _ => {
                                nodes.get_mut(&msg.dest).unwrap().handle(&msg);
                            }
                        }
                    }
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

struct Node {
    id: String,
    ids: HashSet<String>,
    msg_id: usize,
    uuid: Uuid,
    messages: Vec<usize>,
    neighbors: Vec<String>,
    counter: usize,
}

impl Node {
    fn init(id: String, ids: HashSet<String>) -> Node {
        Node { id, ids, msg_id: 0, uuid: Uuid::new_v4(), messages: Vec::default(), neighbors: Vec::default(), counter: 0 }
    }

    fn handle(&mut self, msg: &Msg) {
        match &msg.body {
            MsgBody::Echo(body) => {
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::EchoOk(EchoOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: body.msg_id,
                        echo: body.echo.clone(),
                    }),
                }.write();
            }
            MsgBody::EchoOk(_) => {}
            MsgBody::Init(body) => {
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::InitOk(InitOk {
                        in_reply_to: body.msg_id,
                    }),
                }.write();
            }
            MsgBody::InitOk(_) => {}
            MsgBody::Generate(body) => {
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::GenerateOk(GenerateOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: body.msg_id,
                        id: Uuid::new_v4().as_u128(),
                    }),
                }.write();
            }
            MsgBody::GenerateOk(_) => {}
            MsgBody::Broadcast(body) => {
                let message = body.message.clone();
                if !self.messages.contains(&message) {
                    self.messages.push(message);
                    for i in 0..self.neighbors.len() {
                        let neighbor = self.neighbors.get(i).unwrap();
                        Msg {
                            src: self.id.clone(),
                            dest: neighbor.clone(),
                            body: MsgBody::Broadcast(Broadcast {
                                msg_id: self.get_msg_id(),
                                message,
                            }),
                        }.write();
                    }
                }
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::BroadcastOk(BroadcastOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: body.msg_id,
                    }),
                }.write();
            }
            MsgBody::BroadcastOk(_) => {}
            MsgBody::Read(body) => {
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::ReadOk(ReadOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: body.msg_id,
                        messages: self.messages.clone(),
                        value: self.counter,
                    }),
                }.write();
            }
            MsgBody::ReadOk(_) => {}
            MsgBody::Topology(body) => {
                if let Some(neighbors) = body.topology.get(&self.id) {
                    self.neighbors = neighbors.clone();
                }
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::TopologyOk(TopologyOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: body.msg_id,
                    }),
                }.write();
            }
            MsgBody::TopologyOk(_) => {}
            MsgBody::Add(body) => {
                self.counter += body.delta;
                Msg {
                    src: self.id.clone(),
                    dest: msg.src.clone(),
                    body: MsgBody::AddOk(AddOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: body.msg_id,
                    }),
                }.write();
            }
            MsgBody::AddOk(_) => {}
        }
    }

    fn get_msg_id(&mut self) -> usize {
        let id = self.msg_id;
        self.msg_id += 1;
        id
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Msg {
    src: String,
    dest: String,
    body: MsgBody,
}

impl Msg {
    fn from_json(line: &str) -> Result<Msg, serde_json::Error> {
        serde_json::from_str(line)
    }

    fn write(&self) {
        let reply = serde_json::to_string(self).unwrap();
        io::stdout().write(reply.as_bytes()).expect("panicked while sending message");
        io::stdout().write("\n".as_bytes()).expect("panicked while sending message");
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(tag = "type")]
enum MsgBody {
    #[serde(rename = "echo")]
    Echo(Echo),
    #[serde(rename = "echo_ok")]
    EchoOk(EchoOk),
    #[serde(rename = "init")]
    Init(Init),
    #[serde(rename = "init_ok")]
    InitOk(InitOk),
    #[serde(rename = "generate")]
    Generate(Generate),
    #[serde(rename = "generate_ok")]
    GenerateOk(GenerateOk),
    #[serde(rename = "broadcast")]
    Broadcast(Broadcast),
    #[serde(rename = "broadcast_ok")]
    BroadcastOk(BroadcastOk),
    #[serde(rename = "read")]
    Read(Read),
    #[serde(rename = "read_ok")]
    ReadOk(ReadOk),
    #[serde(rename = "topology")]
    Topology(Topology),
    #[serde(rename = "topology_ok")]
    TopologyOk(TopologyOk),
    #[serde(rename = "add")]
    Add(Add),
    #[serde(rename = "add_ok")]
    AddOk(AddOk),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Init {
    msg_id: usize,
    #[serde(rename = "node_id")]
    id: String,
    #[serde(rename = "node_ids")]
    ids: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct InitOk {
    in_reply_to: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Echo {
    msg_id: usize,
    echo: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct EchoOk {
    msg_id: usize,
    in_reply_to: usize,
    echo: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Generate {
    msg_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct GenerateOk {
    msg_id: usize,
    in_reply_to: usize,
    id: u128,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Broadcast {
    msg_id: usize,
    message: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct BroadcastOk {
    msg_id: usize,
    in_reply_to: usize,
}


#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Read {
    msg_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct ReadOk {
    msg_id: usize,
    in_reply_to: usize,
    // used in broadcast
    #[serde(skip_serializing_if = "Vec::is_empty")]
    messages: Vec<usize>,
    // used in add
    #[serde(skip_serializing_if = "is_zero")]
    value: usize,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Topology {
    msg_id: usize,
    topology: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct TopologyOk {
    msg_id: usize,
    in_reply_to: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Add {
    msg_id: usize,
    delta: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct AddOk {
    msg_id: usize,
    in_reply_to: usize,
}

mod tests {
    use crate::*;

    #[test]
    fn t1() {
        let s = r#"{
  "src": "c1",
  "dest": "n1",
  "body": {
    "type": "echo",
    "msg_id": 1,
    "echo": "Please echo 35"
  }
}"#;

        let msg = Msg::from_json(s).unwrap();
        assert_eq!(msg.src, "c1");
        assert_eq!(msg.dest, "n1");
        assert_eq!(msg.body, MsgBody::Echo(Echo { msg_id: 1, echo: "Please echo 35".to_string() }));
    }


    #[test]
    fn t3() {
        let msg = Msg {
            src: "n1".to_string(),
            dest: "n2".to_string(),
            body: MsgBody::InitOk(InitOk {
                in_reply_to: 1,
            }),
        };
        let s = serde_json::to_string(&msg).unwrap();
        println!("{}", s);
    }
}
