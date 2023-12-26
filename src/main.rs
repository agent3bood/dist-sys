use std::collections::{ HashMap, HashSet};
use std::io;
use std::io::{BufRead, Write};
use serde::{Deserialize, Serialize};

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
}

impl Node {
    fn init(id: String, ids: HashSet<String>) -> Node {
        Node { id, ids, msg_id: 0 }
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

    // fn reply(&self, node: &Node) {
    //     let reply: Msg;
    //     match &self.body {
    //         MsgBody::Echo(body) => {
    //             reply = Msg {
    //                 src: node.id.clone(),
    //                 dest: self.src.clone(),
    //                 body: MsgBody::EchoOk(EchoOk {
    //                     msg_id: node.get_msg_id(),
    //                     in_reply_to: body.msg_id.clone(),
    //                     echo: body.echo.clone(),
    //                 }),
    //             };
    //         }
    //         MsgBody::Init(body) => {}
    //         _ => {}
    //     }
    // }
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
